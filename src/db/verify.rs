use sqlx::{postgres::PgQueryResult, PgPool};

pub async fn code(user_id: i64, pool: &PgPool) -> sqlx::Result<Option<i64>> {
    match sqlx::query!(
        r#"
SELECT code FROM users WHERE id = $1
        "#,
        user_id
    )
    .fetch_one(pool)
    .await
    {
        Ok(rec) => Ok(rec.code),
        Err(err) => Err(err),
    }
}

pub async fn delete_code(user_id: i64, pool: &PgPool) -> sqlx::Result<PgQueryResult> {
    sqlx::query!(
        r#"
UPDATE users SET code = NULL WHERE id = $1
        "#,
        user_id
    )
    .execute(pool)
    .await
}
