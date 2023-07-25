use actix_web::{
    http::{header::ContentType, StatusCode},
    HttpRequest, HttpResponse,
};

use crate::html;

pub async fn all(req: HttpRequest) -> HttpResponse {
    if req.method() == "GET" {
        HttpResponse::build(StatusCode::NOT_FOUND)
            .content_type(ContentType::html())
            .body(html::DEFAULT)
    } else {
        HttpResponse::NotFound().finish()
    }
}
