use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    clone::Clone,
};
use utoipa::{self, ToSchema};
use itertools::Itertools;

// use actix_ws::{Session};
use futures::stream::{FuturesUnordered, StreamExt};
use tokio::sync::Mutex;

use serde::{Serialize, Deserialize};
use serde_json;

use crate::{
    messages::{
        MessageTypes,
        Message
        // MessageUpateType
    },
    session::WsChatSession,
    db::models::User
};

#[derive(Serialize, Deserialize)]
pub struct AuthCookie {
    pub user_id: i64,
    pub session_id: String
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct LoginEvent {
    #[schema(example = "test@test.com")]
    pub email: String,
    #[schema(example = "abcd1234")]
    pub password: String,
    #[schema(example = "bruhmeme")]
    pub code: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SignUpEvent {
    #[schema(example = "test")]
    pub username: String,
    #[schema(example = "test@test.com")]
    pub email: String,
    #[schema(example = "abcd1234")]
    pub password: String,
    #[schema(example = "bruhmeme")]
    pub code: String,
}

#[derive(Serialize, Deserialize)]
pub struct VerifyEvent {
    pub code: i64
}

#[derive(Serialize, Deserialize)]
pub struct ClientEvent {
    #[serde(flatten)]
    pub data: MessageTypes,
    pub client_id: usize,
    pub room: String,
}

impl utoipa::ToSchema for ClientEvent {
    fn schema() -> utoipa::openapi::schema::Schema {
        utoipa::openapi::ObjectBuilder::new()
            .property(
                "client_id",
                utoipa::openapi::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::SchemaType::Integer)
                    .format(Some(utoipa::openapi::SchemaFormat::Int64)),
            )
            .required("client_id")
            .property(
                "room",
                utoipa::openapi::Object::with_type(utoipa::openapi::SchemaType::String),
            )
            .required("room")
            .property(
                "type",
                utoipa::openapi::Object::with_type(utoipa::openapi::SchemaType::String),
            )
            .required("room")
            .property(
                "data",
                utoipa::openapi::Object::with_type(utoipa::openapi::SchemaType::Object),
            )
            .required("data")
            .example(Some(serde_json::json!({
                "type": "MessageCreate",
                "data": {
                    "content": "test123"
                },
                "room": "5fe9d2ab-2174-4a30-8245-cc5de2563dce",
                "client_id": 1
            })))
            .into()
    }
}

#[derive(Clone)]
pub struct Chat {
    pub inner: Arc<Mutex<ChatInner>>,
}

pub struct ChatInner {
    pub sessions: HashMap<usize, Vec<WsChatSession>>,
    pub rooms: HashMap<String, HashSet<usize>>,
    pub visitor_count: Arc<AtomicUsize>
}

impl Chat {
    pub fn new(visitor_count: Arc<AtomicUsize>, room_names: Vec<String>) -> Self {
        let mut rooms = HashMap::new();
        rooms.insert("5fe9d2ab-2174-4a30-8245-cc5de2563dce".to_owned(), HashSet::new());
        for name in room_names {
            rooms.insert(name, HashSet::new());
        }
        Chat {
            inner: Arc::new(Mutex::new(ChatInner {
                sessions: HashMap::new(),
                rooms,
                visitor_count,
            })),
        }
    }

    pub async fn get_sessions_by_user_id(&self, user_id: usize) -> Option<Vec<WsChatSession>> {
        let inner = self.inner.lock().await;
        if let Some(_) = inner.sessions.get(&user_id) {
            Some(inner.sessions[&user_id].to_owned())
        } else {
            None
        }
    }

    pub async fn insert_session(&self, user_id: usize, session: WsChatSession) {
        let mut inner = self.inner.lock().await;
        inner.sessions.entry(user_id).or_insert_with(Vec::new).push(session);
    }

    pub async fn find_user_by_id(&self, user_id: usize) -> Option<User> {
        let inner = self.inner.lock().await;
        if let Some(ses) = inner.sessions.get(&user_id) {
            if ses.is_empty() {
                None
            } else {
                Some(ses[0].user.clone())
            }
        } else {
            None
        }
    }

    pub async fn list_rooms(&self) -> Vec<String> {
        let mut rooms = Vec::new();
        let inner = self.inner.lock().await;

        for key in inner.rooms.keys() {
            rooms.push(key.to_owned())
        }
        drop(inner);
        rooms.push(self.get_sessions_by_user_id(2).await.unwrap().iter().map(|ses| ("ses: ".to_string() + &ses.session_id).to_string()).join(", "));
        rooms
    }

    pub async fn new_visitor(&self) -> usize {
        let inner = self.inner.lock().await;
        inner.visitor_count.fetch_add(1, Ordering::SeqCst)
    }

    pub async fn leave_room(&self, channel_id: String, user_id: usize) {
        log::info!("{} id leaving channel_id {}", user_id, channel_id);

        {
            let mut inner = self.inner.lock().await;
            if let Some(sessions) = inner.rooms.get_mut(&channel_id) {
                if sessions.remove(&user_id) {
                    if sessions.len() == 0 {
                        inner.rooms.remove(&channel_id);
                        return;
                    }
                    drop(inner);
                    self.send_message(&channel_id, MessageTypes::MessageCreate(Message::system("Someone left".to_string(), &channel_id.clone(), 0))).await;
                }
            }
        }
    }

    pub async fn join_room(&self, channel_id: String, user_id: usize) {
        log::info!("{} id joining channel_id {}", user_id, channel_id);
        // let mut rooms = Vec::new();
        // drop MutexGuard
        {
            let mut inner = self.inner.lock().await;

            /* No longer a feature */
            // // remove user from their old room (intentional feature)

            // for (n, sessions) in &mut inner.rooms {
            //     if sessions.remove(&user_id) {
            //         rooms.push(n.to_owned());
            //     }
            // }

            inner.rooms
                .entry(channel_id.clone())
                .or_insert_with(HashSet::new)
                .insert(user_id);    
        }

        // log::info!("ROOMS: {:?}", rooms);
        // for room in rooms {
        //     self.send_message(&room, MessageTypes::MessageCreate(MessageCreateType{content: "Someone disconnected".to_string()})).await;
        // }

        self.send_message(&channel_id, MessageTypes::MessageCreate(Message::system("Someone joined".to_string(), &channel_id.clone(), 0))).await;
    }

    // pub async fn insert(&self, user_id: usize, session: Session) {
    //     let mut inner = self.inner.lock().await;
    //     let values = inner.sessions.entry(user_id).or_insert_with(Vec::new);
    //     values.push(session);
    // }

    pub async fn insert_id(&self, room: String, user_id: usize) {
        let mut inner = self.inner.lock().await;
        let values = inner.rooms.entry(room).or_insert_with(HashSet::new);
        values.insert(user_id);
    }

    // send global. Please try to not use this
    pub async fn send(&self, msg: MessageTypes) {
        let mut inner = self.inner.lock().await;
        let unordered = FuturesUnordered::new();
        for (user_id, sessions) in inner.sessions.drain() {
            let msg = msg.clone();
            unordered.push(async move {
                let mut results = Vec::new();
                for mut session in sessions {
                    // let mut buf = [0u8; 100];
                    // let writer = SliceWrite::new(&mut buf[..]);
                    // let mut ser = Serializer::new(writer);
                    // msg.serialize(&mut ser).unwrap();
                    // let writer = ser.into_inner();
                    // let size = writer.bytes_written();
                    // let b = buf.to_vec();
                    let res = session.session.binary(serde_cbor::to_vec(&msg).unwrap()).await;
                    // let res = session.session.text(serde_json::to_string(&msg).unwrap()).await;
                    results.push(res.map(|_| session).map_err(|_| log::info!("Dropping session")));
                }
                (user_id, results)
            });
        }
        drop(inner);
        let res = unordered.collect::<Vec<(usize, Vec<Result<WsChatSession, ()>>)>>().await;
        let mut inner = self.inner.lock().await;
        for (user_id, results) in res {
            inner.sessions.insert(user_id, results.into_iter().filter_map(|i| i.ok()).collect());
        }
    }

    // send a message to a room
    // can add a skip_id parameter
    pub async fn send_message(&self, room: &str, message: MessageTypes) {
        let mut inner = self.inner.lock().await;
        // hahaha lmao we need an ordered list of futures bruh Rust
        let unordered = FuturesUnordered::new();
        log::info!("SENDING TO ROOM: {}", room);
        if let Some(users) = inner.rooms.get(room) {
            log::info!("ROOM HAS USERS: {:?}", users);
            
            let users_cloned = users.clone();
            for (user_id, _) in inner.sessions.clone() {
                if users_cloned.contains(&user_id) {
                    let msg = message.clone();
                    if let Some(sessions) = inner.sessions.remove(&user_id) {
                        log::info!("sending to user: {}", user_id);
                        unordered.push(async move {
                            let mut results = Vec::new();
                            for mut session in sessions {
                                println!("{}", serde_json::to_string(&msg).unwrap());
                                let res = session.session.binary(serde_cbor::to_vec(&msg).unwrap()).await;
                                // let res = session.session.text(serde_json::to_string(&msg).unwrap()).await;
                                results.push(res.map(|_| session).map_err(|_| log::info!("Dropping session")));
                            }
                            (user_id, results)
                        });
                    }
                }
            }
        } else {
            return;
        }
        drop(inner);
        let res = unordered.collect::<Vec<(usize, Vec<Result<WsChatSession, ()>>)>>().await;
        let mut inner = self.inner.lock().await;
        for (user_id, results) in res {
            inner.sessions.insert(user_id, results.into_iter().filter_map(|i| i.ok()).collect());
        }
    }

    // send a message to all the sessions active on user_id
    #[allow(dead_code)]
    pub async fn send_to_id(&self, id: usize, message: MessageTypes) {
        let mut inner = self.inner.lock().await;
        let msg = message.clone();
        if let Some(sessions) = inner.sessions.remove(&id) {
            let mut results = Vec::new();
            for mut session in sessions {
                let res = session.session.binary(serde_cbor::to_vec(&msg).unwrap()).await;
                // let res = session.session.text(serde_json::to_string(&msg).unwrap()).await;
                results.push(res.map(|_| session).map_err(|_| log::info!("Dropping session")));
            }
            inner.sessions.insert(id, results.into_iter().filter_map(|i| i.ok()).collect());
        }
    }
}