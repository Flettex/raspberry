use actix_identity::Identity;
use actix_web::{
    http::{header::ContentType, StatusCode},
    web, HttpResponse, HttpRequest, HttpMessage
};
use actix_session::Session;

use argon2::{
    password_hash::{
        PasswordHash, PasswordVerifier
    },
    Argon2
};

use sqlx::postgres::PgPool;
use serde_json::json;

use crate::server;
use crate::db;
use utoipa;

#[utoipa::path(
    post,
    path = "/login",
    responses(
        (status = 200, description = "Successful Response", body = String),
        (status = 400, description = "Password does not match or email doesn't exist or the database failed to create a session", body = String)
    ),
    request_body(content = LoginEvent, description = "user email, user password", content_type = "application/json")
)]
pub async fn post(
    body: web::Json<server::LoginEvent>,
    pool: web::Data<PgPool>,
    session: Session,
    id: Option<Identity>,
    req: HttpRequest
) -> HttpResponse {
    if let Some(_) = id {
        return HttpResponse::Ok().finish();
    }
    let pl = body.into_inner();
    log::info!("GIVEN: {}\n REAL CODE: {}", pl.code, session.get::<String>("captcha").unwrap().unwrap());
    if pl.code != session.get::<String>("captcha").unwrap().unwrap() {
        return HttpResponse::build(StatusCode::BAD_REQUEST)
            .content_type(ContentType::plaintext())
            .body("You are a bot");
    }
    session.remove("captcha");
    let argon2 = Argon2::default();
    match db::login::get_user_and_password(pl.email, pool.as_ref()).await {
        Ok((user_id, password)) => {
            if argon2.verify_password(pl.password.as_bytes(), &PasswordHash::new(&password).unwrap()).is_ok() {
                match db::login::create_session(user_id, pool.as_ref()).await {
                    Ok(session_id) => {
                        Identity::login(&req.extensions(), json!({
                            "user_id": user_id,
                            "session_id": session_id.to_string()
                        }).to_string()).unwrap();
                        HttpResponse::Ok().finish()
                    }
                    Err(err) => {
                        println!("{}", err);
                        HttpResponse::build(StatusCode::BAD_REQUEST)
                            .content_type(ContentType::plaintext())
                            .body("DB failed to create session")
                    } 
                }
            } else {
                HttpResponse::build(StatusCode::BAD_REQUEST)
                    .content_type(ContentType::plaintext())
                    .body("Password does not match")
            }
        }
        Err(err) => {
            println!("{}", err);
            HttpResponse::build(StatusCode::BAD_REQUEST)
                .content_type(ContentType::plaintext())
                .body("Email is not in record")
        }
    }
}