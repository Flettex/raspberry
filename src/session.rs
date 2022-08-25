use std::{
    sync::Arc,
    time::{Duration, Instant}, collections::HashSet
};

use crate::server;
use crate::messages::{
    MessageTypes,
    MessageCreateType,
    WsReceiveTypes,
    GuildCreateType,
    ReadyEventType,
    MemberCreateType,
    MemberRemoveType,
    ChannelCreateType
};
use crate::db::{
    self,
    models
};
use actix_ws::{Session, MessageStream, Message, CloseReason};
use tokio::sync::Mutex;
use serde_json;

use std::fmt;
use futures::StreamExt;

use sqlx::{postgres::PgPool, types::Uuid};

impl fmt::Display for WsReceiveTypes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WsReceiveTypes::MessageCreate(msg) => write!(f, "{}", msg.content),
            WsReceiveTypes::MessageUpdate(msg) => write!(f, "Updating {} to {}", msg.id, msg.content),
            WsReceiveTypes::GuildCreate(guild) => write!(f, "Creating guild named {}, described: {:?}\n Icon: {:?}", guild.name, guild.desc, guild.icon),
            WsReceiveTypes::ChannelCreate(chan) => write!(f, "Creating channel named {}, described: {:?}, Position: {}, Guild: {} ", chan.name, chan.desc, chan.position, chan.guild_id),
            WsReceiveTypes::MemberCreate(mem) => write!(f, "new member to guild {}", mem.guild_id),
            WsReceiveTypes::Null => write!(f, "{}", "null"),
        }
    }
}

pub struct WsChatSession {
    pub id: Arc<Mutex<usize>>,

    pub rooms: Arc<Mutex<HashSet<String>>>,

    pub name: Arc<Mutex<Option<String>>>,

    pub alive: Arc<Mutex<Instant>>,

    pub stream: Arc<Mutex<MessageStream>>,

// below should not be mutated at all

    pub srv: server::Chat,

    pub pool: PgPool,

    pub session_id: String,

    pub session: Session,
}

impl WsChatSession {
    pub fn decode_json(&self, s: &str) -> serde_json::Result<WsReceiveTypes> {
        serde_json::from_str(s)
    }

    // updated to MessageCreate only because no other events are sent anyways
    pub async fn send_to_all_rooms(&self, msg: String) {
        for room in &*self.rooms.lock().await {
            self.srv.send_message(&room, MessageTypes::MessageCreate(MessageCreateType{content: msg.clone(), room: room.to_string()})).await;
        }
    }

    pub async fn send_event(&self, msg: MessageTypes) {
        // println!("{}", serde_json::to_string(&msg).unwrap_or("Something failed idk".to_string()));
        self.session.clone().text(serde_json::to_string(&msg).unwrap()).await.unwrap();
    }

    pub async fn hb(&self) {
        // spawn this, not await this
        let mut session = self.session.clone();
        let mut interval = actix_web::rt::time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            if session.ping(b"").await.is_err() {
                break;
            }
            if Instant::now().duration_since(*self.alive.lock().await) > Duration::from_secs(10) {
                // disconnect
                log::info!("L imagine running out of internet");
                self.disconnect(None).await;
                break;
            }
        }
    }

    pub async fn disconnect(&self, reason: Option<CloseReason>) {
        let usr_id = self.id.lock().await;
        let id = usr_id.clone();
        // drop the STUPID mutex guard smh
        drop(usr_id);
        let session = self.session.clone();
        // idk if closing session here is a good idea but eh
        let _ = session.close(reason).await;
        db::ws_session::toggle_user_status(id.try_into().unwrap(), false, &self.pool).await.unwrap();
        self.send_to_all_rooms("Someone disconnected".to_string()).await;
        for room in &*self.rooms.lock().await {
            self.srv.leave_room(room.to_string(), id).await;
        }
    }

    pub async fn start(&self) {
        // connect
        // join user to room Main
        let usr_id = self.id.lock().await;
        let mut id = usr_id.clone();
        // hahaha drop immediately after use because I'm scared of deadlocks.
        drop(usr_id);
        self.srv.insert_id("Main".to_string(), id).await;
        // add visitor count
        let count = self.srv.new_visitor().await;
        let mut stream = self.stream.lock().await;
        let mut session = self.session.clone();
        // READY event
        println!("Session_id: {}", self.session_id.clone());
        let user: models::User = match db::ws_session::get_user_by_session_id(self.session_id.clone(), &self.pool).await {
            Ok(usr) => usr,
            Err(_err) => {
                println!("{:?}", _err);
                let _ = session.close(None).await;
                return
            }
        };
        db::ws_session::toggle_user_status(user.id, true, &self.pool).await.unwrap();
        db::ws_session::update_user_last_login(Uuid::parse_str(&self.session_id.clone()).unwrap(), &self.pool).await.unwrap();
        // idek why I am doing this, this makes sure the user is not impersonating or editing the cookie in some way
        if id != user.id as usize {
            id = user.id as usize
        }
        *self.name.lock().await = Some(user.username.clone());
        println!("CODE: {:?}", user.code);
        if user.code.is_some() {
            self.send_event(MessageTypes::MessageCreate(MessageCreateType{content: "WARNING: Your account is not verified. Please check your email and verify at /verify".to_string(), room: "Main".to_string()})).await;
        }
        let guilds: Vec<models::Guild> = match db::ws_session::get_guilds_by_user_id(id as i64, &self.pool).await {
            Ok(glds) => glds,
            Err(err) => {
                println!("{:?}", err);
                vec![]
            }
        };

        let mut guildchannels: Vec<models::GuildChannels> = vec![];
    
        // do smth about each guild the user is in
        for guild in guilds.clone() {
            let channels = db::ws_session::get_channels_by_guild_id(guild.id, &self.pool).await.unwrap();
            guildchannels.push(models::GuildChannels {
                id: guild.id,
                name: guild.name.to_owned(),
                description: guild.description,
                icon: guild.icon,
                creator_id: guild.creator_id,
                created_at: guild.created_at,
                channels: channels.to_owned()
            });
            for channel in channels.to_owned() {
                self.rooms.lock().await.insert(channel.id.to_string());
                self.srv.join_room(channel.id.to_string(), id).await;
            }
        }
    
        // ready event
        self.send_event(MessageTypes::ReadyEvent(ReadyEventType{user: user.clone().into(), guilds: guildchannels})).await;
        self.send_event(MessageTypes::MessageCreate(MessageCreateType{content: format!("Ready! Total visitors {}. User: {}", count, serde_json::to_string(&user.clone()).unwrap()), room: "Main".to_string()})).await;
        
        self.srv.send_message("Main", MessageTypes::MessageCreate(MessageCreateType{content: "Someone connected".to_string(), room: "Main".to_string()})).await;
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
                    // self.srv.send(MessageTypes::MessageCreate(MessageCreateType{content: s.into()})).await;
                    // self.srv.send_message("Main", MessageTypes::MessageCreate(MessageCreateType{content: s.into()})).await;
                    let val = match self.decode_json(s.trim()) {
                        Err(err) => {
                            println!("{}", err);
                            WsReceiveTypes::Null
                        }
                        Ok(val) => val,
                    };
                    // println!("{}", val);
                    match val {
                        WsReceiveTypes::MessageCreate(m) => {
                            if m.content.starts_with('/') {
                                let v: Vec<&str> = m.content.splitn(2, ' ').collect();
                                match v[0] {
                                    "/list" => {
                                        println!("List rooms");
                                        let rooms = self.srv.list_rooms().await;
                                        self.srv.send_message(&m.room, MessageTypes::MessageCreate(MessageCreateType{content: rooms.join(", "), room: m.room.clone()})).await;
                                    }
                                    "/join" => {
                                        if v.len() == 2 {
                                            log::info!("{:?} joining {}", *self.name.lock().await, v[1].to_owned());
                                            self.rooms.lock().await.insert(v[1].to_owned());
                                            self.send_event(MessageTypes::MemberCreate(MemberCreateType { id, room: v[1].to_owned() })).await;
                                            self.srv.join_room(v[1].to_owned(), id).await;
                                            self.srv.send_message(&v[1].to_owned(), MessageTypes::MessageCreate(MessageCreateType {content: "joined".to_string(), room: v[1].to_owned()})).await;
                                        } else {
                                            self.srv.send_message(&m.room, MessageTypes::MessageCreate(MessageCreateType {content: "!!! room name is required".to_string(), room: m.room.clone()})).await;
                                        }
                                    }
                                    "/leave" => {
                                        if v.len() == 2 {
                                            if v[1] == "Main" {
                                                self.send_event(MessageTypes::MessageCreate(MessageCreateType { content: "you can't leave Main dumbass".to_string(), room: "Main".to_string()})).await;
                                                continue;
                                            }
                                            log::info!("{:?} leaving {}", *self.name.lock().await, v[1].to_owned());
                                            let mut rooms = self.rooms.lock().await;
                                            rooms.remove(v[1]);
                                            self.send_event(MessageTypes::MemberRemove(MemberRemoveType { id, room: v[1].to_owned() })).await;
                                            self.srv.leave_room(v[1].to_owned(), id).await;
                                            self.srv.send_message(&v[1].to_owned(), MessageTypes::MessageCreate(MessageCreateType {content: "left".to_string(), room: v[1].to_owned()})).await;
                                        } else {
                                            self.srv.send_message(&m.room, MessageTypes::MessageCreate(MessageCreateType {content: "!!! room name is required".to_string(), room: m.room.clone()})).await;
                                        }
                                    }
                                    "/name" => {
                                        if v.len() == 2 {
                                            // change your nickname (globally) idk why I have this
                                            *self.name.lock().await = Some(v[1].to_owned());
                                        } else {
                                            self.srv.send_message(&m.room, MessageTypes::MessageCreate(MessageCreateType {content: "!!! name is required".to_string(), room: m.room.clone()})).await;
                                        }
                                    }
                                    _ => self.srv.send_message(&m.room, MessageTypes::MessageCreate(MessageCreateType {content: format!("!!! unknown command {:?}", m.content), room: m.room.clone()})).await,
                                }
                                continue;
                            }
                            if !self.rooms.lock().await.contains(&m.room) {
                                // bro's trying to send message to a room they don't have access to
                                continue;
                            }
                            let msg = if let Some(ref name) = *self.name.lock().await {
                                format!("{}: {}", name, m.content)
                            } else {
                                m.content.to_owned()
                            };
                            log::info!("{} {}", msg, id);
                            self.srv.send_message(&m.room, MessageTypes::MessageCreate(MessageCreateType {content: msg.to_string(), room: m.room.clone()})).await
                        }
                        WsReceiveTypes::MessageUpdate(_) => {
                            // update msg
                            // bro can't implement since MessageCreate is not even working yet
                        }
                        WsReceiveTypes::MemberCreate(mem) => {
                            match db::ws_session::join_guild(id as i64, mem.guild_id, &self.pool).await {
                                Ok(channels) => {
                                    for c in channels {
                                        self.rooms.lock().await.insert(c.id.to_string());
                                        self.send_event(MessageTypes::GuildCreate(GuildCreateType {guild: db::ws_session::get_guild_by_id(mem.guild_id, &self.pool).await.unwrap()})).await;
                                        self.send_event(MessageTypes::ChannelCreate(ChannelCreateType {channel: c.to_owned()})).await;
                                    }
                                }
                                Err(err) => {
                                    println!("{:?}", err);
                                }
                            }
                        }
                        WsReceiveTypes::GuildCreate(guild) => {
                            match db::ws_session::create_guild(id as i64, guild, &self.pool).await {
                                Ok(rec) => {
                                    // self.rooms.lock().await.insert(rec.name.to_owned());
                                    /* Very Broken right now, waiting for a fix */
                                    self.send_event(MessageTypes::MemberCreate(MemberCreateType { id, room: rec.id.to_string() })).await;
                                    // self.srv.join_room(rec.name.to_owned(), id).await;
                                    self.send_event(MessageTypes::GuildCreate(GuildCreateType {guild: rec.to_owned()})).await;
                                    // guild create no longer have these
                                    // self.srv.send_message(&rec.id.to_owned(), MessageTypes::MessageCreate(MessageCreateType {content: "joined".to_string(), room: rec.id.to_owned()})).await;
                                }
                                Err(err) => {
                                    println!("{:?}", err);
                                }
                            }
                        }
                        WsReceiveTypes::ChannelCreate(channel) => {
                            match db::ws_session::create_channel(channel, &self.pool).await {
                                Ok(rec) => {
                                    self.rooms.lock().await.insert(rec.id.to_string());
                                    self.srv.join_room(rec.id.to_string(), id).await;
                                    self.send_event(MessageTypes::ChannelCreate(ChannelCreateType {channel: rec.to_owned()})).await;
                                }
                                Err(err) => {
                                    println!("{:?}", err);
                                }
                            }
                        }
                        WsReceiveTypes::Null => {
                            // migrating AWAY from raw messages
                            // if s.starts_with('/') {
                            //     let v: Vec<&str> = s.splitn(2, ' ').collect();
                            //     match v[0] {
                            //         "/list" => {
                            //             println!("List rooms");
                            //             let rooms = self.srv.list_rooms().await;
                            //             self.send_to_all_rooms(rooms.join(", ")).await;
                            //         }
                            //         "/join" => {
                            //             if v.len() == 2 {
                            //                 log::info!("{:?} joining {}", *self.name.lock().await, v[1].to_owned());
                            //                 self.rooms.lock().await.push(v[1].to_owned());
                            //                 self.send_event(MessageTypes::MemberCreate(MemberCreateType { id: *self.id.lock().await, room: v[1].to_owned() })).await;
                            //                 self.srv.join_room(v[1].to_owned(), *id).await;
                            //                 self.srv.send_message(&v[1].to_owned(), MessageTypes::MessageCreate(MessageCreateType {content: "joined".to_string(), room: v[1].to_owned()})).await;
                            //             } else {
                            //                 self.send_to_all_rooms("!!! room name is required".to_string()).await;
                            //             }
                            //         }
                            //         "/name" => {
                            //             if v.len() == 2 {
                            //                 *self.name.lock().await = Some(v[1].to_owned());
                            //             } else {
                            //                 self.send_to_all_rooms(MessageTypes::MessageCreate(MessageCreateType {content: "!!! name is required".to_string()})).await;
                            //             }
                            //         }
                            //         _ => self.send_to_all_rooms(MessageTypes::MessageCreate(MessageCreateType {content: format!("!!! unknown command {:?}", s)})).await,
                            //     }
                            // } else {
                            //     let msg = if let Some(ref name) = *self.name.lock().await {
                            //         format!("{}: {}", name, s)
                            //     } else {
                            //         s.to_owned()
                            //     };
                            //     log::info!("SENDING RAW MESSAGE: {} {}", msg, *id);
                            //     self.send_to_all_rooms(MessageTypes::MessageCreate(MessageCreateType {content: msg})).await
                            // }
                        }
                    }
                }
                Message::Binary(_) => println!("Unexpected binary"),
                Message::Close(reason) => {
                    self.disconnect(reason).await;
                    log::info!("Got close, bailing");
                    return;
                }
                Message::Continuation(_) => {
                    self.disconnect(None).await;
                    log::info!("Got continuation, bailing");
                    return;
                }
                Message::Nop => (),
            }
        }
        // End of buffer for no reason?!??!
        self.disconnect(None).await;
    }
}