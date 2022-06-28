use actix_web::{
    web,
    HttpResponse
};

use crate::db::ws_session;

use sqlx::postgres::PgPool;

pub async fn get(
    pool: web::Data<PgPool>,
) -> HttpResponse {
    HttpResponse::Ok().body(
        vec![
            ws_session::get_all(pool.as_ref()).await.unwrap().into_iter().map(|s| format!("{:?}", s)).collect::<Vec<String>>().join("\n"),
            ws_session::get_all_sessions(pool.as_ref()).await.unwrap().into_iter().map(|s| format!("{:?}", s)).collect::<Vec<String>>().join("\n"),
        ].join("\n\n")
    )
}