use super::{send::*, MessageTypes};
use crate::db;
use crate::session::WsChatSession;
use crate::PLACEHOLDER_UUID;
use async_trait::async_trait;
use enum_dispatch::enum_dispatch;
use raspberry_macros::ratelimit;
use serde::{self, Deserialize, Serialize};
use sqlx::types::Uuid;
use std::clone::Clone;

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
    pub nonce: Uuid,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[ratelimit(2)]
pub struct WsMessageUpdate {
    pub id: Uuid,
    pub content: String,
    pub nonce: Uuid,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[ratelimit(1)]
pub struct WsMessageDelete {
    pub id: Uuid,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WsMessageFetchType {
    pub channel_id: Uuid,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[ratelimit(60)]
pub struct WsGuildCreate {
    pub name: String,
    pub desc: Option<String>,
    pub icon: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WsDMChannelCreate {
    pub user_id: i64
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WsChannelCreate {
    pub name: String,
    pub desc: Option<String>,
    pub position: i64,
    pub guild_id: Uuid,
    pub channel_type: i16,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WsChannelUpdate {
    pub id: Uuid,
    pub name: String,
    pub desc: Option<String>,
    pub position: i64,
    // this might cause weird behaviour later?
    pub channel_type: i16,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WsChannelDelete {
    pub id: Uuid,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WsMemberCreate {
    pub guild_id: Uuid,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WsMemberUpdate {
    pub nickname: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WsMemberFetchType {
    pub guild_id: Uuid,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WsUserFetchType {
    pub id: i64,
}

#[async_trait]
impl Handler for WsMessageCreate {
    async fn handle(&self, ctx: WsChatSession) {
        // commands are no longer supported
        // if self.content.starts_with('/') {
        //     let v: Vec<&str> = self.content.splitn(2, ' ').collect();
        //     // TODO: implement an interaction system to remove commands system...
        //     match v[0] {
        //         "/list" => {
        //             println!("List rooms");
        //             let rooms = ctx.srv.list_rooms().await;
        //             ctx.srv
        //                 .send_message(
        //                     &self.channel_id,
        //                     MessageTypes::MessageCreate(Message::system(
        //                         rooms.join(", "),
        //                         &self.channel_id.clone(),
        //                         0,
        //                     )),
        //                 )
        //                 .await;
        //         }
        //         "/join" => {
        //             if v.len() == 2 {
        //                 log::info!("{:?} joining {}", ctx.user.username, v[1].to_owned());
        //                 ctx.rooms.lock().await.insert(v[1].to_owned());
        //                 // self.send_event(MessageTypes::MemberCreate(MemberCreateType { id, room: v[1].to_owned() })).await;
        //                 ctx.srv
        //                     .join_room(v[1].to_owned(), ctx.user.id as usize)
        //                     .await;
        //                 ctx.srv
        //                     .send_message(
        //                         &v[1].to_owned(),
        //                         MessageTypes::MessageCreate(Message::system(
        //                             "joined".to_string(),
        //                             v[1],
        //                             0,
        //                         )),
        //                     )
        //                     .await;
        //             } else {
        //                 ctx.srv
        //                     .send_message(
        //                         &self.channel_id,
        //                         MessageTypes::MessageCreate(Message::system(
        //                             "!!! room name is required".to_string(),
        //                             &self.channel_id.clone(),
        //                             0,
        //                         )),
        //                     )
        //                     .await;
        //             }
        //         }
        //         "/leave" => {
        //             if v.len() == 2 {
        //                 if v[1] == PLACEHOLDER_UUID {
        //                     ctx.send_event(MessageTypes::MessageCreate(Message::system(
        //                         "you can't leave Main dumbass".to_string(),
        //                         PLACEHOLDER_UUID,
        //                         0,
        //                     )))
        //                     .await;
        //                     return ();
        //                 }
        //                 log::info!("{:?} leaving {}", ctx.user.username, v[1].to_owned());
        //                 let mut rooms = ctx.rooms.lock().await;
        //                 rooms.remove(v[1]);
        //                 // self.send_event(MessageTypes::MemberRemove(MemberRemoveType { id, room: v[1].to_owned() })).await;
        //                 ctx.srv
        //                     .leave_room(v[1].to_owned(), ctx.user.id as usize)
        //                     .await;
        //                 ctx.srv
        //                     .send_message(
        //                         &v[1].to_owned(),
        //                         MessageTypes::MessageCreate(Message::system(
        //                             "left".to_string(),
        //                             v[1],
        //                             0,
        //                         )),
        //                     )
        //                     .await;
        //             } else {
        //                 ctx.srv
        //                     .send_message(
        //                         &self.channel_id,
        //                         MessageTypes::MessageCreate(Message::system(
        //                             "!!! room name is required".to_string(),
        //                             &self.channel_id.clone(),
        //                             0,
        //                         )),
        //                     )
        //                     .await;
        //             }
        //         }
        //         _ => {
        //             ctx.srv
        //                 .send_message(
        //                     &self.channel_id,
        //                     MessageTypes::MessageCreate(Message::system(
        //                         format!("!!! unknown command {:?}", self.content),
        //                         &self.channel_id.clone(),
        //                         0,
        //                     )),
        //                 )
        //                 .await
        //         }
        //     }
        //     return ();
        // }

        // The permissions update will be soon

        // if !ctx.rooms.lock().await.contains(&self.channel_id) {
        //     // bro's trying to send message to a room they don't have access to
        //     return ();
        // }
        // let msg = format!("{}: {}", self.user.username, m.content);
        let msg = &self.content;
        log::info!("{} {}", msg, ctx.user.id);
        if self.channel_id == PLACEHOLDER_UUID {
            ctx.srv
                .send_guild_message(
                    &self.channel_id,
                    MessageTypes::MessageCreate(Message::user(
                        msg.to_string(),
                        &self.channel_id.clone(),
                        ctx.user.to_owned().into(),
                        self.nonce,
                    )),
                )
                .await;
        } else if let Ok(msg) = db::ws_session::create_message(
                msg.to_string(),
                ctx.user.id,
                Uuid::parse_str(&self.channel_id.clone()).unwrap(),
                &ctx.pool,
            )
            .await
            {
                if let Some(guild_id) = msg.guild_id {
                    ctx.srv
                        .send_guild_message(
                            &guild_id.to_string(),
                            MessageTypes::MessageCreate(Message::from_guildmsg(
                                msg,
                                ctx.user.to_owned().into(),
                                self.nonce,
                            )),
                        )
                        .await;
                } else {
                    // no guild_id means dm channel
                    // Therefore user1, user2 fields must both be present. We will unwrap.
                    // let user1 = msg.user1;
                    // let user2 = msg.user2;
                    ctx.srv
                        .send_dm(
                            msg.user1.unwrap() as usize,
                            msg.user2.unwrap() as usize,
                            MessageTypes::MessageCreate(Message::from_guildmsg(
                                msg,
                                ctx.user.to_owned().into(),
                                self.nonce,
                            ))
                        )
                        .await;
                }
        }
    }
}

#[async_trait]
impl Handler for WsMessageUpdate {
    async fn handle(&self, ctx: WsChatSession) {
        // Should not error unless the user delete and update the message at the exact same time. VERY unlikely.
        if let Ok(updated) = db::ws_session::update_message(self.id, ctx.user.id, self.content.to_owned(), &ctx.pool).await {
            if let Some(guild_id) = updated.guild_id {
                ctx.srv
                    .send_guild_message(
                        &guild_id.to_string(),
                        MessageTypes::MessageUpdate(Message {
                            id: updated.id,
                            content: updated.content,
                            created_at: updated.created_at,
                            edited_at: updated.edited_at,
                            author: ctx.user.to_owned().into(),
                            channel_id: updated.channel_id,
                            nonce: self.nonce,
                        }),
                    )
                    .await;
            } else {
                ctx.srv
                    .send_dm(
                        updated.user1.unwrap() as usize,
                        updated.user2.unwrap() as usize,
                        MessageTypes::MessageUpdate(Message {
                            id: updated.id,
                            content: updated.content,
                            created_at: updated.created_at,
                            edited_at: updated.edited_at,
                            author: ctx.user.to_owned().into(),
                            channel_id: updated.channel_id,
                            nonce: self.nonce,
                        }),
                    )
                    .await;
            }
        }
    }
}

#[async_trait]
impl Handler for WsMessageDelete {
    async fn handle(&self, ctx: WsChatSession) {
        if let Ok(info) = db::ws_session::delete_message(self.id, &ctx.pool).await {
            if let Some(guild_id) = info.guild_id {
                ctx.srv
                    .send_guild_message(
                        &guild_id.to_string(),
                        MessageTypes::MessageDelete(MessageDeleteType {
                            id: self.id,
                            channel_id: info.channel_id,
                        }),
                    )
                    .await
            } else {
                ctx.srv
                    .send_dm(
                        info.user1.unwrap() as usize,
                        info.user2.unwrap() as usize,
                        MessageTypes::MessageDelete(MessageDeleteType {
                            id: self.id,
                            channel_id: info.channel_id,
                        })
                    )
                    .await;
            }
        }
    }
}

#[async_trait]
impl Handler for WsMessageFetchType {
    async fn handle(&self, ctx: WsChatSession) {
        let mut messages = db::ws_session::fetch_message(self.channel_id, &ctx.pool)
            .await
            .unwrap();
        messages.reverse();
        ctx.send_event(MessageTypes::Messages(MessagesType {
            channel_id: self.channel_id,
            messages,
        }))
        .await;
    }
}

#[async_trait]
impl Handler for WsMemberCreate {
    async fn handle(&self, ctx: WsChatSession) {
        match db::ws_session::join_guild(ctx.user.id, self.guild_id, &ctx.pool).await {
            Ok(channels) => {
                let guild = db::ws_session::get_guild_by_id(self.guild_id, &ctx.pool)
                    .await
                    .unwrap();
                ctx.srv
                    .join_guild(guild.id.to_string(), ctx.user.id as usize)
                    .await;
                let mut lock = ctx.rooms.lock().await;
                lock.insert(guild.id.to_string());
                drop(lock);
                ctx.send_event(MessageTypes::GuildCreate(GuildCreateType {
                    guild: guild.to_owned(),
                }))
                .await;
                ctx.srv
                    .send_guild_message(
                        &guild.id.to_string(),
                        MessageTypes::MemberCreate(MemberCreateType {
                            id: ctx.user.id as usize,
                            guild: guild.to_owned(),
                        }),
                    )
                    .await;
                for c in channels {
                    ctx.send_event(MessageTypes::ChannelCreate(ChannelCreateType {
                        channel: c.to_owned(),
                    }))
                    .await;
                }
            }
            Err(err) => {
                println!("{:?}", err);
            }
        }
    }
}

// member update is nickname update

#[async_trait]
impl Handler for WsMemberUpdate {
    async fn handle(&self, ctx: WsChatSession) {
        match db::ws_session::update_nickname(ctx.user.id, self.nickname.to_string(), &ctx.pool)
            .await
        {
            Ok(guild_id) => {
                // frontend can handle this, if it errors then too bad!
                // ctx.send_event(MessageTypes::GuildRemove(GuildRemoveType { guild: guild.to_owned() })).await;
                ctx.srv
                    .send_guild_message(
                        &guild_id.to_string(),
                        MessageTypes::MemberUpdate(MemberUpdateType {
                            id: ctx.user.id as usize,
                            nickname: self.nickname.to_string(),
                        }),
                    )
                    .await;
            }
            Err(err) => {
                println!("{:?}", err);
            }
        }
    }
}

// member delete will be implemented once kicking is implemented in the frontend

#[async_trait]
impl Handler for WsMemberFetchType {
    async fn handle(&self, ctx: WsChatSession) {
        if self.guild_id.to_string() == *PLACEHOLDER_UUID {
            // nobody is in Main though hmm, this is purely waste of bandwidth!
            return;
        }
        ctx.send_event(MessageTypes::Members(MembersType {
            guild_id: self.guild_id,
            members: db::ws_session::fetch_member(self.guild_id, &ctx.pool)
                .await
                .unwrap(),
        }))
        .await;
    }
}

#[async_trait]
impl Handler for WsUserFetchType {
    async fn handle(&self, ctx: WsChatSession) {
        if let Some(user) = ctx.srv.find_user_by_id(self.id as usize).await {
            ctx.send_event(MessageTypes::UserFetch(user.into())).await;
        } else {
            match db::ws_session::get_user_by_id(self.id, &ctx.pool).await {
                Ok(user) => {
                    ctx.send_event(MessageTypes::UserFetch(user)).await;
                }
                Err(err) => {
                    println!("{:?}", err);
                }
            }
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
                ctx.send_event(MessageTypes::MemberCreate(MemberCreateType {
                    id: ctx.user.id as usize,
                    guild: rec,
                }))
                .await;
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
impl Handler for WsDMChannelCreate {
    async fn handle(&self, ctx: WsChatSession) {
        match db::ws_session::create_dm_channel(ctx.user.id, self.user_id, &ctx.pool).await {
            Ok(rec) => {
                ctx.srv.send_dm(ctx.user.id as usize, self.user_id as usize, MessageTypes::ChannelCreate(ChannelCreateType {
                    channel: rec.to_owned()
                }))
                .await;
            }
            Err(err) => {
                println!("{:?}", err);
                // ctx.srv.send_dm(ctx.user.id as usize, self.user_id as usize, MessageTypes::ChannelCreate(ChannelCreateType {
                //     channel: 
                // }))
                // .await;
            }
        }
    }
}

#[async_trait]
impl Handler for WsChannelCreate {
    async fn handle(&self, ctx: WsChatSession) {
        if self.channel_type == 1 {
            return;
        }
        match db::ws_session::create_channel(self.to_owned(), &ctx.pool).await {
            Ok(rec) => {
                // ctx.rooms.lock().await.insert(rec.id.to_string());
                // ctx.srv
                //     .join_room(rec.id.to_string(), ctx.user.id as usize)
                //     .await;
                ctx.send_event(MessageTypes::ChannelCreate(ChannelCreateType {
                    channel: rec.to_owned(),
                }))
                .await;
            }
            Err(err) => {
                println!("{:?}", err);
            }
        }
    }
}

#[async_trait]
impl Handler for WsChannelUpdate {
    async fn handle(&self, ctx: WsChatSession) {
        // will error if it is a dm channel
        if let Ok(updated) = db::ws_session::update_channel(self, &ctx.pool).await {
            // CHANNEL_UPDATE is forbidden if it is a DM channel
            if let Some(guild_id) = updated.guild_id {
                ctx.srv
                    .send_guild_message(
                        &guild_id.to_string(),
                        MessageTypes::ChannelUpdate(ChannelUpdateType {
                            id: updated.id,
                            desc: updated.description,
                            position: updated.position,
                            channel_type: updated.channel_type,
                        }),
                    )
                    .await;
            } else {
                // that's weird...
            }
        }
    }
}

#[async_trait]
impl Handler for WsChannelDelete {
    async fn handle(&self, ctx: WsChatSession) {
        if let Ok(Some(guild_id)) = db::ws_session::delete_channel(self.id, &ctx.pool).await {
            // if let Some(guild_id) = guild_id {
                ctx.srv.send_guild_message(
                    &guild_id.to_string(),
                    MessageTypes::ChannelDelete(ChannelDeleteType { id: self.id }),
                ).await;
            // } else {
                // wtf it is a DM channel??!?
            // }
        }
    }
}
