use sqlx::{
    PgPool,
    types::{
        Uuid
    }
};
use crate::server::User;

pub async fn get_user_by_session_id(session_id: String, pool: &PgPool) -> sqlx::Result<User> {
    match sqlx::query_as!(
        User,
        r#"
SELECT us.*
FROM user_sessions AS u
INNER JOIN users AS us ON u.session_id = $1;
        "#,
        Uuid::parse_str(&session_id).unwrap()
    )
    .fetch_one(pool)
    .await {
        Ok(rec) => Ok(rec),
        Err(err) => Err(err)
    }
}