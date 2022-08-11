use sqlx::{
    PgPool,
    types::{
        Uuid
    }
};
use crate::messages::WsGuildCreate;

use super::models::{
    User,
    UserSession,
    Guild,
    Member
};

pub async fn get_all(pool: &PgPool) -> sqlx::Result<Vec<User>> {
    sqlx::query_as!(
        User,
        r#"
SELECT *
FROM users;
        "#
    )
    .fetch_all(pool)
    .await
}

pub async fn get_all_sessions(pool: &PgPool) -> sqlx::Result<Vec<UserSession>> {
    sqlx::query_as!(
        UserSession,
        r#"
SELECT *
FROM user_sessions;
        "#
    )
    .fetch_all(pool)
    .await
}

pub async fn get_all_guilds(pool: &PgPool) -> sqlx::Result<Vec<Guild>> {
    sqlx::query_as!(
        Guild,
        r#"
SELECT *
FROM guild;
        "#
    ).fetch_all(pool).await
}

pub async fn get_all_members(pool: &PgPool) -> sqlx::Result<Vec<Member>> {
    sqlx::query_as!(
        Member,
        r#"
SELECT *
FROM member;
        "#
    ).fetch_all(pool).await
}

pub async fn get_user_by_session_id(session_id: String, pool: &PgPool) -> sqlx::Result<User> {
    println!("Received session_id: {}", session_id);
    sqlx::query_as!(
        User,
        r#"
SELECT us.*
FROM users AS us
INNER JOIN user_sessions AS u ON u.session_id = $1 AND u.userid = us.id;
        "#,
        Uuid::parse_str(&session_id).unwrap()
    )
    .fetch_one(pool)
    .await
}

pub async fn get_guild_ids_by_user_id(user_id: i64, pool: &PgPool) -> sqlx::Result<Vec<Uuid>> {
    match sqlx::query!(
        r#"
SELECT m.guild_id
FROM member AS m
INNER JOIN users AS us ON us.id = $1 AND m.user_id = us.id;
        "#,
        user_id
    ).fetch_all(pool).await {
        Ok(recs) => Ok(recs.iter().map(|s| s.guild_id).collect::<Vec<Uuid>>()),
        Err(err) => Err(err)
    }
}

pub async fn get_guild_by_id(guild_id: Uuid, pool: &PgPool) -> sqlx::Result<Guild> {
    sqlx::query_as!(
        Guild,
        r#"
SELECT * FROM guild WHERE id = $1;
        "#,
        guild_id
    )
    .fetch_one(pool)
    .await
}

/* START: creates */

pub async fn create_guild(id: i64, guild: WsGuildCreate, pool: &PgPool) -> sqlx::Result<Guild>  {
    match sqlx::query_as!(
        Guild,
        r#"
INSERT INTO guild (creator_id, name, description, icon) 
VALUES ($1, $2, $3, $4) RETURNING *
        "#,
        id,
        guild.name,
        guild.desc,
        guild.icon
    )
    .fetch_one(pool)
    .await {
        Ok(rec) => {
            sqlx::query!(
                r#"
            INSERT INTO member (guild_id, user_id)
            VALUES ($1, $2)
                "#,
                rec.id,
                id
            ).execute(pool).await.unwrap();
            Ok(rec)
        }
        Err(err) => Err(err)
    }
}

// pub async fn get_user_id_by_session_id(session_id: String, pool: &PgPool) -> sqlx::Result<i64> {
//     match sqlx::query!(
//         r#"
// SELECT us.id
// FROM user_sessions AS u
// INNER JOIN users AS us ON u.session_id = $1;
//         "#,
//         Uuid::parse_str(&session_id).unwrap()
//     )
//     .fetch_one(pool)
//     .await {
//         Ok(rec) => Ok(rec.id),
//         Err(err) => Err(err)
//     }
// }