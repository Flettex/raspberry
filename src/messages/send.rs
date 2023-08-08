use crate::db::models::{self, Channel, Guild, GuildChannels, User, UserClient, MemberClient};
use crate::format;
use chrono::{NaiveDate, NaiveDateTime, Utc};
use serde::{self, Deserialize, Serialize};
use sqlx::types::Uuid;
use std::clone::Clone;

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
            author: UserFetchType {
                id: author_id,
                username: "System".to_string(),
                profile: None,
                description: None,
                created_at: NaiveDate::from_ymd_opt(2016, 7, 8)
                    .unwrap()
                    .and_hms_opt(9, 10, 11)
                    .unwrap(),
                is_staff: true,
                is_superuser: true,
            },
            edited_at: Utc::now().naive_utc(),
            created_at: Utc::now().naive_utc(),
            nonce: Uuid::new_v4(),
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
            nonce,
        }
    }

    #[allow(dead_code)]
    pub fn from_dbmsg(msg: models::Message, author: UserFetchType, nonce: Uuid) -> Self {
        Self {
            id: msg.id,
            content: msg.content,
            channel_id: msg.channel_id,
            author,
            edited_at: msg.edited_at,
            created_at: msg.edited_at,
            nonce,
        }
    }

    pub fn from_guildmsg(msg: models::MessageWithGuild, author: UserFetchType, nonce: Uuid) -> Self {
        Self {
            id: msg.id,
            content: msg.content,
            channel_id: msg.channel_id,
            author,
            edited_at: msg.edited_at,
            created_at: msg.edited_at,
            nonce,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MessageDeleteType {
    pub id: Uuid,
    pub channel_id: Uuid,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ReadyEventType {
    pub user: UserClient,
    pub guilds: Vec<GuildChannels>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GuildCreateType {
    pub guild: Guild,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChannelCreateType {
    pub channel: Channel,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChannelUpdateType {
    pub id: Uuid,
    pub desc: Option<String>,
    pub position: i64,
    pub channel_type: i16,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChannelDeleteType {
    pub id: Uuid,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MemberCreateType {
    // user id
    pub id: usize,
    pub guild: Guild,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MemberRemoveType {
    pub id: usize,
    pub room: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MemberUpdateType {
    pub id: usize,
    pub nickname: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MessagesType {
    pub channel_id: Uuid,
    pub messages: Vec<models::Message>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MembersType {
    pub guild_id: Uuid,
    pub members: Vec<MemberClient>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
            is_superuser: u.is_superuser,
        }
    }
}
