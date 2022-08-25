use sqlx::{
    PgPool
};

// use super::models::{
//     User,
//     UserSession,
//     Guild
// };

#[allow(dead_code)]
pub async fn get_all_channel_names(pool: &PgPool) -> sqlx::Result<Vec<String>> {
    match sqlx::query!(
        r#"
SELECT id
FROM channel;
        "#
    )
    .fetch_all(pool)
    .await {
        Ok(recs) => Ok(recs.iter().map(|s| s.id.to_string()).collect::<Vec<String>>()),
        Err(err) => Err(err)
    }
}