use serde::{self, Deserialize, Deserializer, Serializer};
use sqlx::types::chrono::NaiveDateTime;
// de::Error

// const FORMAT: &str = "%+";
// const FORMAT: &str = "%Y-%m-%d %H:%M:%S";

#[allow(dead_code)]
pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let time: i64 = Deserialize::deserialize(deserializer)?;
    Ok(NaiveDateTime::from_timestamp_opt(time, 0).unwrap())
}

#[allow(dead_code)]
pub fn serialize<S>(date: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.collect_str(&date.timestamp())
}
