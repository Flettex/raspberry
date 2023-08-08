use std::{
    collections::HashSet,
    sync::Arc,
    time::{Duration, Instant},
};

use crate::db::{self, models};
use crate::messages::{Handler, Message as Msg, MessageTypes, ReadyEventType, WsReceiveTypes};
use crate::{controllers::ws::WsMsgType, server, PLACEHOLDER_UUID};
use actix_ws::{CloseReason, Message, MessageStream, Session};
// use serde_json;
use tokio::sync::Mutex;

use futures::StreamExt;
use std::fmt;

use sqlx::postgres::PgPool;
use sqlx::types::Uuid;

impl fmt::Display for WsReceiveTypes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WsReceiveTypes::MessageCreate(msg) => write!(f, "{}", msg.content),
            WsReceiveTypes::MessageUpdate(msg) => {
                write!(f, "Updating {} to {}", msg.id, msg.content)
            }
            WsReceiveTypes::GuildCreate(guild) => write!(
                f,
                "Creating guild named {}, described: {:?}\n Icon: {:?}",
                guild.name, guild.desc, guild.icon
            ),
            WsReceiveTypes::ChannelCreate(chan) => write!(
                f,
                "Creating channel named {}, described: {:?}, Position: {}, Guild: {} ",
                chan.name, chan.desc, chan.position, chan.guild_id
            ),
            WsReceiveTypes::MemberCreate(mem) => write!(f, "new member to guild {}", mem.guild_id),
            WsReceiveTypes::MessageFetch(m) => {
                write!(f, "Fetching message from channel_id {}", m.channel_id)
            }
            WsReceiveTypes::MemberFetch(m) => {
                write!(f, "Fetching member from guild_id {}", m.guild_id)
            }
            WsReceiveTypes::UserFetch(u) => write!(f, "Fetching user {}", u.id),
            _ => write!(f, "Unimplemented"),
        }
    }
}

#[derive(Clone)]
pub struct WsChatSession {
    // name and id fields are replaced by user model from the database.
    pub user: models::User,

    pub rooms: Arc<Mutex<HashSet<String>>>,

    pub alive: Arc<Mutex<Instant>>,

    // stream does not satisfy traits, and is being passed in as a paramter instead.
    // pub stream: Arc<Mutex<MessageStream>>,

    // below should not be mutated at all
    pub srv: server::Chat,

    pub pool: PgPool,

    pub session_id: String,

    pub session: Session,

    pub recv_type: WsMsgType,
}

impl WsChatSession {
    pub fn decode_json(&self, s: &str) -> serde_json::Result<WsReceiveTypes> {
        serde_json::from_str(s)
    }

    // updated to MessageCreate only because no other events are sent anyways
    // pub async fn send_to_all_rooms(&self, mut msg: Msg) {
    //     for room in &*self.rooms.lock().await {
    //         msg.channel_id = Uuid::parse_str(PLACEHOLDER_UUID).unwrap();
    //         self.srv
    //             .send_guild_message(&room, MessageTypes::MessageCreate(msg.to_owned()))
    //             .await;
    //     }
    // }

    pub async fn send_event(&self, msg: MessageTypes) {
        // println!("{}", serde_json::to_string(&msg).unwrap_or("Something failed idk".to_string()));
        match self.recv_type {
            WsMsgType::Json => self
                .session
                .clone()
                .text(serde_json::to_string(&msg).unwrap())
                .await
                .unwrap(),
            WsMsgType::Cbor => self
                .session
                .clone()
                .binary(serde_cbor::to_vec(&msg).unwrap())
                .await
                .unwrap(),
        }
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
        let session = self.session.clone();
        // idk if closing session here is a good idea but eh
        let _ = session.close(reason).await;
        db::ws_session::toggle_user_status(self.user.id, false, &self.pool)
            .await
            .unwrap();
        // self.send_to_all_rooms(Msg::system(
        //     format!("User {} disconneced", self.user.id),
        //     PLACEHOLDER_UUID,
        //     self.user.id,
        // ))
        // .await;
        for room in &*self.rooms.lock().await {
            self.srv
                .leave_guild(room.to_string(), self.user.id as usize)
                .await;
        }
    }

    pub async fn start(&self, mut stream: MessageStream) {
        // connect
        // join user to guild Main
        self.srv
            .insert_id(PLACEHOLDER_UUID.to_string(), self.user.id as usize)
            .await;
        // add visitor count, very useless so removing soon!
        let count = self.srv.new_visitor().await;
        // let mut stream = self.stream.lock().await;
        let mut session = self.session.clone();
        println!("Session_id: {}", self.session_id.clone());
        // let user: models::User = match db::ws_session::get_user_by_session_id(self.session_id.clone(), &self.pool).await {
        //     Ok(usr) => usr,
        //     Err(_err) => {
        //         println!("{:?}", _err);
        //         let _ = session.close(None).await;
        //         return
        //     }
        // };
        db::ws_session::toggle_user_status(self.user.id, true, &self.pool)
            .await
            .unwrap();
        db::ws_session::update_user_last_login(
            Uuid::parse_str(&self.session_id.clone()).unwrap(),
            &self.pool,
        )
        .await
        .unwrap();
        println!("CODE: {:?}", self.user.code);
        if self.user.code.is_some() {
            self.send_event(MessageTypes::MessageCreate(Msg::system("WARNING: Your account is not verified. Please check your email and verify at /verify".to_string(), PLACEHOLDER_UUID, 0))).await;
        }
        let guilds: Vec<models::Guild> =
            match db::ws_session::get_guilds_by_user_id(self.user.id, &self.pool).await {
                Ok(glds) => glds,
                Err(err) => {
                    println!("{:?}", err);
                    vec![]
                }
            };

        let mut guildchannels: Vec<models::GuildChannels> = vec![];

        // do smth about each guild the user is in, should occupy rooms for the entire time because
        // we should not be reading the rooms while it is being initialized
        let mut rooms = self.rooms.lock().await;

        for guild in guilds.clone() {
            // about the permissions part...
            self.srv
                .join_guild(guild.id.to_string(), self.user.id as usize)
                .await;
            rooms.insert(guild.id.to_string());
            let channels = db::ws_session::get_channels_by_guild_id(guild.id, &self.pool)
                .await
                .unwrap();
            guildchannels.push(models::GuildChannels {
                id: guild.id,
                name: guild.name.to_owned(),
                description: guild.description,
                icon: guild.icon,
                creator_id: guild.creator_id,
                created_at: guild.created_at,
                channels: channels.to_owned(),
            });
            // no longer joining any of the channel
            // for channel in channels.to_owned() {
                // rooms.insert(channel.id.to_string());
                // self.srv
                //     .join_room(channel.id.to_string(), self.user.id as usize)
                //     .await;
            // }
        }

        // Drop mutex guard after
        drop(rooms);

        // ready event
        self.send_event(MessageTypes::ReadyEvent(ReadyEventType {
            user: self.user.clone().into(),
            guilds: guildchannels,
        }))
        .await;
        self.send_event(MessageTypes::MessageCreate(Msg::system(
            format!(
                "Ready! Total visitors {}. User: {}",
                count,
                serde_json::to_string(&models::UserClient::from(self.user.clone())).unwrap()
            ),
            PLACEHOLDER_UUID,
            0,
        )))
        .await;

        self.srv
            .send_guild_message(
                PLACEHOLDER_UUID,
                MessageTypes::MessageCreate(Msg::system(
                    "Someone connected".to_string(),
                    PLACEHOLDER_UUID,
                    0,
                )),
            )
            .await;
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
                    let val: Option<WsReceiveTypes> = self.decode_json(s.trim()).ok();
                    /* Starting from binary update, text events will be deprecated */
                    if let Some(val) = val {
                        println!("{}", val);
                        val.handle(self.to_owned()).await;
                    }
                }
                Message::Binary(b) => {
                    // println!("{}", serde_cbor::from_slice(b.as_ref()).unwrap());
                    println!("{:?}", serde_cbor::from_slice::<WsReceiveTypes>(b.as_ref()));
                    let val: Option<WsReceiveTypes> = serde_cbor::from_slice(b.as_ref()).ok();
                    if let Some(val) = val {
                        println!("{}", val);
                        val.handle(self.to_owned()).await;
                    }
                }
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
