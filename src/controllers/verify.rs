use actix_identity::Identity;
use actix_web::{
    http::{header::ContentType, StatusCode},
    web, HttpResponse,
};
use sqlx::PgPool;

use crate::db;
use crate::server;

pub async fn post(
    body: web::Json<server::VerifyEvent>,
    pool: web::Data<PgPool>,
    id: Option<Identity>,
) -> HttpResponse {
    if let Some(session_id) = id {
        let session_cookie: server::AuthCookie =
            serde_json::from_str(&session_id.id().unwrap()).unwrap();
        match db::verify::code(session_cookie.user_id, pool.as_ref()).await {
            Ok(code) => {
                if code.is_none() {
                    return HttpResponse::Ok().body("already verified");
                }
                if body.code == code.unwrap() {
                    // this can't error bro we just got the code
                    db::verify::delete_code(session_cookie.user_id, pool.as_ref())
                        .await
                        .unwrap();
                    HttpResponse::Ok().finish()
                } else {
                    HttpResponse::build(StatusCode::BAD_REQUEST)
                        .content_type(ContentType::plaintext())
                        .body("Bad request, wrong code")
                }
            }
            Err(_) => HttpResponse::build(StatusCode::BAD_REQUEST)
                .content_type(ContentType::plaintext())
                .body("Bad request (database errored, you are unlucky)"),
        }
    } else {
        HttpResponse::Ok().finish()
    }
}
