use actix_session::Session;
use actix_web::{http::header::ContentType, services, web, HttpResponse};
use utoipa::OpenApi;
use utoipa_swagger_ui::{SwaggerUi, Url};
// use std::collections::HashMap;
use captcha_rs::CaptchaBuilder;

pub mod admin;
pub mod channels;
pub mod count;
pub mod default;
pub mod discord;
pub mod extractor;
pub mod guilds;
pub mod index;
pub mod login;
pub mod logout;
pub mod samesite;
pub mod signup;
pub mod sqlx;
pub mod verify;
pub mod ws;
// use self::admin::format_html;
use crate::html;
use crate::server::{ClientEvent, LoginEvent, SignUpEvent};
use crate::IS_DEV;

macro_rules! view {
    ( $path: expr, $content: expr ) => {
        web::resource($path).route(web::get().to(|| async {
            HttpResponse::Ok()
                .content_type(ContentType::html())
                .body($content)
        }))
    };
    ( $path: expr, $content: expr, $useless: expr) => {
        web::resource($path).route(web::get().to(|session: Session| async move {
            let captcha = CaptchaBuilder::new()
                .length(5)
                .width(130)
                .height(50)
                .dark_mode(false)
                .complexity(5) // min: 1, max: 10
                .build();
            session.insert("captcha", captcha.text).unwrap();
            let replacements: HashMap<&str, String> =
                HashMap::from_iter([("captcha", captcha.to_base64())]);
            HttpResponse::Ok()
                .content_type(ContentType::html())
                .body(format_html($content, replacements))
        }))
    };
}

pub fn config(cfg: &mut web::ServiceConfig) {
    #[derive(OpenApi)]
    #[openapi(
        paths(index::post, login::post, signup::post),
        components(schemas(LoginEvent, ClientEvent, SignUpEvent))
    )]
    struct ApiDoc;

    // dev only

    if IS_DEV {
        cfg.service(web::resource("/admin").route(web::get().to(admin::get)))
            .service(web::resource("/sqlx").route(web::post().to(sqlx::post)));
    }

    // views

    cfg.service(services![
        view!("/", html::INDEX).route(web::post().to(index::post)),
        // view!("/chat", html::CHAT),
        // view!("/logout", html::LOGOUT),
        // view!("/verify", html::VERIFY)
    ]);

    cfg.service(web::resource("/discord").route(web::get().to(discord::get)));

    // controllers

    cfg.service(web::resource("/health").route(web::get().to(HttpResponse::Ok)))
        .service(
            web::resource("/signup")
                .route(web::get().to(|session: Session| async move {
                    let captcha = CaptchaBuilder::new()
                        .length(5)
                        .width(130)
                        .height(50)
                        .dark_mode(false)
                        .complexity(5) // min: 1, max: 10
                        .build();
                    session.insert("captcha", captcha.text.clone()).unwrap();
                    HttpResponse::Ok()
                        .content_type(ContentType::plaintext())
                        .body(captcha.to_base64())
                }))
                .route(web::post().to(signup::post)),
        )
        .service(
            web::resource("/login")
                .route(web::get().to(|session: Session| async move {
                    let captcha = CaptchaBuilder::new()
                        .length(5)
                        .width(130)
                        .height(50)
                        .dark_mode(false)
                        .complexity(5) // min: 1, max: 10
                        .build();
                    session.insert("captcha", captcha.text.clone()).unwrap();
                    HttpResponse::Ok()
                        .content_type(ContentType::plaintext())
                        .body(captcha.to_base64())
                }))
                .route(web::post().to(login::post)),
        )
        .service(web::resource("/logout").route(web::delete().to(logout::delete)))
        .service(web::resource("verify").route(web::post().to(verify::post)))
        .service(web::resource("/count").route(web::get().to(count::get)))
        .service(web::resource("/samesite").route(web::get().to(samesite::get)))
        .service(web::resource("/ws").route(web::get().to(ws::get)))
        .service(web::resource("/channels/{channel_id}").route(web::get().to(channels::get)))
        .service(web::resource("/guilds/{guild_id}").route(web::get().to(guilds::get)))
        .service(
            // default page
            web::scope("")
                .service(SwaggerUi::new("/swagger-ui/{_:.*}").urls(vec![(
                    Url::new("/", "/api-doc/openapi.json"),
                    ApiDoc::openapi(),
                )]))
                .default_service(web::to(default::all)),
        );
}
