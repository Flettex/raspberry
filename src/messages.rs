use serde::{self, Serialize, Deserialize};
// use sqlx::types::Uuid;
use std::clone::Clone;
use sqlx::types::Uuid;
use crate::db::models::{
    Guild,
    UserClient,
    GuildChannels, Channel
};

#[derive(Serialize, Deserialize, Clone)]
pub struct MessageCreateType {
    pub content: String,
    pub room: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MessageUpateType {
    pub id: usize,
    pub content: String,
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
    // which room, will be guild later.
    pub room: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MemberRemoveType {
    pub id: usize,
    pub room: String
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum MessageTypes {
    MessageCreate(MessageCreateType),
    MessageUpate(MessageUpateType),
    ReadyEvent(ReadyEventType),
    GuildCreate(GuildCreateType),
    ChannelCreate(ChannelCreateType),
    MemberCreate(MemberCreateType),
    MemberRemove(MemberRemoveType)
}

/* Ws Events */

#[derive(Serialize, Deserialize, Clone)]
pub struct WsMessageCreate {
    pub content: String,
    pub room: String,
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
pub struct WsGuildCreate {
    pub name: String,
    pub desc: Option<String>,
    pub icon: Option<String>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WsChannelCreate {
    pub name: String,
    pub desc: Option<String>,
    pub position: i32, // idk what to do with this tbh
    pub guild_id: Uuid
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WsMemberCreate {
    pub guild_id: Uuid
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum WsReceiveTypes {
    // {"type":"MessageUpdate", "data":{"content":"",id:1}}
    MessageUpdate(WsMessageUpdate),
    // {"type":"MessageCreate", "data":{"content":"", room:""}}
    MessageCreate(WsMessageCreate),
    // {"type":"GuildCreate", "data":{"name": "breme's server"}}
    GuildCreate(WsGuildCreate),
    // {"type":"ChannelCreate", "data":{"name": "dumbdumbs", "position": 0, "guild_id": "bruh-bruh-bruh-bruh-bruh-bruh"}}
    ChannelCreate(WsChannelCreate),
    // {"type": "MemberCreate", "data":{"guild_id": "bruh-bruh-bruh-bruh-bruh-bruh"}}
    MemberCreate(WsMemberCreate),
    // {"type":"Null"}
    // used for testing purposes
    Null
}