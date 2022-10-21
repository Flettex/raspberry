use serde::{self, Serialize, Deserialize};
use std::clone::Clone;
use sqlx::types::Uuid;
use async_trait::async_trait;
use crate::session::WsChatSession;
use crate::db;
use crate::PLACEHOLDER_UUID;
use super::{
    MessageTypes,
    send::*
};
use enum_dispatch::enum_dispatch;
use raspberry_macros::ratelimit;

#[async_trait]
#[enum_dispatch]
pub trait Handler {
    async fn handle(&self, ctx: WsChatSession) -> ();
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[ratelimit(1)]
pub struct WsMessageCreate {
    pub content: String,
    pub channel_id: String,
    pub nonce: Uuid
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[ratelimit(2)]
pub struct WsMessageUpdate {
    pub id: Uuid,
    pub content: String,
    pub nonce: Uuid
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[ratelimit(60)]
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

#[async_trait]
impl Handler for WsMessageCreate {
    async fn handle(&self, ctx: WsChatSession) {
        if self.content.starts_with('/') {
            let v: Vec<&str> = self.content.splitn(2, ' ').collect();
            // TODO: implement an interaction system to remove commands system...
            match v[0] {
                "/list" => {
                    println!("List rooms");
                    let rooms = ctx.srv.list_rooms().await;
                    ctx.srv.send_message(&self.channel_id, MessageTypes::MessageCreate(Message::system(rooms.join(", "), &self.channel_id.clone(), 0))).await;
                }
                "/join" => {
                    if v.len() == 2 {
                        log::info!("{:?} joining {}", ctx.user.username, v[1].to_owned());
                        ctx.rooms.lock().await.insert(v[1].to_owned());
                        // self.send_event(MessageTypes::MemberCreate(MemberCreateType { id, room: v[1].to_owned() })).await;
                        ctx.srv.join_room(v[1].to_owned(), ctx.user.id as usize).await;
                        ctx.srv.send_message(&v[1].to_owned(), MessageTypes::MessageCreate(Message::system("joined".to_string(), v[1], 0))).await;
                    } else {
                        ctx.srv.send_message(&self.channel_id, MessageTypes::MessageCreate(Message::system("!!! room name is required".to_string(), &self.channel_id.clone(), 0))).await;
                    }
                }
                "/leave" => {
                    if v.len() == 2 {
                        if v[1] == PLACEHOLDER_UUID {
                            ctx.send_event(MessageTypes::MessageCreate(Message::system("you can't leave Main dumbass".to_string(), PLACEHOLDER_UUID, 0))).await;
                            return ();
                        }
                        log::info!("{:?} leaving {}", ctx.user.username, v[1].to_owned());
                        let mut rooms = ctx.rooms.lock().await;
                        rooms.remove(v[1]);
                        // self.send_event(MessageTypes::MemberRemove(MemberRemoveType { id, room: v[1].to_owned() })).await;
                        ctx.srv.leave_room(v[1].to_owned(), ctx.user.id as usize).await;
                        ctx.srv.send_message(&v[1].to_owned(), MessageTypes::MessageCreate(Message::system("left".to_string(), v[1], 0))).await;
                    } else {
                        ctx.srv.send_message(&self.channel_id, MessageTypes::MessageCreate(Message::system("!!! room name is required".to_string(), &self.channel_id.clone(), 0))).await;
                    }
                }
                _ => ctx.srv.send_message(&self.channel_id, MessageTypes::MessageCreate(Message::system(format!("!!! unknown command {:?}", self.content), &self.channel_id.clone(), 0))).await,
            }
            return ();
        }
        if !ctx.rooms.lock().await.contains(&self.channel_id) {
            // bro's trying to send message to a room they don't have access to
            return ();
        }
        // let msg = format!("{}: {}", self.user.username, m.content);
        let msg = &self.content;
        log::info!("{} {}", msg, ctx.user.id);
        if self.channel_id == PLACEHOLDER_UUID {
            ctx.srv.send_message(&self.channel_id, MessageTypes::MessageCreate(Message::user(msg.to_string(), &self.channel_id.clone(), ctx.user.to_owned().into(), self.nonce))).await;
        } else if db::ws_session::create_message(msg.to_string(), ctx.user.id, Uuid::parse_str(&self.channel_id.clone()).unwrap(), &ctx.pool).await.is_ok() {
            ctx.srv.send_message(&self.channel_id, MessageTypes::MessageCreate(Message::user(msg.to_string(), &self.channel_id.clone(), ctx.user.to_owned().into(), self.nonce))).await;
        }
    }
}

#[async_trait]
impl Handler for WsMessageUpdate {
    async fn handle(&self, ctx: WsChatSession) {
        let updated = db::ws_session::update_message(self.id, self.content.to_owned(), &ctx.pool).await.unwrap();
        ctx.srv.send_message(&updated.channel_id.to_string(), MessageTypes::MessageUpdate(Message {
            id: updated.id,
            content: updated.content,
            created_at: updated.created_at,
            edited_at: updated.edited_at,
            author: ctx.user.to_owned().into(),
            channel_id: updated.channel_id,
            nonce: self.nonce
        })).await;
    }
}

#[async_trait]
impl Handler for WsMemberCreate {
    async fn handle(&self, ctx: WsChatSession) {
        match db::ws_session::join_guild(ctx.user.id, self.guild_id, &ctx.pool).await {
            Ok(channels) => {
                let guild = db::ws_session::get_guild_by_id(self.guild_id, &ctx.pool).await.unwrap();
                ctx.send_event(MessageTypes::GuildCreate(GuildCreateType { guild: guild.to_owned() })).await;
                for c in channels {
                    ctx.rooms.lock().await.insert(c.id.to_string());
                    ctx.srv.join_room(c.id.to_string(), ctx.user.id as usize).await;
                    ctx.send_event(MessageTypes::ChannelCreate(ChannelCreateType {channel: c.to_owned()})).await;
                    ctx.srv.send_message(&c.id.to_string(), MessageTypes::MemberCreate(MemberCreateType { id: ctx.user.id as usize, guild: guild.to_owned() })).await;
                }
            }
            Err(err) => {
                println!("{:?}", err);
            }
        }
    }
}

#[async_trait]
impl Handler for WsMemberFetchType {
    async fn handle(&self, ctx: WsChatSession) {
        if self.guild_id.to_string() == PLACEHOLDER_UUID.to_string() {
            // nobody is in Main though hmmm
            return;
        }
        ctx.send_event(MessageTypes::Members(MembersType{guild_id: self.guild_id, members: db::ws_session::fetch_member(self.guild_id, &ctx.pool).await.unwrap()})).await;
    }
}

#[async_trait]
impl Handler for WsMessageFetchType {
    async fn handle(&self, ctx: WsChatSession) {
        let mut messages = db::ws_session::fetch_message(self.channel_id, &ctx.pool).await.unwrap();
        messages.reverse();
        ctx.send_event(MessageTypes::Messages(MessagesType{channel_id: self.channel_id, messages})).await;
    }
}

#[async_trait]
impl Handler for WsUserFetchType {
    async fn handle(&self, ctx: WsChatSession) {
        if let Some(user) = ctx.srv.find_user_by_id(self.id).await {
            ctx.send_event(MessageTypes::UserFetch(user.into())).await;
        }
    }
}

#[async_trait]
impl Handler for WsGuildCreate {
    async fn handle(&self, ctx: WsChatSession) {
        match db::ws_session::create_guild(ctx.user.id, self.to_owned(), &ctx.pool).await {
            Ok(rec) => {
                // self.rooms.lock().await.insert(rec.name.to_owned());
                /* Very Broken right now, waiting for a fix */
                ctx.send_event(MessageTypes::MemberCreate(MemberCreateType { id: ctx.user.id as usize, guild: rec })).await;
                // self.srv.join_room(rec.name.to_owned(), id).await;
                // guild create no longer have these
                // self.srv.send_message(&rec.id.to_owned(), MessageTypes::MessageCreate(MessageCreateType {content: "joined".to_string(), channel_id: rec.id.to_owned()})).await;
            }
            Err(err) => {
                println!("{:?}", err);
            }
        }
    }
}

#[async_trait]
impl Handler for WsChannelCreate {
    async fn handle(&self, ctx: WsChatSession) {
        match db::ws_session::create_channel(self.to_owned(), &ctx.pool).await {
            Ok(rec) => {
                ctx.rooms.lock().await.insert(rec.id.to_string());
                ctx.srv.join_room(rec.id.to_string(), ctx.user.id as usize).await;
                ctx.send_event(MessageTypes::ChannelCreate(ChannelCreateType {channel: rec.to_owned()})).await;
            }
            Err(err) => {
                println!("{:?}", err);
            }
        }
    }
}