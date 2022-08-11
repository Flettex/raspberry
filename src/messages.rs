use serde::{self, Serialize, Deserialize};
use sqlx::types::Uuid;
use std::clone::Clone;
use crate::db::models::{
    User,
    Guild
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
    pub user: User,
    pub guilds: Vec<Uuid>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GuildCreateType {
    pub guild: Guild
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
#[serde(tag = "type", content = "data")]
pub enum WsReceiveTypes {
    // {"type":"MessageUpdate", "data":{"content":"",id:1}}
    MessageUpdate(WsMessageUpdate),
    // {"type":"MessageCreate", "data":{"content":"", room:""}}
    MessageCreate(WsMessageCreate),
    // {"type":"GuildCreate", "data":{"name": "breme's server"}}
    GuildCreate(WsGuildCreate),
    // {"type":"Null"}
    // used for testing purposes
    Null
}