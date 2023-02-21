use std::clone::Clone;
use std::convert::From;

use serde::{Serialize, Deserialize};

use sqlx::types::{
    chrono::{
        NaiveDateTime,   
    },
    Uuid
};

use crate::format;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub id: Uuid,
    pub content: String,
    #[serde(with = "format::date_format2")]
    pub created_at: NaiveDateTime,
    #[serde(with = "format::date_format2")]
    pub edited_at: NaiveDateTime,
    pub author_id: i64,
    pub channel_id: Uuid,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password: String,
    pub profile: Option<String>,
    #[serde(with = "format::date_format2")]
    pub created_at: NaiveDateTime,
    pub description: Option<String>,
    pub allow_login: bool,
    pub is_online: bool,
    pub is_staff: bool,
    pub is_superuser: bool,
    pub code: Option<i64>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserSession {
    pub session_id: Uuid,
    pub userid: i64,
    #[serde(with = "format::date_format2")]
    pub last_login: NaiveDateTime,
    pub os: Option<String>,
    pub device: Option<String>,
    pub browser: Option<String>,
    pub original: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Guild {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    #[serde(with = "format::date_format2")]
    pub created_at: NaiveDateTime,
    pub creator_id: i64
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Channel {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub channel_type: i16,
    pub position: i64,
    #[serde(with = "format::date_format2")]
    pub created_at: NaiveDateTime,
    pub guild_id: Option<Uuid>,
    pub user1: Option<i64>,
    pub user2: Option<i64>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Member {
    pub id: Uuid,
    pub nick_name: Option<String>,
    #[serde(with = "format::date_format2")]
    pub joined_at: NaiveDateTime,
    pub guild_id: Uuid,
    pub user_id: i64
}

// Non-database models, modified for client.


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserClient {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub profile: Option<String>,
    #[serde(with = "format::date_format2")]
    pub created_at: NaiveDateTime,
    pub description: Option<String>,
    pub is_staff: bool,
    pub is_superuser: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GuildChannels {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    #[serde(with = "format::date_format2")]
    pub created_at: NaiveDateTime,
    pub creator_id: i64,
    pub channels: Vec<Channel>
}

impl From<User> for UserClient {
    fn from(u: User) -> UserClient {
        UserClient {
            id: u.id,
            username: u.username,
            email: u.email,
            profile: u.profile,
            created_at: u.created_at,
            description: u.description,
            is_staff: u.is_staff,
            is_superuser: u.is_superuser
        }
    }
}