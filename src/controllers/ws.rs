use std::{
    sync::Arc,
    time::Instant,
    collections::HashSet
};

use actix_web::{
    web,
    HttpRequest,
    HttpResponse,
    Error
};
use actix_identity::Identity;

use serde::{Deserialize};

use tokio::sync::Mutex;
use futures::future;

use sqlx::PgPool;

use crate::{
    server::{
        self,
        // MessageTypes,
        // MessageCreateType,
        AuthCookie
    },
    session::WsChatSession,
    PLACEHOLDER_UUID,
    db
};

#[derive(Deserialize)]
pub struct WsQuery {
    pub recv_type: Option<String>
}

pub async fn get(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<server::Chat>,
    pool: web::Data<PgPool>,
    id: Option<Identity>,
    query: web::Query<WsQuery>
) -> Result<HttpResponse, Error> {
    if let Some(session_id) = id {
        let recv_type = match query.into_inner().recv_type {
            Some(t) => {
                if t == "json".to_string() {
                    "json".to_string()
                } else if t == "cbor".to_string() {
                    "cbor".to_string()
                } else {
                    "cbor".to_string()
                }
            }
            None => "cbor".to_string()
        };
        log::info!("recv type: {}", recv_type);
        let (response, session, stream) = actix_ws::handle(&req, stream)?;
        let session_cookie: AuthCookie = serde_json::from_str(&session_id.id().unwrap()).unwrap();
        log::info!("Inserted session");
        let alive = Arc::new(Mutex::new(Instant::now()));
        println!("{}", session_cookie.session_id.clone());
        match db::ws_session::get_user_by_session_id(session_cookie.session_id.clone(), pool.as_ref()).await {
            Ok(user) => {
                actix_web::rt::spawn(async move {
                    let chat_session = WsChatSession {
                        user: user.clone(),
                        rooms: Arc::new(Mutex::new(HashSet::from([PLACEHOLDER_UUID.to_owned()]))),
                        srv: srv.as_ref().clone(),
                        pool: pool.as_ref().clone(),
                        alive,
                        session,
                        session_id: session_cookie.session_id,
                        recv_type
                        // stream: Arc::new(Mutex::new(stream))
                    };
                    srv.insert_session(user.id as usize, chat_session.clone()).await;
                    future::join(chat_session.hb(), chat_session.start(stream)).await;
                });
                log::info!("Spawned");
            },
            Err(_err) => {
                println!("{:?}", _err);
                let _ = session.close(None).await;
            }
        };

        Ok(response)
    } else {
        Ok(HttpResponse::Ok().finish())
    }
}