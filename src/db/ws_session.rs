use sqlx::{
    PgPool,
    // types::{
    //     Uuid
    // }
};
use crate::server::{
    User,
    UserSession
};

pub async fn get_all(pool: &PgPool) -> sqlx::Result<Vec<User>> {
    match sqlx::query_as!(
        User,
        r#"
SELECT *
FROM users;
        "#
    )
    .fetch_all(pool)
    .await {
        Ok(recs) => Ok(recs),
        Err(err) => Err(err)
    }
}

pub async fn get_all_sessions(pool: &PgPool) -> sqlx::Result<Vec<UserSession>> {
    match sqlx::query_as!(
        UserSession,
        r#"
SELECT *
FROM user_sessions;
        "#
    )
    .fetch_all(pool)
    .await {
        Ok(recs) => Ok(recs),
        Err(err) => Err(err)
    }
}
// pub async fn get_user_by_session_id(session_id: String, pool: &PgPool) -> sqlx::Result<User> {
//     match sqlx::query_as!(
//         User,
//         r#"
// SELECT us.*
// FROM user_sessions AS u
// INNER JOIN users AS us ON u.session_id = $1;
//         "#,
//         Uuid::parse_str(&session_id).unwrap()
//     )
//     .fetch_one(pool)
//     .await {
//         Ok(rec) => Ok(rec),
//         Err(err) => Err(err)
//     }
// }

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