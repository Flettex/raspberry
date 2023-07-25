use sqlx::{types::Uuid, PgPool};

use super::signup::UserAgent;

pub async fn create_session(user_id: i64, pool: &PgPool, uag: UserAgent) -> sqlx::Result<Uuid> {
    match sqlx::query!(
        r#"
INSERT INTO user_sessions ( userid, os, browser, device, original )
VALUES ( $1, $2, $3, $4, $5 )
RETURNING session_id
        "#,
        user_id,
        uag.os,
        uag.browser,
        uag.device,
        uag.original
    )
    .fetch_one(pool)
    .await
    {
        Ok(rec) => Ok(rec.session_id),
        Err(err) => Err(err),
    }
}

pub async fn get_user_and_password(email: String, pool: &PgPool) -> sqlx::Result<(i64, String)> {
    match sqlx::query!(
        r#"
SELECT id, password FROM users WHERE email = $1
        "#,
        email
    )
    .fetch_one(pool)
    .await
    {
        Ok(rec) => Ok((rec.id, rec.password)),
        Err(err) => Err(err),
    }
}
