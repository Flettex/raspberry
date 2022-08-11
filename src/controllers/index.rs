use actix_web::{
    Responder,
    HttpResponse,
    web,
};
use serde_json;

use crate::server;
use crate::messages::{
    MessageTypes,
    MessageCreateType
};
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
    srv.send(MessageTypes::MessageCreate(MessageCreateType{content: serde_json::to_string(&json).unwrap(), room: json.room})).await;
    HttpResponse::Ok()
    // Ok(HttpResponse::Ok().content_type("text/plain").body("Test"))
}