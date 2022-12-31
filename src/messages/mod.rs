use serde::{self, Serialize, Deserialize};
use std::clone::Clone;
mod send;
mod receive;
pub use send::*;
pub use receive::*;
use enum_dispatch::enum_dispatch;
use crate::session::WsChatSession;

#[derive(Serialize, Deserialize, Clone)]
pub struct UnauthorizedError {
    pub content: String
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum ErrorMessageTypes {
    ErrorUnauthorized(UnauthorizedError)
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

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", content = "data")]
#[enum_dispatch(Handler)]
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
}