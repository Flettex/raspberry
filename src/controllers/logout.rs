use actix_identity::Identity;
use actix_web::{
    http::{header::ContentType, StatusCode},
    web, HttpResponse,
};
use sqlx::PgPool;

use crate::db;
use crate::server::AuthCookie;

pub async fn delete(id: Option<Identity>, pool: web::Data<PgPool>) -> HttpResponse {
    if let Some(session_id) = id {
        let session_cookie: AuthCookie = serde_json::from_str(&session_id.id().unwrap()).unwrap();
        match db::logout::delete_session(
            sqlx::types::Uuid::parse_str(&session_cookie.session_id).unwrap(),
            pool.as_ref(),
        )
        .await
        {
            Ok(_) => {
                session_id.logout();
                HttpResponse::Ok().finish()
            }
            Err(_) => {
                session_id.logout();
                HttpResponse::build(StatusCode::BAD_REQUEST)
                    .content_type(ContentType::plaintext())
                    .body("Bad request")
            }
        }
    } else {
        HttpResponse::Ok().finish()
    }
}
