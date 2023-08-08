use actix_identity::Identity;
use actix_web::{
    // http::{header::ContentType, StatusCode},
    web,
    HttpResponse,
};
// use actix_session::Session;

use sqlx::postgres::PgPool;
use sqlx::types::Uuid;

use crate::db;
use utoipa;

#[utoipa::path(
    post,
    path = "/channels/{channel_id}",
    responses(
        (status = 200, description = "Successful Response", body = String),
    )
)]
pub async fn get(
    pool: web::Data<PgPool>,
    id: Option<Identity>,
    path: web::Path<(Uuid,)>,
) -> HttpResponse {
    if id.is_some() {
        return HttpResponse::Ok().finish();
    }
    let pl = path.into_inner();
    HttpResponse::Ok().json(db::channels::get_channel(pl.0, &pool).await.unwrap())
}
