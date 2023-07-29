use std::{
    clone::Clone,
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use utoipa::{self, ToSchema};

// use actix_ws::{Session};
use futures::stream::{FuturesUnordered, StreamExt};
use tokio::sync::Mutex;

use serde::{Deserialize, Serialize};
use serde_json;

use crate::{
    controllers::ws::WsMsgType,
    db::models::User,
    messages::{
        Message, // MessageUpateType
        MessageTypes,
    },
    session::WsChatSession,
};

#[derive(Serialize, Deserialize)]
pub struct AuthCookie {
    pub user_id: i64,
    pub session_id: String,
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
    pub code: i64,
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
                    .format(Some(utoipa::openapi::SchemaFormat::KnownFormat(
                        utoipa::openapi::schema::KnownFormat::Int64,
                    ))),
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
    // DMs will use this
    pub sessions: Arc<Mutex<HashMap<usize, Vec<WsChatSession>>>>,
    // Channels will use this
    // New update will no longer use channels
    // pub rooms: Arc<Mutex<HashMap<String, HashSet<usize>>>>,
    // Guilds will use this
    pub guilds: Arc<Mutex<HashMap<String, HashSet<usize>>>>,
    // This is useless
    pub visitor_count: Arc<AtomicUsize>,
}

impl Chat {
    pub fn new(visitor_count: Arc<AtomicUsize>) -> Self {
        // let mut rooms = HashMap::new();
        let mut guilds = HashMap::new();
        // rooms.insert(
        //     "5fe9d2ab-2174-4a30-8245-cc5de2563dce".to_owned(),
        //     HashSet::new(),
        // );
        guilds.insert(
            "5fe9d2ab-2174-4a30-8245-cc5de2563dce".to_owned(),
            HashSet::new(),
        );
        Chat {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            // rooms: Arc::new(Mutex::new(rooms)),
            guilds: Arc::new(Mutex::new(guilds)),
            visitor_count,
        }
    }

    #[allow(dead_code)]
    pub async fn get_sessions_by_user_id(&self, user_id: usize) -> Option<Vec<WsChatSession>> {
        let sessions = self.sessions.lock().await;
        if let Some(_) = sessions.get(&user_id) {
            Some(sessions[&user_id].to_owned())
        } else {
            None
        }
    }

    pub async fn insert_session(&self, user_id: usize, session: WsChatSession) {
        let mut sessions = self.sessions.lock().await;
        sessions
            .entry(user_id)
            .or_insert_with(Vec::new)
            .push(session);
    }

    pub async fn find_user_by_id(&self, user_id: usize) -> Option<User> {
        let sessions = self.sessions.lock().await;
        if let Some(ses) = sessions.get(&user_id) {
            if ses.is_empty() {
                None
            } else {
                Some(ses[0].user.clone())
            }
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub async fn list_guilds(&self) -> Vec<String> {
        // Now lists guilds
        let mut rooms = Vec::new();
        let r = self.guilds.lock().await;

        for key in r.keys() {
            rooms.push(key.to_owned())
        }
        // this is not needed since the line below is commented out
        // drop(r);
        // rooms.push(self.get_sessions_by_user_id(2).await.unwrap().iter().map(|ses| ("ses: ".to_string() + &ses.session_id).to_string()).join(", "));
        rooms
    }

    pub async fn new_visitor(&self) -> usize {
        // Arc<AtomicUsize> does not need another layer bruh
        self.visitor_count.fetch_add(1, Ordering::SeqCst)
    }

    // now rooms are guilds
    pub async fn leave_guild(&self, guild_id: String, user_id: usize) {
        log::info!("{} id leaving guild_id {}", user_id, guild_id);

        {
            let mut rooms = self.guilds.lock().await;
            if let Some(sessions) = rooms.get_mut(&guild_id) {
                if sessions.remove(&user_id) {
                    if sessions.len() == 0 {
                        rooms.remove(&guild_id);
                        return;
                    }
                    // needed here, .send_message uses rooms
                    drop(rooms);
                    self.send_guild_message(
                        &guild_id,
                        MessageTypes::MessageCreate(Message::system(
                            "Someone left".to_string(),
                            &guild_id.clone(),
                            0,
                        )),
                    )
                    .await;
                }
            }
        }
    }

    pub async fn join_guild(&self, guild_id: String, user_id: usize) {
        log::info!("{} id joining guild_id {}", user_id, guild_id);

        // drop MutexGuard
        {
            let mut guilds = self.guilds.lock().await;

            guilds
                .entry(guild_id.clone())
                .or_insert_with(HashSet::new)
                .insert(user_id);
        }

        self.send_guild_message(
            &guild_id,
            MessageTypes::MessageCreate(Message::system(
                "Someone joined".to_string(),
                &guild_id.clone(),
                0,
            )),
        )
        .await;
    }

    // Channels are now removed from cache
    // pub async fn join_room(&self, channel_id: String, user_id: usize) {
    //     log::info!("{} id joining channel_id {}", user_id, channel_id);
    //     // let mut rooms = Vec::new();
    //     // drop MutexGuard
    //     {
    //         let mut rooms = self.rooms.lock().await;

    //         /* No longer a feature */
    //         // // remove user from their old room (intentional feature)

    //         // for (n, sessions) in &mut inner.rooms {
    //         //     if sessions.remove(&user_id) {
    //         //         rooms.push(n.to_owned());
    //         //     }
    //         // }

    //         rooms
    //             .entry(channel_id.clone())
    //             .or_insert_with(HashSet::new)
    //             .insert(user_id);
    //     }

    //     // log::info!("ROOMS: {:?}", rooms);
    //     // for room in rooms {
    //     //     self.send_message(&room, MessageTypes::MessageCreate(MessageCreateType{content: "Someone disconnected".to_string()})).await;
    //     // }

    //     self.send_message(
    //         &channel_id,
    //         MessageTypes::MessageCreate(Message::system(
    //             "Someone joined".to_string(),
    //             &channel_id.clone(),
    //             0,
    //         )),
    //     )
    //     .await;
    // }

    // pub async fn insert(&self, user_id: usize, session: Session) {
    //     let mut inner = self.inner.lock().await;
    //     let values = inner.sessions.entry(user_id).or_insert_with(Vec::new);
    //     values.push(session);
    // }

    pub async fn insert_id(&self, room: String, user_id: usize) {
        // insert_id now works on rooms only
        let mut rooms = self.guilds.lock().await;
        let values = rooms.entry(room).or_insert_with(HashSet::new);
        values.insert(user_id);
    }

    // send global. Please try to not use this
    pub async fn send(&self, msg: MessageTypes) {
        let mut sessions = self.sessions.lock().await;
        let unordered = FuturesUnordered::new();
        for (user_id, sessions) in sessions.drain() {
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
                    let res = match session.recv_type {
                        WsMsgType::Json => {
                            session
                                .session
                                .text(serde_json::to_string(&msg).unwrap())
                                .await
                        }
                        WsMsgType::Cbor => {
                            session
                                .session
                                .binary(serde_cbor::to_vec(&msg).unwrap())
                                .await
                        }
                    };
                    // let res = session.session.text(serde_json::to_string(&msg).unwrap()).await;
                    results.push(
                        res.map(|_| session)
                            .map_err(|_| log::info!("Dropping session")),
                    );
                }
                (user_id, results)
            });
        }
        // Not sure why this is needed, it is probably breaking something
        drop(sessions);
        let res = unordered
            .collect::<Vec<(usize, Vec<Result<WsChatSession, ()>>)>>()
            .await;
        let mut sessions = self.sessions.lock().await;
        for (user_id, results) in res {
            sessions.insert(
                user_id,
                results.into_iter().filter_map(|i| i.ok()).collect(),
            );
        }
    }

    // send a message to a room
    // can add a skip_id parameter
    // pub async fn send_message(&self, room: &str, message: MessageTypes) {
    //     let rooms = self.rooms.lock().await;
    //     let mut sessions = self.sessions.lock().await;
    //     // hahaha lmao we need an ordered list of futures bruh Rust
    //     let unordered = FuturesUnordered::new();
    //     log::info!("SENDING TO ROOM: {}", room);
    //     if let Some(users) = rooms.get(room) {
    //         log::info!("ROOM HAS USERS: {:?}", users);

    //         let users_cloned = users.clone();
    //         for (user_id, _) in sessions.clone() {
    //             if users_cloned.contains(&user_id) {
    //                 let msg = message.clone();
    //                 if let Some(sessions) = sessions.remove(&user_id) {
    //                     log::info!("sending to user: {}", user_id);
    //                     unordered.push(async move {
    //                         let mut results = Vec::new();
    //                         for mut session in sessions {
    //                             // println!("{}", serde_json::to_string(&msg).unwrap());
    //                             let res = match session.recv_type {
    //                                 WsMsgType::Json => {
    //                                     session
    //                                         .session
    //                                         .text(serde_json::to_string(&msg).unwrap())
    //                                         .await
    //                                 }
    //                                 WsMsgType::Cbor => {
    //                                     session
    //                                         .session
    //                                         .binary(serde_cbor::to_vec(&msg).unwrap())
    //                                         .await
    //                                 }
    //                             };
    //                             // let res = session.session.text(serde_json::to_string(&msg).unwrap()).await;
    //                             results.push(
    //                                 res.map(|_| session)
    //                                     .map_err(|_| log::info!("Dropping session")),
    //                             );
    //                         }
    //                         (user_id, results)
    //                     });
    //                 }
    //             }
    //         }
    //     } else {
    //         return;
    //     }
    //     drop(rooms);
    //     drop(sessions);
    //     let res = unordered
    //         .collect::<Vec<(usize, Vec<Result<WsChatSession, ()>>)>>()
    //         .await;
    //     let mut sessions = self.sessions.lock().await;
    //     for (user_id, results) in res {
    //         sessions.insert(
    //             user_id,
    //             results.into_iter().filter_map(|i| i.ok()).collect(),
    //         );
    //     }
    // }

    // send a message to a guild
    pub async fn send_guild_message(&self, room: &str, message: MessageTypes) {
        let guilds = self.guilds.lock().await;
        let mut sessions = self.sessions.lock().await;
        // hahaha lmao we need an ordered list of futures bruh Rust
        let unordered = FuturesUnordered::new();
        log::info!("SENDING TO GUILD: {}", room);
        if let Some(users) = guilds.get(room) {
            log::info!("GUILD HAS USERS: {:?}", users);

            let users_cloned = users.clone();
            for (user_id, _) in sessions.clone() {
                if users_cloned.contains(&user_id) {
                    let msg = message.clone();
                    if let Some(sessions) = sessions.remove(&user_id) {
                        log::info!("sending to user: {}", user_id);
                        unordered.push(async move {
                            let mut results = Vec::new();
                            for mut session in sessions {
                                // println!("{}", serde_json::to_string(&msg).unwrap());
                                let res = match session.recv_type {
                                    WsMsgType::Json => {
                                        session
                                            .session
                                            .text(serde_json::to_string(&msg).unwrap())
                                            .await
                                    }
                                    WsMsgType::Cbor => {
                                        session
                                            .session
                                            .binary(serde_cbor::to_vec(&msg).unwrap())
                                            .await
                                    }
                                };
                                // let res = session.session.text(serde_json::to_string(&msg).unwrap()).await;
                                results.push(
                                    res.map(|_| session)
                                        .map_err(|_| log::info!("Dropping session")),
                                );
                            }
                            (user_id, results)
                        });
                    }
                }
            }
        } else {
            return;
        }
        drop(guilds);
        drop(sessions);
        let res = unordered
            .collect::<Vec<(usize, Vec<Result<WsChatSession, ()>>)>>()
            .await;
        let mut sessions = self.sessions.lock().await;
        for (user_id, results) in res {
            sessions.insert(
                user_id,
                results.into_iter().filter_map(|i| i.ok()).collect(),
            );
        }
    }

    // send a message to all the sessions active on user_id
    #[allow(dead_code)]
    pub async fn send_to_id(&self, id: usize, message: MessageTypes) {
        let mut sessions = self.sessions.lock().await;
        let msg = message.clone();
        if let Some(sessions_) = sessions.remove(&id) {
            let mut results = Vec::new();
            for mut session in sessions_ {
                let res = match session.recv_type {
                    WsMsgType::Json => {
                        session
                            .session
                            .text(serde_json::to_string(&msg).unwrap())
                            .await
                    }
                    WsMsgType::Cbor => {
                        session
                            .session
                            .binary(serde_cbor::to_vec(&msg).unwrap())
                            .await
                    }
                };
                // let res = session.session.text(serde_json::to_string(&msg).unwrap()).await;
                results.push(
                    res.map(|_| session)
                        .map_err(|_| log::info!("Dropping session")),
                );
            }
            sessions.insert(id, results.into_iter().filter_map(|i| i.ok()).collect());
        }
    }

    pub async fn send_dm(&self, id1: usize, id2: usize, message: MessageTypes) {
        let mut sessions = self.sessions.lock().await;
        let msg = message.clone();
        if let Some(sessions_) = sessions.remove(&id1) {
            let mut results = Vec::new();
            for mut session in sessions_ {
                let res = match session.recv_type {
                    WsMsgType::Json => {
                        session
                            .session
                            .text(serde_json::to_string(&msg).unwrap())
                            .await
                    }
                    WsMsgType::Cbor => {
                        session
                            .session
                            .binary(serde_cbor::to_vec(&msg).unwrap())
                            .await
                    }
                };
                // let res = session.session.text(serde_json::to_string(&msg).unwrap()).await;
                results.push(
                    res.map(|_| session)
                        .map_err(|_| log::info!("Dropping session")),
                );
            }
            sessions.insert(id1, results.into_iter().filter_map(|i| i.ok()).collect());
        }
        if let Some(sessions_) = sessions.remove(&id2) {
            let mut results = Vec::new();
            for mut session in sessions_ {
                let res = match session.recv_type {
                    WsMsgType::Json => {
                        session
                            .session
                            .text(serde_json::to_string(&msg).unwrap())
                            .await
                    }
                    WsMsgType::Cbor => {
                        session
                            .session
                            .binary(serde_cbor::to_vec(&msg).unwrap())
                            .await
                    }
                };
                // let res = session.session.text(serde_json::to_string(&msg).unwrap()).await;
                results.push(
                    res.map(|_| session)
                        .map_err(|_| log::info!("Dropping session")),
                );
            }
            sessions.insert(id1, results.into_iter().filter_map(|i| i.ok()).collect());
        }
        // if let (Some(sessions_1), Some(sessions_2)) = (sessions.remove(&id1), sessions.remove(&id2)) {
        //     let mut results = Vec::new();
        //     for mut session in sessions_1.into_iter().chain(sessions_2.into_iter()) {
        //         let res = match session.recv_type {
        //             WsMsgType::Json => {
        //                 session
        //                     .session
        //                     .text(serde_json::to_string(&msg).unwrap())
        //                     .await
        //             }
        //             WsMsgType::Cbor => {
        //                 session
        //                     .session
        //                     .binary(serde_cbor::to_vec(&msg).unwrap())
        //                     .await
        //             }
        //         };
        //         // let res = session.session.text(serde_json::to_string(&msg).unwrap()).await;
        //         results.push(
        //             res.map(|_| session)
        //                 .map_err(|_| log::info!("Dropping session")),
        //         );
        //     }
        //     sessions.insert(id1, results.into_iter().filter_map(|i| i.ok()).collect());
        // }
    }
}
