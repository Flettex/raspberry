use sqlx::types::chrono::{
    NaiveDateTime,
    DateTime
};
use serde::{self, Deserialize, Deserializer, Serializer, de::Error};
// const FORMAT: &str = "%+";
const FORMAT: &str = "%Y-%m-%d %H:%M:%S";

pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let time: String = Deserialize::deserialize(deserializer)?;
    Ok(DateTime::parse_from_rfc3339(&time).map_err(D::Error::custom)?.naive_utc())
}

pub fn serialize<S>(
    date: &NaiveDateTime,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.collect_str(&date.format(FORMAT))
}