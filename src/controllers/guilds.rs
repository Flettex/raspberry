use actix_identity::Identity;
use actix_web::{web, HttpResponse};

use sqlx::postgres::PgPool;
use sqlx::types::Uuid;

use crate::db;
use utoipa;

#[utoipa::path(
    post,
    path = "/guilds/{guild_id}",
    responses(
        (status = 200, description = "Successful Response", body = String),
    )
)]
pub async fn get(
    pool: web::Data<PgPool>,
    id: Option<Identity>,
    path: web::Path<(Uuid,)>,
) -> HttpResponse {
    if let Some(_) = id {
        return HttpResponse::Ok().finish();
    }
    let pl = path.into_inner();
    HttpResponse::Ok().json(db::guilds::get_guild(pl.0, &pool).await.unwrap())
}
