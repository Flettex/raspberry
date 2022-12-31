use sqlx::{
    PgPool,
    types::{
        Uuid
    }
};

use super::models::Guild;

pub async fn get_guild(id: Uuid, pool: &PgPool) -> sqlx::Result<Guild> {
    sqlx::query_as!(
        Guild,
        r#"
SELECT * FROM guild WHERE id = $1
        "#,
        id
    )
    .fetch_one(pool)
    .await
}