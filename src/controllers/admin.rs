use std::collections::HashMap;

use actix_web::{http::header::ContentType, web, HttpResponse};
use regex::Regex;

use crate::db::ws_session;
use crate::html;

use sqlx::postgres::PgPool;

pub fn format_html(resp: &str, replacements: HashMap<&str, String>) -> String {
    let re: Regex = Regex::new(r"\{\{(.*?)\}\}").unwrap();
    let v: Vec<&str> = re.find_iter(resp).map(|s| s.as_str()).collect();
    v.iter()
        .fold(resp.to_string(), |resp, needle| {
            resp.replace(
                needle,
                &replacements[needle
                    .strip_prefix("{{")
                    .unwrap()
                    .strip_suffix("}}")
                    .unwrap()],
            )
        })
        .replace("\n", "<br />")
}

pub async fn get(pool: web::Data<PgPool>) -> HttpResponse {
    let replacements: HashMap<&str, String> = HashMap::from_iter([
        (
            "users_data",
            ws_session::get_all(pool.as_ref())
                .await
                .unwrap()
                .into_iter()
                .map(|s| format!("{:?}", s))
                .collect::<Vec<String>>()
                .join("\n"),
        ),
        (
            "user_sessions_data",
            ws_session::get_all_sessions(pool.as_ref())
                .await
                .unwrap()
                .into_iter()
                .map(|s| format!("{:?}", s))
                .collect::<Vec<String>>()
                .join("\n"),
        ),
        (
            "guilds_data",
            ws_session::get_all_guilds(pool.as_ref())
                .await
                .unwrap()
                .into_iter()
                .map(|s| format!("{:?}", s))
                .collect::<Vec<String>>()
                .join("\n"),
        ),
        (
            "member_data",
            ws_session::get_all_members(pool.as_ref())
                .await
                .unwrap()
                .into_iter()
                .map(|s| format!("{:?}", s))
                .collect::<Vec<String>>()
                .join("\n"),
        ),
    ]);
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format_html(html::ADMIN, replacements))
}
