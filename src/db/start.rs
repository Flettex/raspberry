use sqlx::{
    PgPool
};

// use super::models::{
//     User,
//     UserSession,
//     Guild
// };

pub async fn get_all_guild_names(pool: &PgPool) -> sqlx::Result<Vec<String>> {
    match sqlx::query!(
        r#"
SELECT name
FROM guild;
        "#
    )
    .fetch_all(pool)
    .await {
        Ok(recs) => Ok(recs.iter().map(|s| s.name.clone()).collect::<Vec<String>>()),
        Err(err) => Err(err)
    }
}