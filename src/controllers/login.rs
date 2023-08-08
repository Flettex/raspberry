use actix_identity::Identity;
use actix_session::Session;
use actix_web::{
    http::{header::ContentType, StatusCode},
    web, HttpMessage, HttpRequest, HttpResponse,
};

use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};
use std::sync::Arc;

use serde_json::json;
use sqlx::postgres::PgPool;
use user_agent_parser::UserAgentParser;

use crate::db;
use crate::server;
use db::signup::UserAgent;
use utoipa;

use super::extractor::ValidatedForm;

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
    body: ValidatedForm<server::LoginEvent>,
    pool: web::Data<PgPool>,
    session: Session,
    id: Option<Identity>,
    req: HttpRequest,
    ua_parser: web::Data<Arc<UserAgentParser>>,
) -> HttpResponse {
    if id.is_some() {
        return HttpResponse::Ok().finish();
    }
    let pl = body.decode();
    log::info!(
        "GIVEN: {}\n REAL CODE: {}",
        pl.code,
        session.get::<String>("captcha").unwrap().unwrap()
    );
    if pl.code != session.get::<String>("captcha").unwrap().unwrap() {
        return HttpResponse::build(StatusCode::BAD_REQUEST)
            .content_type(ContentType::plaintext())
            .body("You are a bot");
    }
    session.remove("captcha");
    let argon2 = Argon2::default();
    let user_agent = req
        .headers()
        .get("user-agent")
        .unwrap()
        .to_str()
        .ok()
        .unwrap();
    // println!("USER AGENT {}", user_agent.unwrap());
    let browser = ua_parser.parse_product(user_agent);
    let os = ua_parser.parse_os(user_agent);
    let device = ua_parser.parse_device(user_agent);
    println!(
        "User Agents\nProduct {:#?}\nOs {:#?}\nDevice {:#?}",
        browser, os, device
    );
    match db::login::get_user_and_password(pl.email, pool.as_ref()).await {
        Ok((user_id, password)) => {
            if argon2
                .verify_password(
                    pl.password.as_bytes(),
                    &PasswordHash::new(&password).unwrap(),
                )
                .is_ok()
            {
                let uag = UserAgent {
                    os: Some(format!(
                        "{} {} {}",
                        os.name.unwrap(),
                        os.major.unwrap(),
                        os.minor.unwrap()
                    )),
                    browser: Some(format!(
                        "{} {} {}",
                        browser.name.unwrap(),
                        browser.major.unwrap(),
                        browser.minor.unwrap()
                    )),
                    device: Some(format!(
                        "{} {} {}",
                        device.name.unwrap(),
                        device.model.unwrap(),
                        device.brand.unwrap()
                    )),
                    original: user_agent.to_string(),
                };
                match db::login::create_session(user_id, pool.as_ref(), uag).await {
                    Ok(session_id) => {
                        Identity::login(
                            &req.extensions(),
                            json!({
                                "user_id": user_id,
                                "session_id": session_id.to_string()
                            })
                            .to_string(),
                        )
                        .unwrap();
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
