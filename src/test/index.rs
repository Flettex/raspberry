#[cfg(test)]
mod tests {
    use crate::html;
    use actix_http::Request;
    use actix_web::{
        http::header::ContentType,
        // http::{self, header::ContentType},
        test,
        web,
        App,
        HttpResponse,
    };

    macro_rules! quickget {
        ($content_type: expr, $body: expr) => {
            web::get().to(|| async { HttpResponse::Ok().content_type($content_type).body($body) })
        };
    }

    fn req(uri: &str) -> Request {
        test::TestRequest::get().uri(uri).to_request()
    }

    #[actix_web::test]
    async fn test_index_get() {
        let app =
            test::init_service(App::new().route("/", quickget!(ContentType::html(), html::INDEX)))
                .await;

        // body check
        let resp = test::call_and_read_body(&app, req("/")).await;
        assert_eq!(resp, web::Bytes::from_static(html::INDEX.as_bytes()));

        // status check
        let resp = test::call_service(&app, req("/")).await;
        assert!(resp.status().is_success());
    }

    // #[actix_web::test]
    // async fn test_index_ok() {
    //     // let req = test::TestRequest::default()
    //     //     .insert_header(ContentType::plaintext())
    //     //     .to_http_request();
    //     let resp = index::get().await;
    //     assert_eq!(resp.status(), http::StatusCode::OK);
    // }

    // #[actix_web::test]
    // async fn test_index_not_ok() {
    //     let req = test::TestRequest::default().to_http_request();
    //     let resp = index(req).await;
    //     assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
    // }
}
