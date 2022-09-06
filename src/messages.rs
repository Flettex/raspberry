use chrono::{NaiveDateTime, Utc, NaiveDate};
use serde::{self, Serialize, Deserialize};
// use sqlx::types::Uuid;
use std::clone::Clone;
use sqlx::types::Uuid;
use crate::db::models::{
    self,
    Guild,
    UserClient,
    GuildChannels,
    Channel,
    Member,
    User
};
use crate::format;

#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
    pub id: Uuid,
    pub content: String,
    #[serde(with = "format::date_format2")]
    pub created_at: NaiveDateTime,
    #[serde(with = "format::date_format2")]
    pub edited_at: NaiveDateTime,
    pub author: UserFetchType,
    pub channel_id: Uuid,
    pub nonce: Uuid,
}

impl Message {
    pub fn system(content: String, channel_id: &str, author_id: i64) -> Self {
        Self {
            id: Uuid::new_v4(),
            content,
            channel_id: Uuid::parse_str(channel_id).unwrap(),
            author: UserFetchType{
                id: author_id,
                username: "System".to_string(),
                profile: None,
                description: None,
                created_at: NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
                is_staff: true,
                is_superuser: true
            },
            edited_at: Utc::now().naive_utc(),
            created_at: Utc::now().naive_utc(),
            nonce: Uuid::new_v4()
        }
    }

    pub fn user(content: String, channel_id: &str, author: UserFetchType, nonce: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            content,
            channel_id: Uuid::parse_str(channel_id).unwrap(),
            author,
            edited_at: Utc::now().naive_utc(),
            created_at: Utc::now().naive_utc(),
            nonce
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ReadyEventType {
    pub user: UserClient,
    pub guilds: Vec<GuildChannels>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GuildCreateType {
    pub guild: Guild
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChannelCreateType {
    pub channel: Channel
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MemberCreateType {
    // user id
    pub id: usize,
    pub guild: Guild
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MemberRemoveType {
    pub id: usize,
    pub room: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MessagesType {
    pub channel_id: Uuid,
    pub messages: Vec<models::Message>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MembersType {
    pub guild_id: Uuid,
    pub members: Vec<Member>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserFetchType {
    pub id: i64,
    pub username: String,
    pub profile: Option<String>,
    #[serde(with = "format::date_format2")]
    pub created_at: NaiveDateTime,
    pub description: Option<String>,
    pub is_staff: bool,
    pub is_superuser: bool,
}

impl From<User> for UserFetchType {
    fn from(u: User) -> UserFetchType {
        UserFetchType {
            id: u.id,
            username: u.username,
            profile: u.profile,
            created_at: u.created_at,
            description: u.description,
            is_staff: u.is_staff,
            is_superuser: u.is_superuser
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum MessageTypes {
    Messages(MessagesType),
    Members(MembersType),
    MessageCreate(Message),
    MessageUpdate(Message),
    ReadyEvent(ReadyEventType),
    GuildCreate(GuildCreateType),
    ChannelCreate(ChannelCreateType),
    MemberCreate(MemberCreateType),
    MemberRemove(MemberRemoveType),
    UserFetch(UserFetchType)
}

/* Ws Events */

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WsMessageCreate {
    pub content: String,
    pub channel_id: String,
    pub nonce: Uuid
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WsMessageUpdate {
    pub id: Uuid,
    pub content: String,
    pub nonce: Uuid
}

// supposed to be used for sessions, but not used atm

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WsDevice {
    pub os: String,
    pub device: String,
    pub browser: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WsGuildCreate {
    pub name: String,
    pub desc: Option<String>,
    pub icon: Option<String>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WsChannelCreate {
    pub name: String,
    pub desc: Option<String>,
    pub position: i64,
    pub guild_id: Uuid
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WsMemberCreate {
    pub guild_id: Uuid
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WsMessageFetchType {
    pub channel_id: Uuid
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WsMemberFetchType {
    pub guild_id: Uuid
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WsUserFetchType {
    pub id: usize
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", content = "data")]
pub enum WsReceiveTypes {
    // {"type":"UserFetch", "id": 0}
    UserFetch(WsUserFetchType),
    // {"type":"MessageFetch", "channel_id": "bruh-bruh-bruh-bruh"}
    MessageFetch(WsMessageFetchType),
    // {"type":"MemberFetch"}
    MemberFetch(WsMemberFetchType),
    // {"type":"MessageUpdate", "data":{"content":"",id:1}}
    MessageUpdate(WsMessageUpdate),
    // {"type":"MessageCreate", "data":{"content":"", room:""}}
    MessageCreate(WsMessageCreate),
    // {"type":"GuildCreate", "data":{"name": "breme's server"}}
    GuildCreate(WsGuildCreate),
    // {"type":"ChannelCreate", "data":{"name": "dumbdumbs", "position": 0, "guild_id": "bruh-bruh-bruh-bruh"}}
    ChannelCreate(WsChannelCreate),
    // {"type": "MemberCreate", "data":{"guild_id": "bruh-bruh-bruh-bruh"}}
    MemberCreate(WsMemberCreate),
    // {"type":"Null"}
    // used for testing purposes
    Null
}