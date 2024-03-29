use actix_identity::Identity;
use actix_web::{
    http::{header::ContentType, StatusCode},
    web, HttpMessage, HttpRequest, HttpResponse,
};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use rand::Rng;
use std::{sync::Arc, borrow::Cow};

use actix_session::Session;

use crate::db::signup::{create_password, create_user};
use crate::{db::signup::UserAgent, server, EMAIL_PASSWORD};
use serde_json::json;
use sqlx::postgres::PgPool;
use utoipa;

use user_agent_parser::UserAgentParser;

#[utoipa::path(
    post,
    path = "/signup",
    responses(
        (status = 200, description = "Successful Response", body = String),
        (status = 400, description = "Duplicate email or username or the database failed to create a session", body = String)
    ),
    request_body(content = SignUpEvent, description = "user email, user password, username", content_type = "application/json")
)]
pub async fn post(
    body: web::Json<server::SignUpEvent>,
    pool: web::Data<PgPool>,
    id: Option<Identity>,
    req: HttpRequest,
    session: Session,
    ua_parser: web::Data<Arc<UserAgentParser>>,
) -> HttpResponse {
    let user_agent = req
        .headers()
        .get("user-agent")
        .unwrap()
        .to_str()
        .ok()
        .unwrap_or("");
    // println!("USER AGENT {}", user_agent.unwrap());
    let browser = ua_parser.parse_product(user_agent);
    let os = ua_parser.parse_os(user_agent);
    let device = ua_parser.parse_device(user_agent);
    println!(
        "User Agents\nProduct {:#?}\nOs {:#?}\nDevice {:#?}",
        browser, os, device
    );
    if id.is_some() {
        return HttpResponse::Ok().finish();
    }
    let pl = body.into_inner();
    if pl.code != session.get::<String>("captcha").unwrap().unwrap() {
        return HttpResponse::build(StatusCode::BAD_REQUEST)
            .content_type(ContentType::plaintext())
            .body("You are a bot");
    }
    session.remove("captcha");
    match create_password(pl.password) {
        Ok(password_hash) => {
            // generate random
            let code = rand::thread_rng().gen_range(100000..999999);
            let uag = UserAgent {
                os: Some(format!(
                    "{} {} {}",
                    os.name.unwrap_or(Cow::Borrowed("")),
                    os.major.unwrap_or(Cow::Borrowed("")),
                    os.minor.unwrap_or(Cow::Borrowed(""))
                )),
                browser: Some(format!(
                    "{} {} {}",
                    browser.name.unwrap_or(Cow::Borrowed("")),
                    browser.major.unwrap_or(Cow::Borrowed("")),
                    browser.minor.unwrap_or(Cow::Borrowed(""))
                )),
                device: Some(format!(
                    "{} {} {}",
                    device.name.unwrap_or(Cow::Borrowed("")),
                    device.model.unwrap_or(Cow::Borrowed("")),
                    device.brand.unwrap_or(Cow::Borrowed(""))
                )),
                original: user_agent.to_string(),
            };
            match create_user(
                pl.username,
                pl.email.clone(),
                code.into(),
                password_hash,
                uag,
                pool.as_ref(),
            )
            .await
            {
                Ok((session_id, user_id)) => {
                    Identity::login(
                        &req.extensions(),
                        json!({
                            "user_id": user_id,
                            "session_id": session_id.to_string()
                        })
                        .to_string(),
                    )
                    .unwrap();
                    let email = Message::builder()
                        .from(
                            "Balls Eater <capitalismdiscordbot@gmail.com>"
                                .parse()
                                .unwrap(),
                        )
                        .reply_to("Ballz <capitalismdiscordbot@gmail.com>".parse().unwrap())
                        .to(format!("<{}>", pl.email.clone()).parse().unwrap())
                        .subject("Thanks for registering for flettex")
                        .body(format!("Your verification code is: {}", code))
                        .unwrap();

                    println!("CODE IS {}", code);

                    let creds = Credentials::new(
                        "capitalismdiscordbot@gmail.com".to_string(),
                        EMAIL_PASSWORD.to_string(),
                    );

                    // Open a remote connection to gmail
                    let mailer = SmtpTransport::relay("smtp.gmail.com")
                        .unwrap()
                        .credentials(creds)
                        .build();

                    // Send the email
                    match mailer.send(&email) {
                        Ok(_) => println!("Email sent successfully!"),
                        Err(e) => println!("Could not send email: {:?}", e),
                    }
                    HttpResponse::Ok().finish()
                }
                Err(err) => match err {
                    sqlx::Error::Database(err) => {
                        // duplicate error
                        if err.code() == Some(std::borrow::Cow::Borrowed("25565")) {
                            HttpResponse::build(StatusCode::BAD_REQUEST)
                                .content_type(ContentType::plaintext())
                                .body("Bad request, duplicate")
                        } else {
                            println!("{}", err);
                            if let Some(errcode) = err.code() {
                                HttpResponse::build(StatusCode::BAD_REQUEST)
                                    .content_type(ContentType::plaintext())
                                    .body(format!("Bad request, database error code {}", errcode))
                            } else {
                                HttpResponse::build(StatusCode::BAD_REQUEST)
                                    .content_type(ContentType::plaintext())
                                    .body("Bad request, database error, code unknown")
                            }
                        }
                    }
                    _ => HttpResponse::build(StatusCode::BAD_REQUEST)
                        .content_type(ContentType::plaintext())
                        .body("Bad request, non-database error"),
                },
            }
        }
        Err(_) => HttpResponse::build(StatusCode::BAD_REQUEST)
            .content_type(ContentType::plaintext())
            .body("Unable to hash password"),
    }
}
