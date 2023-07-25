// I stole this code from someone else

use std::future::Future;
use std::pin::Pin;

use actix_http::Payload;
use actix_web::{
    error::ErrorBadRequest,
    http::header,
    web::{Form, Json},
    FromRequest, HttpRequest,
};
use serde::de::DeserializeOwned;

pub enum ValidatedForm<T, K = T> {
    Json(T),
    Form(K),
}

impl<T> ValidatedForm<T> {
    pub fn decode(self) -> T {
        let payload = match self {
            Self::Json(payload) => payload,
            Self::Form(payload) => payload,
        };
        payload
    }
}

impl<T: DeserializeOwned> FromRequest for ValidatedForm<T>
where
    T: 'static + DeserializeOwned,
{
    type Error = actix_web::error::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let req = req.clone();
        let mut pl = payload.take();

        Box::pin(async move {
            let content_type = req.headers().get(header::CONTENT_TYPE);

            if let Some(content_type) = content_type {
                let content_type = content_type.to_str().unwrap_or("").to_string();

                if content_type.starts_with("application/json") {
                    let data = Json::<T>::from_request(&req, &mut pl)
                        .await
                        .unwrap()
                        .into_inner();

                    // data.validate().unwrap();

                    return Ok(Self::Json(data));
                } else if content_type.starts_with("application/x-www-form-urlencoded") {
                    let data = Form::<T>::from_request(&req, &mut pl)
                        .await
                        .unwrap()
                        .into_inner();

                    // data.validate().unwrap();

                    return Ok(Self::Form(data));
                }
            }

            return Err(ErrorBadRequest("invalid content".to_string()));
        })
    }
}
