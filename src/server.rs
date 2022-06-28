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
    pub userid: i64
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

    pub async fn insert(&self, user_id: usize, session: Session) {
        let mut inner = self.inner.lock().await;
        let values = inner.sessions.entry(user_id).or_insert_with(|| Vec::new());
        values.push(session);
    }

    // send global.
    pub async fn send(&self, msg: MessageTypes) {
        let mut inner = self.inner.lock().await;
        let mut unordered = FuturesUnordered::new();
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
        while let Some((user_id, results)) = unordered.next().await {
            inner.sessions.insert(user_id, results.into_iter().filter_map(|i| i.ok()).collect());
        }
    }

    // can add a skip_id parameter
    pub async fn send_message(&self, room: &str, message: MessageTypes) {
        let mut inner = self.inner.lock().await;
        if let Some(users) = inner.rooms.get(room) {
            let mut unordered = FuturesUnordered::new();
            let users_cloned = users.clone();
            for (user_id, _) in inner.sessions.clone() {
                if users_cloned.contains(&user_id) {
                    let msg = message.clone();
                    if let Some(sessions) = inner.sessions.remove(&user_id) {
                        unordered.push(async move {
                            let mut results = Vec::new();
                            for mut session in sessions {
                                let res = session.text(serde_json::to_string(&msg).unwrap()).await;
                                results.push(res.map(|_| session).map_err(|_| log::info!("Dropping session")));
                            }
                            (user_id, results)
                        });
                    }
                }
            }
            while let Some((user_id, results)) = unordered.next().await {
                inner.sessions.insert(user_id, results.into_iter().filter_map(|i| i.ok()).collect());
            }
        }
    }
}