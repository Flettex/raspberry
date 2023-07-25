use actix_web::{web, HttpResponse, Responder};
use serde_json;

use crate::messages::{Message, MessageTypes};
use crate::server;
use utoipa;

#[utoipa::path(
    post,
    path = "/",
    responses(
        (status = 200, description = "Test", body = String)
    ),
    request_body(content = ClientEvent, description = "Testy westy", content_type = "application/json")
)]
pub async fn post(
    body: web::Json<server::ClientEvent>,
    srv: web::Data<server::Chat>,
) -> impl Responder /* Result<HttpResponse, Error> */ {
    // println!("Event: {}", body.event_name);
    let json = body.into_inner();
    srv.send(MessageTypes::MessageCreate(Message::system(
        serde_json::to_string(&json).unwrap(),
        &json.room,
        0,
    )))
    .await;
    HttpResponse::Ok()
    // Ok(HttpResponse::Ok().content_type("text/plain").body("Test"))
}
