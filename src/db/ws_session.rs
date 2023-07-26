use crate::{messages::{WsChannelCreate, WsChannelUpdate, WsGuildCreate, UserFetchType}, db::models::MessageWithGuild};
use sqlx::{postgres::PgQueryResult, types::Uuid, PgPool};

use super::models::{Channel, Guild, Member, Message, User, UserSession, MemberClient, MessageInfo};

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
    )
    .fetch_all(pool)
    .await
}

pub async fn get_all_members(pool: &PgPool) -> sqlx::Result<Vec<Member>> {
    sqlx::query_as!(
        Member,
        r#"
SELECT *
FROM member;
        "#
    )
    .fetch_all(pool)
    .await
}

pub async fn get_user_by_session_id(session_id: String, pool: &PgPool) -> sqlx::Result<User> {
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

pub async fn get_guild_by_id(guild_id: Uuid, pool: &PgPool) -> sqlx::Result<Guild> {
    sqlx::query_as!(
        Guild,
        r#"
SELECT * FROM guild WHERE id = $1
        "#,
        guild_id
    )
    .fetch_one(pool)
    .await
}

pub async fn get_guilds_by_user_id(user_id: i64, pool: &PgPool) -> sqlx::Result<Vec<Guild>> {
    match sqlx::query_as!(
        Guild,
        r#"
SELECT g.*
FROM member AS m
INNER JOIN guild AS g ON g.id = m.guild_id
WHERE m.user_id = $1;
        "#,
        user_id
    )
    .fetch_all(pool)
    .await
    {
        Ok(recs) => Ok(recs),
        Err(err) => Err(err),
    }
}

pub async fn get_channels_by_guild_id(guild_id: Uuid, pool: &PgPool) -> sqlx::Result<Vec<Channel>> {
    sqlx::query_as!(
        Channel,
        r#"
SELECT * FROM channel WHERE guild_id = $1
        "#,
        guild_id
    )
    .fetch_all(pool)
    .await
}

pub async fn fetch_message(channel_id: Uuid, pool: &PgPool) -> sqlx::Result<Vec<Message>> {
    sqlx::query_as!(
        Message,
        r#"
SELECT *
FROM message
WHERE channel_id = $1
ORDER BY (created_at) DESC
LIMIT 1000 
        "#,
        channel_id
    )
    .fetch_all(pool)
    .await
}

pub async fn fetch_member(guild_id: Uuid, pool: &PgPool) -> sqlx::Result<Vec<MemberClient>> {
    // we might have to run 2 queries
    match sqlx::query!(
        r#"
SELECT m.*, u.username, u.profile, u.description, u.created_at, u.is_online, u.is_staff, u.is_superuser
FROM member m
JOIN users u ON m.user_id = u.id
WHERE guild_id = $1
LIMIT 1000
        "#,
        guild_id
    )
    .fetch_all(pool)
    .await {
        Ok(rec) => Ok(rec.iter().map(|m| MemberClient{
            id: m.id,
            nick_name: m.nick_name.to_owned(),
            joined_at: m.joined_at,
            guild_id: m.guild_id,
            user_id: m.user_id,
            user: UserFetchType{
                id: m.user_id,
                username: m.username.to_owned(),
                profile: m.profile.to_owned(),
                description: m.description.to_owned(),
                created_at: m.created_at,
                is_staff: m.is_staff,
                is_superuser: m.is_superuser
            }
        }).collect()),
        Err(err) => Err(err)
    }
}

/* START: creates */

pub async fn create_guild(id: i64, guild: WsGuildCreate, pool: &PgPool) -> sqlx::Result<Guild> {
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
    .await
    {
        Ok(rec) => {
            sqlx::query!(
                r#"
            INSERT INTO member (guild_id, user_id)
            VALUES ($1, $2)
                "#,
                rec.id,
                id
            )
            .execute(pool)
            .await
            .unwrap();
            Ok(rec)
        }
        Err(err) => Err(err),
    }
}

pub async fn create_message(
    content: String,
    author_id: i64,
    channel_id: Uuid,
    pool: &PgPool,
) -> sqlx::Result<MessageWithGuild> {
    // Production database SUCKS. The CTE columns are all type of NULL which
    // messes up the Option<T> types. We have to perform an INNER JOIN on the
    // origional table message in order to return the full message.
    // The performance shouldn't be much worse, since we are using primary keys.
    sqlx::query_as!(
        MessageWithGuild,
        r#"
WITH cte AS (
    INSERT INTO message (content, author_id, channel_id)
    VALUES ($1, $2, $3)
    RETURNING *
)
SELECT m.*, ch.guild_id, ch.user1, ch.user2
FROM cte AS c
INNER JOIN channel AS ch ON c.channel_id = ch.id
INNER JOIN message AS m ON m.id = c.id
"#,
        content,
        author_id,
        channel_id
    )
    .fetch_one(pool)
    .await
}

pub async fn delete_message(message_id: Uuid, pool: &PgPool) -> sqlx::Result<MessageInfo> {
    sqlx::query_as!(
        MessageInfo,
        r#"
WITH cte AS (
    DELETE FROM message WHERE id = $1 RETURNING channel_id
)
SELECT ch.id AS channel_id, ch.guild_id, ch.user1, ch.user2
FROM cte AS c
INNER JOIN channel AS ch ON c.channel_id = ch.id
        "#,
        message_id
    )
    .fetch_one(pool)
    .await
}

pub async fn create_channel(channel: WsChannelCreate, pool: &PgPool) -> sqlx::Result<Channel> {
    match sqlx::query_as!(
        Channel,
        r#"
INSERT INTO channel (name, description, position, guild_id, channel_type) 
VALUES ($1, $2, $3, $4, $5) RETURNING *
        "#,
        channel.name,
        channel.desc,
        channel.position,
        Some(channel.guild_id),
        channel.channel_type
    )
    .fetch_one(pool)
    .await
    {
        Ok(rec) => Ok(rec),
        Err(err) => Err(err),
    }
}

pub async fn join_guild(user_id: i64, guild_id: Uuid, pool: &PgPool) -> sqlx::Result<Vec<Channel>> {
    sqlx::query_as!(
        Channel,
        r#"
WITH gids AS (
    INSERT INTO member (user_id, guild_id)
    VALUES ($1, $2) RETURNING guild_id
) SELECT * FROM channel WHERE guild_id = (SELECT guild_id from gids)
        "#,
        user_id,
        Some(guild_id)
    )
    .fetch_all(pool)
    .await
}

pub async fn toggle_user_status(
    user_id: i64,
    online: bool,
    pool: &PgPool,
) -> sqlx::Result<PgQueryResult> {
    sqlx::query!(
        r#"
UPDATE users SET is_online = $1 WHERE id = $2
        "#,
        online,
        user_id
    )
    .execute(pool)
    .await
}

pub async fn update_user_last_login(
    session_id: Uuid,
    pool: &PgPool,
) -> sqlx::Result<PgQueryResult> {
    sqlx::query!(
        r#"
UPDATE user_sessions SET last_login = NOW() WHERE session_id = $1
        "#,
        session_id
    )
    .execute(pool)
    .await
}

pub async fn update_message(
    message_id: Uuid,
    content: String,
    pool: &PgPool,
) -> sqlx::Result<MessageWithGuild> {
    sqlx::query_as!(
        MessageWithGuild,
        r#"
UPDATE message
SET content = $1, edited_at = NOW()
FROM
  (
    SELECT guild_id, user1, user2
    FROM channel AS c
    JOIN message m ON m.channel_id = c.id
  ) sub
WHERE id = $2
RETURNING *
        "#,
        content,
        message_id
    )
    .fetch_one(pool)
    .await
    // let mut transaction = pool.begin().await.unwrap();
    // sqlx::query!(
    //     r#"
    // UPDATE message
    // SET content = $1, edited_at = NOW()
    // WHERE id = $2
    //     "#,
    //     content,
    //     message_id
    // ).execute(&mut transaction).await.unwrap();
    // let res = sqlx::query_as!(
    //     Message,
    //     r#"
    // SELECT * FROM message WHERE id = $1
    //     "#,
    //     message_id
    // ).fetch_one(&mut transaction).await;
    // transaction.commit().await.unwrap();
    // res
}

pub async fn update_nickname(id: i64, nick: String, pool: &PgPool) -> sqlx::Result<Uuid> {
    match sqlx::query!(
        r#"
UPDATE member
SET nick_name = $2
WHERE user_id = $1
RETURNING guild_id
        "#,
        id,
        nick
    )
    .fetch_one(pool)
    .await
    {
        Ok(rec) => Ok(rec.guild_id),
        Err(err) => Err(err),
    }
}

pub async fn update_channel(chan: &WsChannelUpdate, pool: &PgPool) -> sqlx::Result<Channel> {
    sqlx::query_as!(
        Channel,
        r#"
UPDATE channel
SET name = $2, description = $3, position = $4, channel_type = $5
WHERE id = $1
RETURNING *
        "#,
        chan.id,
        chan.name,
        chan.desc,
        chan.position,
        chan.channel_type,
    )
    .fetch_one(pool)
    .await
}

pub async fn delete_channel(id: Uuid, pool: &PgPool) -> sqlx::Result<Option<Uuid>> {
    match sqlx::query!(
        r#"
DELETE FROM channel WHERE id = $1 RETURNING guild_id
        "#,
        id
    )
    .fetch_one(pool)
    .await
    {
        Ok(rec) => Ok(rec.guild_id),
        Err(err) => Err(err),
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
