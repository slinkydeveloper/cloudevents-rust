extern crate chrono;
extern crate hostname;
extern crate serde;
extern crate serde_json;
extern crate uuid;

use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum SpecVersion {
    #[serde(rename = "1.0")]
    V10,
}

impl fmt::Display for SpecVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpecVersion::V10 => write!(f, "1.0"),
        }
    }
}

impl TryFrom<String> for SpecVersion {
    type Error = String;

    fn try_from(value: String) -> Result<Self, String> {
        match value.as_str() {
            "1.0" => Ok(SpecVersion::V10),
            _ => Err(format!("Invalid specversion {}", value)),
        }
    }
}
