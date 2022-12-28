use sqlx::{
    PgPool,
    types::{
        Uuid
    }
};

use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher,
        SaltString,
        Error
    },
    Argon2
};

pub struct UserAgent {
    pub os: Option<String>,
    pub device: Option<String>,
    pub browser: Option<String>,
    pub original: String
}

pub fn create_password(password: String) -> Result<String, Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    match argon2.hash_password(password.as_bytes(), &salt) {
        Ok(hash) => Ok(hash.to_string()),
        Err(err) => Err(err)
    }
}

pub async fn create_user(username: String, email: String, code: i64, password_hash: String, uag: UserAgent, pool: &PgPool) -> sqlx::Result<(Uuid, i64)> {
    match sqlx::query!(
        r#"
INSERT INTO users ( username, email, password, code )
VALUES ( $1, $2, $3, $4 )
RETURNING id
        "#,
        username,
        email,
        password_hash,
        code
    )
        .fetch_one(pool)
        .await {
        Ok(rec) => {
            Ok(
                // this query cannot error...
                (
                    sqlx::query!(
                        r#"
    INSERT INTO user_sessions ( userid, os, browser, device, original )
    VALUES ( $1, $2, $3, $4, $5 )
    RETURNING session_id
                        "#,
                        rec.id,
                        uag.os,
                        uag.browser,
                        uag.device,
                        uag.original
                    )
                    .fetch_one(pool)
                    .await?
                    .session_id,
                    rec.id
                )
            )
        }
        Err(e) => Err(e),
    }
}