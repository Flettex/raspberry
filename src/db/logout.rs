use sqlx::{postgres::PgQueryResult, types::Uuid, PgPool};

pub async fn delete_session(session_id: Uuid, pool: &PgPool) -> sqlx::Result<PgQueryResult> {
    sqlx::query!(
        r#"
DELETE FROM user_sessions WHERE session_id = $1
        "#,
        session_id
    )
    .execute(pool)
    .await
}
