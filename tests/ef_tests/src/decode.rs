use crate::error::Error;

use hex::FromHex;
use primitive_types::H160;
use serde::{Deserialize, Deserializer};
use std::fs;
use std::path::Path;

pub fn json_decode<T: serde::de::DeserializeOwned>(string: &str) -> Result<T, Error> {
    serde_json::from_str(string).map_err(|e| Error::FailedToParseTest(format!("{:?}", e)))
}

pub fn json_decode_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, Error> {
    fs::read_to_string(path)
        .map_err(|e| {
            Error::FailedToParseTest(format!("Unable to load {}: {:?}", path.display(), e))
        })
        .and_then(|s| {
            let v: Result<serde_json::Value, serde_json::Error> = serde_json::from_str(s.as_str());

            match v {
                Ok(v) => match v {
                    serde_json::Value::Object(o) => {
                        let mut s = Ok(String::new());
                        for (_, v) in o {
                            s = match v {
                                serde_json::Value::Object(o) => {
                                    let r = serde_json::to_string(&o).unwrap();
                                    Ok(r)
                                }
                                _ => Err(Error::FailedToParseTest("bad!".to_owned())),
                            };
                        }
                        s
                    }
                    _ => Err(Error::FailedToParseTest("bad!".to_owned())),
                },
                Err(_) => Err(Error::FailedToParseTest("bad".to_owned())),
            }
        })
        .and_then(|s| json_decode(&s))
}

/// Deserializes a lowercase hex string to a `Vec<u8>`.
pub fn from_hex_to_buffer<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    String::deserialize(deserializer).and_then(|string| {
        let without_prefix = string.trim_start_matches("0x");
        hex::decode(without_prefix).map_err(|err| Error::custom(err.to_string()))
    })
}

/// Deserializes a lowercase hex string to a `Vec<u8>`.
pub fn from_hex_to_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    String::deserialize(deserializer).and_then(|string| {
        let without_prefix = string.trim_start_matches("0x");
        u64::from_str_radix(without_prefix, 16).map_err(|err| Error::custom(err.to_string()))
    })
}
