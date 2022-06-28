use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    clone::Clone,
};

use sqlx::types::chrono::{
    NaiveDateTime,
    DateTime
};

use serde::{Serialize, Deserialize};
use serde_json;
use rand::{self, rngs::ThreadRng, Rng};

#[derive(Message)]
#[rtype(result = "()")]
pub struct Message {
    pub data: MessageTypes
}

#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<Message>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
}

mod date_format {
    use crate::_server::{
        NaiveDateTime,
        DateTime
    };
    use serde::{self, Deserialize, Deserializer, Serializer, de::Error};
    const FORMAT: &str = "%+";

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveDateTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let time: String = Deserialize::deserialize(deserializer)?;
		Ok(Some(DateTime::parse_from_rfc3339(&time).map_err(D::Error::custom)?.naive_utc()))
    }

    pub fn serialize<S>(
        date: &Option<NaiveDateTime>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(date) = date {
            let s = format!("{}", date.format(FORMAT));
            serializer.serialize_str(&s)
        } else {
            serializer.serialize_str("")
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password: String,
    pub profile: Option<String>,
    #[serde(with = "date_format")]
    pub created_at: Option<NaiveDateTime>,
    pub description: Option<String>,
    pub allow_login: bool,
    pub is_online: bool,
    pub is_staff: bool,
    pub is_superuser: bool
}

#[derive(Serialize, Deserialize)]
pub struct Guild {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    #[serde(with = "date_format")]
    pub created_at: Option<NaiveDateTime>,
    pub creator_id: i64
}

#[derive(Serialize, Deserialize, Message)]
#[rtype(result = "()")]
pub struct ReadyEvent {
    pub user: User,
    pub guilds: Vec<Guild>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MessageCreateType {
    content: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MessageUpateType {
    id: usize,
    content: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum MessageTypes {
    MessageCreate(MessageCreateType),
    MessageUpate(MessageUpateType)
}

#[derive(Message, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct ClientEvent {
    #[serde(flatten)]
    pub data: MessageTypes,
    pub client_id: usize,
    pub room: String,
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

#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    pub id: usize,
    pub msg: String,
    pub room: String,
}

pub struct ListRooms;

impl actix::Message for ListRooms {
    type Result = Vec<String>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Join {
    pub id: usize,
    pub name: String,
}

#[derive(Debug)]
pub struct ChatServer {
    sessions: HashMap<usize, Recipient<Message>>,
    rooms: HashMap<String, HashSet<usize>>,
    rng: ThreadRng,
    visitor_count: Arc<AtomicUsize>,
}

impl ChatServer {
    pub fn new(visitor_count: Arc<AtomicUsize>) -> ChatServer {
        let mut rooms = HashMap::new();
        // global chat ig
        rooms.insert("Main".to_owned(), HashSet::new());

        ChatServer {
            sessions: HashMap::new(),
            rooms,
            rng: rand::thread_rng(),
            visitor_count,
        }
    }

    fn send_message(&self, room: &str, message: &str, skip_id: usize) {
        if let Some(sessions) = self.rooms.get(room) {
            for id in sessions {
                if *id != skip_id {
                    if let Some(addr) = self.sessions.get(id) {
                        addr.do_send(Message{data: MessageTypes::MessageCreate(MessageCreateType{content: message.to_owned()})});
                    }
                }
            }
        }
    }

    fn send_event(&self, room: &str, data: MessageTypes) {
        if let Some(sessions) = self.rooms.get(room) {
            for id in sessions {
                if let Some(addr) = self.sessions.get(id) {
                    addr.do_send(Message{data: data.clone()});
                }
            }
        }
    }
}

impl Actor for ChatServer {
    type Context = Context<Self>;
}

impl Handler<ReadyEvent> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: ReadyEvent, _: &mut Context<Self>) -> Self::Result {
        println!("Ready Event");

        self.send_message("Main", &serde_json::to_string(&msg).unwrap(), 0);
    }
}

impl Handler<Connect> for ChatServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        println!("Someone joined");

        self.send_message("Main", "Someone joined", 0);

        let id = self.rng.gen::<usize>();
        self.sessions.insert(id, msg.addr);

        self.rooms
            .entry("Main".to_owned())
            .or_insert_with(HashSet::new)
            .insert(id);

        let count = self.visitor_count.fetch_add(1, Ordering::SeqCst);
        self.send_message("Main", &format!("Total visitors {}", count), 0);

        id
    }
}

impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        println!("Someone disconnected");

        let mut rooms: Vec<String> = Vec::new();

        if self.sessions.remove(&msg.id).is_some() {
            for (name, sessions) in &mut self.rooms {
                if sessions.remove(&msg.id) {
                    rooms.push(name.to_owned());
                }
            }
        }
        for room in rooms {
            self.send_message(&room, "Someone disconnected", 0);
        }
    }
}

impl Handler<ClientMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        self.send_message(&msg.room, msg.msg.as_str(), msg.id);
    }
}

impl Handler<ClientEvent> for ChatServer {
    type Result = ();

    fn handle(&mut self, ev: ClientEvent, _: &mut Context<Self>) {
        self.send_event(&ev.room, ev.data);
    }
}

impl Handler<ListRooms> for ChatServer {
    type Result = MessageResult<ListRooms>;

    fn handle(&mut self, _: ListRooms, _: &mut Context<Self>) -> Self::Result {
        let mut rooms = Vec::new();

        for key in self.rooms.keys() {
            rooms.push(key.to_owned())
        }

        MessageResult(rooms)
    }
}

impl Handler<Join> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Join, _: &mut Context<Self>) {
        let Join { id, name } = msg;
        let mut rooms = Vec::new();

        for (n, sessions) in &mut self.rooms {
            if sessions.remove(&id) {
                rooms.push(n.to_owned());
            }
        }

        for room in rooms {
            self.send_message(&room, "Someone disconnected", 0);
        }

        self.rooms
            .entry(name.clone())
            .or_insert_with(HashSet::new)
            .insert(id);

        self.send_message(&name, "Someone connected", id);
    }
}