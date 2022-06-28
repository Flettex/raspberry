use actix_web::{
    Responder,
    HttpResponse,
    web,
};
use serde_json;

use crate::server::{
    self,
    MessageTypes,
    MessageCreateType,
};

pub async fn post(
    body: web::Json<server::ClientEvent>,
    srv: web::Data<server::Chat>,
) -> impl Responder /* Result<HttpResponse, Error> */ {
    // println!("Event: {}", body.event_name);
    srv.send(MessageTypes::MessageCreate(MessageCreateType{content: serde_json::to_string(&body.into_inner()).unwrap()})).await;
    HttpResponse::Ok()
    // Ok(HttpResponse::Ok().content_type("text/plain").body("Test"))
}