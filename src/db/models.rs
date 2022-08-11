use std::clone::Clone;

use serde::{Serialize, Deserialize};

use sqlx::types::{
    chrono::{
        NaiveDateTime,   
    },
    Uuid
};

use crate::format;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password: String,
    pub profile: Option<String>,
    #[serde(with = "format::date_format")]
    pub created_at: Option<NaiveDateTime>,
    pub description: Option<String>,
    pub allow_login: bool,
    pub is_online: bool,
    pub is_staff: bool,
    pub is_superuser: bool,
    pub code: Option<i32>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserSession {
    pub session_id: Uuid,
    pub userid: i64,
    #[serde(with = "format::date_format")]
    pub last_login: Option<NaiveDateTime>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Guild {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    #[serde(with = "format::date_format")]
    pub created_at: Option<NaiveDateTime>,
    pub creator_id: i64
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