use actix_web::{
    web,
    services,
    HttpResponse,
    http::header::ContentType
};
use actix_session::Session;
use utoipa::OpenApi;
use utoipa_swagger_ui::{SwaggerUi, Url};
use std::collections::HashMap;
use captcha_rs::{
    CaptchaBuilder
};

pub mod login;
pub mod logout;
pub mod signup;
pub mod index;
pub mod ws;
pub mod count;
pub mod default;
pub mod admin;
pub mod sqlx;
pub mod verify;
use self::admin::format_html;
use crate::html;
use crate::server::{
    LoginEvent,
    ClientEvent
};
use crate::IS_DEV;

macro_rules! view {
    ( $path: expr, $content: expr ) => {
        web::resource($path)
            .route(web::get().to(|| async {
                HttpResponse::Ok()
                    .content_type(ContentType::html())
                    .body($content)
            }))
    };
    ( $path: expr, $content: expr, $useless: expr) => {
        web::resource($path)
            .route(web::get().to(|session: Session| async move {
                let captcha = CaptchaBuilder::new()
                    .length(5)
                    .width(130)
                    .height(50)
                    .dark_mode(false)
                    .complexity(5) // min: 1, max: 10
                    .build();
                session.insert("captcha", captcha.text).unwrap();
                let replacements: HashMap<&str, String> = HashMap::from_iter([
                    ("captcha", captcha.base_img),
                ]);
                HttpResponse::Ok()
                    .content_type(ContentType::html())
                    .body(format_html($content, replacements))
            }))
    }
}

macro_rules! no_resource {
    () => {
        web::resource("")
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    #[derive(OpenApi)]
    #[openapi(
        handlers(
            index::post,
            login::post
        ),
        components(LoginEvent, ClientEvent)
    )]
    struct ApiDoc;

    cfg.service(
        services![
            view!("/", html::INDEX)
                .route(web::post().to(index::post)),
            view!("/chat", html::CHAT),
            view!("/signup", html::SIGNUP, true)
                .route(web::post().to(signup::post)),
            view!("/login", html::LOGIN, true)
                .route(web::post().to(login::post)),
            view!("/logout", html::LOGOUT)
                .route(web::delete().to(logout::delete)),
            view!("/verify", html::VERIFY)
                .route(web::post().to(verify::post)),
            web::resource("/count")
                .route(web::get().to(count::get)),
            web::resource("/health")
                .route(web::get().to(|| {
                    HttpResponse::Ok()
                })),
            web::resource("/ws")
                .route(web::get().to(ws::get)),
            if IS_DEV {
                web::resource("/admin")
                    .route(web::get().to(admin::get))
            } else {
                no_resource!()
            },
            if IS_DEV {
                web::resource("/sqlx")
                    .route(web::post().to(sqlx::post))
            } else {
                no_resource!()
            },
            // default page
            web::scope("")
                .service(SwaggerUi::new("/swagger-ui/{_:.*}").urls(vec![
                    (
                        Url::new("/", "/api-doc/openapi.json"),
                        ApiDoc::openapi()
                    )
                ]))
                .default_service(web::to(default::all)),
        ]
    );
}