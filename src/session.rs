use std::{
    sync::Arc,
    time::{Duration, Instant}
};

use crate::server::{
    self,
    MessageTypes,
    MessageCreateType
};
use actix_rt;
use actix_ws::{Session, MessageStream, Message};
use tokio::sync::Mutex;

use serde::{Serialize, Deserialize};
// use serde_json::{json};
use std::fmt;
use futures::StreamExt;

use sqlx::postgres::PgPool;

#[derive(Serialize, Deserialize, Clone)]
pub struct WsMessageCreate {
    pub content: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WsMessageUpdate {
    pub id: usize,
    pub content: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WsDevice {
    pub os: String,
    pub device: String,
    pub browser: String
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum WsReceiveTypes {
    // {"type":"MessageUpdate", "data":{"content":"",id:1}}
    MessageUpdate(WsMessageUpdate),
    // {"type":"MessageCreate", "data":{"content":""}}
    MessageCreate(WsMessageCreate),
    // {"type":"Null"}
    // used for testing purposes
    Null
}

impl fmt::Display for WsReceiveTypes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WsReceiveTypes::MessageCreate(msg) => write!(f, "{}", msg.content),
            WsReceiveTypes::MessageUpdate(msg) => write!(f, "Updating {} to {}", msg.id, msg.content),
            WsReceiveTypes::Null => write!(f, "{}", "null"),
        }
    }
}

pub struct WsChatSession {
    pub id: usize,

    pub room: String,

    pub name: Option<String>,

    pub srv: server::Chat,

    pub pool: PgPool,

    pub session_id: String,

    pub session: Session,

    pub alive: Arc<Mutex<Instant>>,

    pub stream: Arc<Mutex<MessageStream>>
}

impl WsChatSession {
    pub async fn hb(&self) {
        let mut session = self.session.clone();
        let mut interval = actix_rt::time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            if session.ping(b"").await.is_err() {
                break;
            }
            if Instant::now().duration_since(*self.alive.lock().await) > Duration::from_secs(10) {
                // disconnect
                let _ = session.close(None).await;
                break;
            }
        }
    }

    pub async fn start(&self) {
        // connect
        // self.srv.inner.rooms
        //     .entry("Main".to_owned())
        //     .or_insert_with(HashSet::new)
        //     .insert(id);
        let mut stream = self.stream.lock().await;
        let mut session = self.session.clone();
        while let Some(Ok(msg)) = stream.next().await {
            log::debug!("WEBSOCKET MESSAGE: {:?}", msg);
            match msg {
                Message::Ping(bytes) => {
                    if session.pong(&bytes).await.is_err() {
                        return;
                    }
                }
                Message::Pong(_) => {
                    *self.alive.lock().await = Instant::now();
                }
                Message::Text(s) => {
                    log::info!("Relaying text, {}", s);
                    let s: &str = s.as_ref();
                    self.srv.send(MessageTypes::MessageCreate(MessageCreateType{content: s.into()})).await;
                    self.srv.send_message("Main", MessageTypes::MessageCreate(MessageCreateType{content: s.into()})).await;
                    let val = match self.decode_json(s.trim()) {
                        Err(err) => {
                            println!("{}", err);
                            WsReceiveTypes::Null
                        }
                        Ok(val) => val,
                    };
                    println!("{}", val);
                    match val {
                        WsReceiveTypes::MessageCreate(m) => {
                            let msg = if let Some(ref name) = self.name {
                                format!("{}: {}", name, m.content)
                            } else {
                                m.content.to_owned()
                            };
                            log::info!("{} {}", msg, self.id);
                            self.srv.send_message(&self.room, MessageTypes::MessageCreate(MessageCreateType {content: msg.to_string()})).await
                        }
                        WsReceiveTypes::MessageUpdate(_) => {
                            // update msg
                        }
                        WsReceiveTypes::Null => {
                            if s.starts_with('/') {
                                let v: Vec<&str> = s.splitn(2, ' ').collect();
                                match v[0] {
                                    "/list" => {
                                        println!("List rooms");
                                        // self.srv
                                        //     .send(server::ListRooms)
                                        //     .into_actor(self)
                                        //     .then(|res, _, ctx| {
                                        //         match res {
                                        //             Ok(rooms) => {
                                        //                 for room in rooms {
                                        //                     ctx.text(format_m(&room));
                                        //                 }
                                        //             }
                                        //             _ => println!("Something is wrong"),
                                        //         }
                                        //         fut::ready(())
                                        //     })
                                        //     .wait(ctx)
                                    }
                                    "/join" => {
                                        if v.len() == 2 {
                                            // *self.room = v[1].to_owned();
                                            self.srv.send_message(&self.room, MessageTypes::MessageCreate(MessageCreateType {content: "joined".to_string()})).await;
                                        } else {
                                            self.srv.send_message(&self.room, MessageTypes::MessageCreate(MessageCreateType {content: "!!! room name is required".to_string()})).await;
                                        }
                                    }
                                    "/name" => {
                                        if v.len() == 2 {
                                            // *self.name = Some(v[1].to_owned());
                                        } else {
                                            self.srv.send_message(&self.room, MessageTypes::MessageCreate(MessageCreateType {content: "!!! name is required".to_string()})).await;
                                        }
                                    }
                                    _ => self.srv.send_message(&self.room, MessageTypes::MessageCreate(MessageCreateType {content: format!("!!! unknown command {:?}", s)})).await,
                                }
                            } else {
                                let msg = if let Some(ref name) = self.name {
                                    format!("{}: {}", name, s)
                                } else {
                                    s.to_owned()
                                };
                                log::info!("{} {}", msg, self.id);
                                self.srv.send_message(&self.room, MessageTypes::MessageCreate(MessageCreateType {content: msg})).await
                            }
                        }
                    }
                }
                Message::Binary(_) => println!("Unexpected binary"),
                Message::Close(reason) => {
                    let _ = session.close(reason).await;
                    log::info!("Got close, bailing");
                    return;
                }
                Message::Continuation(_) => {
                    let _ = session.close(None).await;
                    log::info!("Got continuation, bailing");
                    return;
                }
                Message::Nop => (),
            }
            // match msg {
            //     Message::Ping(bytes) => {
            //         if session.pong(&bytes).await.is_err() {
            //             return;
            //         }
            //     }
            //     Message::Text(s) => {
            //         log::info!("Relaying text, {}", s);
            //         let s: &str = s.as_ref();
            //         self.srv.send(MessageTypes::MessageCreate(MessageCreateType{content: s.into()})).await;
            //     }
            //     Message::Close(reason) => {
            //         let _ = session.close(reason).await;
            //         log::info!("Got close, bailing");
            //         return;
            //     }
            //     Message::Continuation(_) => {
            //         let _ = session.close(None).await;
            //         log::info!("Got continuation, bailing");
            //         return;
            //     }
            //     Message::Pong(_) => {
            //         *self.alive.lock().await = Instant::now();
            //     }
            //     Message::Binary(_) => println!("Unexpected binary"),
            //     _ => (),
            // };
        }
        let _ = session.close(None).await;
    }

    pub fn decode_json(&self, s: &str) -> serde_json::Result<WsReceiveTypes> {
        serde_json::from_str(s)
    }
}