use std::{
    sync::Arc,
    time::{Instant}
};

use actix_web::{
    web,
    HttpRequest,
    HttpResponse,
    Error
};
use actix_identity::Identity;
// use actix_ws::{Message};
// use actix_rt;
use tokio::sync::Mutex;
// use futures::StreamExt;
use futures::future;

use sqlx::PgPool;

use crate::{
    server::{
        self,
        // MessageTypes,
        // MessageCreateType,
        AuthCookie
    },
    session::{
        WsChatSession
    }
};

pub async fn get(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<server::Chat>,
    pool: web::Data<PgPool>,
    id: Identity,
) -> Result<HttpResponse, Error> {
    if let Some(session_id) = id.identity() {
        let (response, session, stream) = actix_ws::handle(&req, stream)?;
        let session_cookie: AuthCookie = serde_json::from_str(&session_id).unwrap();
        srv.insert(session_cookie.user_id.try_into().unwrap(), session.clone()).await;
        log::info!("Inserted session");
        let alive = Arc::new(Mutex::new(Instant::now()));
        
        actix_web::rt::spawn(async move {
            let chat_session = WsChatSession {
                id: session_cookie.user_id.try_into().unwrap(),
                rooms: Arc::new(Mutex::new(vec!["Main".to_owned()])),
                name: Arc::new(Mutex::new(None)),
                srv: srv.as_ref().clone(),
                pool: pool.as_ref().clone(),
                alive,
                session,
                session_id,
                stream: Arc::new(Mutex::new(stream))
            };
            future::join(chat_session.hb(), chat_session.start()).await;
        });
        log::info!("Spawned");
        Ok(response)
    } else {
        Ok(HttpResponse::Ok().finish())
    }
}