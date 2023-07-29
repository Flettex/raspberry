use serde::{self, Deserialize, Serialize};
use std::clone::Clone;
mod receive;
mod send;
use crate::session::WsChatSession;
use enum_dispatch::enum_dispatch;
pub use receive::*;
pub use send::*;
// use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct UnauthorizedError {
    pub content: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum ErrorMessageTypes {
    ErrorUnauthorized(UnauthorizedError),
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum MessageTypes {
    Messages(MessagesType),
    Members(MembersType),
    MessageCreate(Message),
    MessageUpdate(Message),
    MessageDelete(MessageDeleteType),
    ReadyEvent(ReadyEventType),
    GuildCreate(GuildCreateType),
    ChannelCreate(ChannelCreateType),
    ChannelUpdate(ChannelUpdateType),
    ChannelDelete(ChannelDeleteType),
    MemberCreate(MemberCreateType),
    MemberUpdate(MemberUpdateType),
    MemberRemove(MemberRemoveType),
    UserFetch(UserFetchType),
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
    // {"type":"Messagedelete", "id": "bruh-bruh-bruh-bruh"}
    MessageDelete(WsMessageDelete),
    // {"type":"GuildCreate", "data":{"name": "breme's server"}}
    GuildCreate(WsGuildCreate),
    // {"type":"ChannelCreate", "data":{"name": "dumbdumbs", "position": 0, "guild_id": "bruh-bruh-bruh-bruh"}}
    ChannelCreate(WsChannelCreate),
    // 
    DMChannelCreate(WsDMChannelCreate),
    // 
    ChannelUpdate(WsChannelUpdate),
    // 
    ChannelDelete(WsChannelDelete),
    // {"type": "MemberCreate", "data":{"guild_id": "bruh-bruh-bruh-bruh"}}
    MemberCreate(WsMemberCreate),
    //
    MemberUpdate(WsMemberUpdate),
}
