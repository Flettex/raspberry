use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    clone::Clone,
};
use utoipa::{self, Component};

use actix_ws::{Session};
use futures::stream::{FuturesUnordered, StreamExt};
use tokio::sync::Mutex;

use serde::{Serialize, Deserialize};
use serde_json;

use crate::messages::{
    MessageTypes,
    MessageCreateType,
    // MessageUpateType
};

#[derive(Serialize, Deserialize)]
pub struct AuthCookie {
    pub user_id: i64,
    pub session_id: String
}

#[derive(Serialize, Deserialize, Component)]
pub struct LoginEvent {
    #[component(example = "test@test.com")]
    pub email: String,
    #[component(example = "abcd1234")]
    pub password: String
}

#[derive(Serialize, Deserialize)]
pub struct SignUpEvent {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct VerifyEvent {
    pub code: i32
}

#[derive(Serialize, Deserialize)]
pub struct ClientEvent {
    #[serde(flatten)]
    pub data: MessageTypes,
    pub client_id: usize,
    pub room: String,
}

impl utoipa::Component for ClientEvent {
    fn component() -> utoipa::openapi::schema::Component {
        utoipa::openapi::ObjectBuilder::new()
            .property(
                "client_id",
                utoipa::openapi::PropertyBuilder::new()
                    .component_type(utoipa::openapi::ComponentType::Integer)
                    .format(Some(utoipa::openapi::ComponentFormat::Int64)),
            )
            .required("client_id")
            .property(
                "room",
                utoipa::openapi::Property::new(utoipa::openapi::ComponentType::String),
            )
            .required("room")
            .property(
                "type",
                utoipa::openapi::Property::new(utoipa::openapi::ComponentType::String),
            )
            .required("room")
            .property(
                "data",
                utoipa::openapi::Property::new(utoipa::openapi::ComponentType::Object),
            )
            .required("data")
            .example(Some(serde_json::json!({
                "type": "MessageCreate",
                "data": {
                    "content": "test123"
                },
                "room": "Main",
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
    pub sessions: HashMap<usize, Vec<Session>>,
    pub rooms: HashMap<String, HashSet<usize>>,
    pub visitor_count: Arc<AtomicUsize>
}

impl Chat {
    pub fn new(visitor_count: Arc<AtomicUsize>, room_names: Vec<String>) -> Self {
        // TODO: make visitor count actually work
        let mut rooms = HashMap::new();
        rooms.insert("Main".to_owned(), HashSet::new());
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

    pub async fn list_rooms(&self) -> Vec<String> {
        let mut rooms = Vec::new();
        let inner = self.inner.lock().await;

        for key in inner.rooms.keys() {
            rooms.push(key.to_owned())
        }

        rooms
    }

    pub async fn new_visitor(&self) -> usize {
        let inner = self.inner.lock().await;
        inner.visitor_count.fetch_add(1, Ordering::SeqCst)
    }

    pub async fn leave_room(&self, room: String, user_id: usize) {
        log::info!("{} id leaving room {}", user_id, room);

        {
            let mut inner = self.inner.lock().await;
            if let Some(sessions) = inner.rooms.get_mut(&room) {
                if sessions.remove(&user_id) {
                    if sessions.len() == 0 {
                        inner.rooms.remove(&room);
                        return;
                    }
                    drop(inner);
                    self.send_message(&room, MessageTypes::MessageCreate(MessageCreateType{content: "Someone disconnected".to_string(), room: room.clone()})).await;
                }
            }
        }
    }

    pub async fn join_room(&self, room: String, user_id: usize) {
        log::info!("{} id joining room {}", user_id, room);
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
                .entry(room.clone())
                .or_insert_with(HashSet::new)
                .insert(user_id);    
        }

        // log::info!("ROOMS: {:?}", rooms);
        // for room in rooms {
        //     self.send_message(&room, MessageTypes::MessageCreate(MessageCreateType{content: "Someone disconnected".to_string()})).await;
        // }

        self.send_message(&room, MessageTypes::MessageCreate(MessageCreateType{content: "Someone connected".to_string(), room: room.clone()})).await;
    }

    pub async fn insert(&self, user_id: usize, session: Session) {
        let mut inner = self.inner.lock().await;
        let values = inner.sessions.entry(user_id).or_insert_with(Vec::new);
        values.push(session);
    }

    pub async fn insert_id(&self, room: String, user_id: usize) {
        let mut inner = self.inner.lock().await;
        let values = inner.rooms.entry(room).or_insert_with(HashSet::new);
        values.insert(user_id);
    }

    // send global.
    pub async fn send(&self, msg: MessageTypes) {
        let mut inner = self.inner.lock().await;
        let unordered = FuturesUnordered::new();
        for (user_id, sessions) in inner.sessions.drain() {
            let msg = msg.clone();
            unordered.push(async move {
                let mut results = Vec::new();
                for mut session in sessions {
                    let res = session.text(serde_json::to_string(&msg).unwrap()).await;
                    results.push(res.map(|_| session).map_err(|_| log::info!("Dropping session")));
                }
                (user_id, results)
            });
        }
        drop(inner);
        let res = unordered.collect::<Vec<(usize, Vec<Result<Session, ()>>)>>().await;
        let mut inner = self.inner.lock().await;
        for (user_id, results) in res {
            inner.sessions.insert(user_id, results.into_iter().filter_map(|i| i.ok()).collect());
        }
    }

    // send a message to a room
    // can add a skip_id parameter
    pub async fn send_message(&self, room: &str, message: MessageTypes) {
        let mut inner = self.inner.lock().await;
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
                                let res = session.text(serde_json::to_string(&msg).unwrap()).await;
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
        let res = unordered.collect::<Vec<(usize, Vec<Result<Session, ()>>)>>().await;
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
                let res = session.text(serde_json::to_string(&msg).unwrap()).await;
                results.push(res.map(|_| session).map_err(|_| log::info!("Dropping session")));
            }
            inner.sessions.insert(id, results.into_iter().filter_map(|i| i.ok()).collect());
        }
    }
}