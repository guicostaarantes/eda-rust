use serde::{Deserialize, Serialize};

pub struct JsonSerializerImpl;

impl JsonSerializerImpl {
    pub fn serialize<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
        serde_json::to_string(value)
    }

    pub fn deserialize<'a, T: Deserialize<'a>>(value: &'a str) -> Result<T, serde_json::Error> {
        serde_json::from_str(value)
    }
}
