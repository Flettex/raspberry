use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicUsize},
        Arc,
    },
    clone::Clone,
};

use actix_ws::{Session};
use futures::stream::{FuturesUnordered, StreamExt};
use tokio::sync::Mutex;

use serde::{Serialize, Deserialize};
use serde_json;

use sqlx::types::{
    chrono::{
        NaiveDateTime,   
    },
    Uuid
};

use crate::format;

#[derive(Serialize, Deserialize)]
pub struct AuthCookie {
    pub user_id: i64,
    pub session_id: String
}


#[derive(Serialize, Deserialize)]
pub struct LoginEvent {
    pub email: String,
    pub password: String
}

#[derive(Serialize, Deserialize)]
pub struct SignUpEvent {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct ClientEvent {
    #[serde(flatten)]
    pub data: MessageTypes,
    pub client_id: usize,
    pub room: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password: String,
    pub profile: Option<String>,
    #[serde(with = "format::date_format")]
    pub created_at: Option<NaiveDateTime>,
    pub description: Option<String>,
    pub allow_login: bool,
    pub is_online: bool,
    pub is_staff: bool,
    pub is_superuser: bool
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserSession {
    #[serde(with = "format::uid_format")]
    pub session_id: Uuid,
    pub userid: i64,
    #[serde(with = "format::date_format")]
    pub last_login: Option<NaiveDateTime>
}

#[derive(Serialize, Deserialize)]
pub struct Guild {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    #[serde(with = "format::dt_format")]
    pub created_at: Option<NaiveDateTime>,
    pub creator_id: i64
}

#[derive(Serialize, Deserialize)]
pub struct ReadyEvent {
    pub user: User,
    pub guilds: Vec<Guild>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MessageCreateType {
    pub content: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MessageUpateType {
    pub id: usize,
    pub content: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum MessageTypes {
    MessageCreate(MessageCreateType),
    MessageUpate(MessageUpateType)
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
    pub fn new(visitor_count: Arc<AtomicUsize>) -> Self {
        // TODO: make visitor count actually work
        let mut rooms = HashMap::new();
        rooms.insert("Main".to_owned(), HashSet::new());
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

    pub async fn join_room(&self, room: String, user_id: usize) {
        log::info!("{} id joining room {}", user_id, room);
        let mut rooms = Vec::new();
        // drop MutexGuard
        {
            let mut inner = self.inner.lock().await;

            for (n, sessions) in &mut inner.rooms {
                if sessions.remove(&user_id) {
                    rooms.push(n.to_owned());
                }
            }

            inner.rooms
                .entry(room.clone())
                .or_insert_with(HashSet::new)
                .insert(user_id);    
        }

        log::info!("ROOMS: {:?}", rooms);
        for room in rooms {
            self.send_message(&room, MessageTypes::MessageCreate(MessageCreateType{content: "Someone disconnected".to_string()})).await;
        }

        self.send_message(&room, MessageTypes::MessageCreate(MessageCreateType{content: "Someone connected".to_string()})).await;
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
        }
        drop(inner);
        let res = unordered.collect::<Vec<(usize, Vec<Result<Session, ()>>)>>().await;
        let mut inner = self.inner.lock().await;
        for (user_id, results) in res {
            inner.sessions.insert(user_id, results.into_iter().filter_map(|i| i.ok()).collect());
        }
    }

    // send a message to all the sessions for ID
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