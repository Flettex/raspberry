use sqlx::types::Uuid;
use serde::{self, Deserialize, Deserializer, Serializer};

pub fn deserialize<'de, D>(deserializer: D) -> Result<Uuid, D::Error>
where
    D: Deserializer<'de>,
{
    let uid: String = Deserialize::deserialize(deserializer)?;
    Ok(Uuid::parse_str(&uid).unwrap())
}

pub fn serialize<S>(
    uid: &Uuid,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&uid.to_string())
}