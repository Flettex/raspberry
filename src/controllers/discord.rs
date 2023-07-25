use actix_web::{
    http::{header::ContentType, StatusCode},
    // HttpRequest,
    HttpResponse,
};

use crate::html;

pub async fn get() -> HttpResponse {
    HttpResponse::build(StatusCode::NOT_FOUND)
        .content_type(ContentType::html())
        .body(html::DISCORD)
}
