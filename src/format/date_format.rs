use sqlx::types::chrono::{
    NaiveDateTime,
    DateTime
};
use serde::{self, Deserialize, Deserializer, Serializer, de::Error};
// const FORMAT: &str = "%+";
const FORMAT: &str = "%Y-%m-%d %H:%M:%S";

pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveDateTime>, D::Error>
where
    D: Deserializer<'de>,
{
    let time: String = Deserialize::deserialize(deserializer)?;
    Ok(Some(DateTime::parse_from_rfc3339(&time).map_err(D::Error::custom)?.naive_utc()))
}

pub fn serialize<S>(
    date: &Option<NaiveDateTime>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(date) = date {
        serializer.collect_str(&date.format(FORMAT))
    } else {
        // this just can't happen because of the way database is designed
        serializer.serialize_str("")
    }
}