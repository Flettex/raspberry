use sqlx::{types::Uuid, PgPool};

use super::models::Channel;

pub async fn get_channel(id: Uuid, pool: &PgPool) -> sqlx::Result<Channel> {
    sqlx::query_as!(
        Channel,
        r#"
SELECT * FROM channel WHERE id = $1
        "#,
        id
    )
    .fetch_one(pool)
    .await
}
