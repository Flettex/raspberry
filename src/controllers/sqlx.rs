use std::str;

use actix_web::{web, HttpResponse, Responder};
use futures::StreamExt;
use sqlx::{
    postgres::PgTypeInfo,
    types::{
        chrono::{DateTime, NaiveDateTime, Utc},
        Uuid,
    },
    Column, PgPool, Row, TypeInfo,
};

const POSTGRES_EPOCH: i64 = 946702800;

fn get_val(item: &[u8], info: &PgTypeInfo) -> String {
    println!("{}", info.name());
    // postgres dude why is it [110, 117, 108, 108]
    if item == [110, 117, 108, 108] {
        return "NULL".to_string();
    } else if info.name() == "INT8" {
        return i64::from_be_bytes(item.try_into().unwrap()).to_string();
    } else if info.name() == "INT4" {
        // shouldn't be possible anymore since production DB is stupid
        // but in case..
        return i32::from_be_bytes(item.try_into().unwrap()).to_string();
    } else if info.name() == "UUID" {
        return Uuid::from_slice(item).unwrap().to_string();
    } else if info.name() == "TIMESTAMP" {
        let timestamp = i64::from_be_bytes(item.try_into().unwrap()) + POSTGRES_EPOCH * 1000000;
        let naive = NaiveDateTime::from_timestamp_opt(timestamp / 1000000, 0).unwrap();
        let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
        return datetime.format("%Y-%m-%d %H:%M:%S").to_string();
    } else if info.name() == "BOOL" {
        if item[0] == 1 {
            return "true".to_string();
        } else {
            return "false".to_string();
        }
    }
    // TEXT and varchar
    match str::from_utf8(item) {
        Ok(s) => s.to_string(),
        Err(_) => {
            format!("{:?}", item)
        }
    }
}

pub async fn post(body: web::Bytes, pool: web::Data<PgPool>) -> impl Responder {
    HttpResponse::Ok().body(
        sqlx::query(str::from_utf8(&body).unwrap())
            .fetch(pool.as_ref())
            .fold("".to_string(), |acc, row| async move {
                format!(
                    "{}Row\n{}\n",
                    acc,
                    row.as_ref()
                        .unwrap()
                        .columns()
                        .iter()
                        .map(|col| {
                            println!("{:?}", col);
                            format!(
                                "{}: {}",
                                col.name(),
                                get_val(
                                    row.as_ref()
                                        .unwrap()
                                        .try_get_raw(col.ordinal())
                                        .unwrap()
                                        .as_bytes()
                                        .unwrap_or("null".as_bytes()),
                                    col.type_info()
                                )
                            )
                        })
                        .collect::<Vec<String>>()
                        .join("\n")
                )
            })
            .await,
    )
}

// HttpResponse::Ok().body(sqlx::query(str::from_utf8(&body).unwrap()).fetch(pool.as_ref()).fold("".to_string(), |acc, row| async move {format!("{}Row\n{}\n", acc, row.as_ref().unwrap().columns().iter().map(|col| {println!("{:?}", col); format!("{}: {}", col.name(), get_val(row.as_ref().unwrap().try_get_raw(col.ordinal()).unwrap() .as_bytes().unwrap_or("null".as_bytes()),col.type_info()))}).collect::<Vec<String>>().join("\n"))}).await)
