use std::{
    env,
    sync::{
        atomic::{AtomicUsize},
        Arc,
    },
};

use actix_identity::IdentityMiddleware;
use actix_session::{storage::CookieSessionStore, SessionMiddleware, config::{CookieContentSecurity, PersistentSession}};
use actix_web::{
    web,
    App,
    HttpServer,
    middleware::Logger,
    cookie::{SameSite, Key, time::Duration},
};
use actix_http::header;
use actix_cors::Cors;
use clokwerk::{Scheduler, TimeUnits};

use chrono::offset::Utc;

use sqlx::postgres::PgPool;

use self::server::Chat;

// self use
mod controllers;
mod server;
mod session;
mod test;

// for controllers
mod db;
// mod session;
// test views for debugging purposes...
mod html;
// serde formatting date, uuid fields in structs
mod format;
// messages for server and sessions
mod messages;

const IS_DEV: bool = option_env!("RAILWAY_STATIC_URL").is_none();

const EMAIL_PASSWORD: &str = env!("EMAIL_PASSWORD");

const PLACEHOLDER_UUID: &str = "5fe9d2ab-2174-4a30-8245-cc5de2563dce";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let app_state = Arc::new(AtomicUsize::new(0));

    log::info!("{}", &env::var("DATABASE_URL").unwrap_or("postgres://postgres:1234@localhost:5432/flettex".to_string()));

    let pool: PgPool = PgPool::connect(
        &env::var("DATABASE_URL")
            .unwrap_or("postgres://postgres:1234@localhost:5432/flettex".to_string()),
    )
    .await
    .expect("Failed to create pool");

    let pool2 = pool.clone();

    let pool3 = pool.clone();

    let mut scheduler = Scheduler::with_tz(Utc);

    actix_web::rt::spawn(async move {
        log::info!("WIPING SESSIONS");
        sqlx::query!(
            r#"
DELETE FROM user_sessions WHERE last_login < (NOW() - INTERVAL '7 days')
            "#
        ).execute(&pool3).await.unwrap();
    });

    scheduler.every(1.days()).at("00:00").run(move || {
        let pool4 = pool2.clone();
        actix_web::rt::spawn(async move {
            log::info!("WIPING SESSIONS");
            sqlx::query!(
                r#"
DELETE FROM user_sessions WHERE last_login < (NOW() - INTERVAL '7 days')
                "#
            ).execute(&pool4).await.unwrap();
        });
    });

    // db::start::get_all_channel_names(&pool3).await.unwrap()
    let server = Chat::new(app_state.clone(), vec![]);

    // let is_dev = env::var("RAILWAY_STATIC_URL").is_err();

    log::info!(
        "{}",
        format!(
            "starting HTTP server at {}",
            if IS_DEV {
                "http://localhost:8080"
            } else {
                "production url"
            }
        )
    );
    
    HttpServer::new(move || {
        // log::info!("{}", env::var("SECRET_KEY").unwrap());
        let mut key: Vec<u8> = env::var("SECRET_KEY").unwrap().replace("'", "").split(",").collect::<Vec<&str>>().iter().map(|x| x.parse::<u8>().unwrap()).collect();
        key.extend(key.clone().iter().rev());
        let secret_key = Key::from(&key);
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_origin_fn(|origin, _req_head| {
                origin.as_bytes().starts_with(b"https://pineapple-deploy.vercel.app") || origin.as_bytes().starts_with(b"http://localhost")
                    || origin.as_bytes().starts_with(b"http://127.0.0.1")
                    || origin.as_bytes().starts_with(b"https://gearsgoround.com")
            })
            .supports_credentials()
            // set allowed methods list
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            // set allowed request header list
            .allowed_headers(&[header::AUTHORIZATION, header::ACCEPT, header::COOKIE])
            // add header to allowed list
            .allowed_header(header::CONTENT_TYPE)
            // set list of headers that are safe to expose
            .expose_headers(&[header::CONTENT_DISPOSITION])
            .max_age(3600);
        App::new()
            .wrap(cors)
            .wrap(IdentityMiddleware::default())
            .wrap(
                SessionMiddleware::builder(
                    CookieSessionStore::default(),
                    secret_key.clone()
                )
                .cookie_name("auth-cookie".to_string())
                .cookie_same_site(if IS_DEV {SameSite::Lax} else {SameSite::None})
                .cookie_http_only(true)
                .cookie_secure(if IS_DEV {false} else {true})
                .cookie_content_security(CookieContentSecurity::Private)
                .session_lifecycle(PersistentSession::default().session_ttl(Duration::days(7)))
                .build()

            )
            .app_data(web::Data::from(app_state.clone()))
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(server.clone()))
            .configure(controllers::config)
            .wrap(Logger::default())
    })
    .workers(2)
    .bind((if IS_DEV { "127.0.0.1" } else { "0.0.0.0" }, 8080))?
    .run()
    .await
}
