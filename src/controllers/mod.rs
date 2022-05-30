use actix_web::{
    web,
    services,
    HttpResponse,
    http::header::ContentType,
};

pub mod login;
pub mod logout;
pub mod signup;
pub mod index;
pub mod ws;
pub mod count;
pub mod default;
use crate::html;

macro_rules! view {
    ( $path: expr, $content: expr ) => {
        web::resource($path)
            .route(web::get().to(|| async {
                HttpResponse::Ok()
                    .content_type(ContentType::html())
                    .body($content)
            }))
    };
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        services![
            view!("/", html::INDEX)
                .route(web::post().to(index::post)),
            view!("/chat", html::CHAT),
            view!("/signup", html::SIGNUP)
                .route(web::post().to(signup::post)),
            view!("/login", html::LOGIN)
                .route(web::post().to(login::post)),
            web::resource("/count")
                .route(web::get().to(count::get)),
            web::resource("/health")
                .route(web::get().to(|| {
                    HttpResponse::Ok()
                })),
            web::resource("/ws")
                .route(web::get().to(ws::get)),
            web::resource("/logout")
                .route(web::delete().to(logout::delete)),
            // default page
            web::scope("")
                .default_service(web::to(default::all))
        ]
    );
}