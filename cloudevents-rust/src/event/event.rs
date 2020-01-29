extern crate chrono;
extern crate hostname;
extern crate serde;
extern crate serde_json;
extern crate uuid;

use chrono::{DateTime, FixedOffset};
use hostname::get_hostname;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use crate::event::SpecVersion;
use crate::event::Payload;

const DEFAULT_TYPE: &str = "generated.cloudevents-sdk";
const DEFAULT_SOURCE: &str = "cloudevents.io";

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Builder)]
#[builder(setter(into, strip_option))]
pub struct Event {
    #[builder(default = "Uuid::new_v4().to_string()")]
    pub id: String,

    #[builder(default = "get_hostname().unwrap_or(DEFAULT_SOURCE.to_string())")]
    pub source: String,

    #[builder(default = "SpecVersion::V10")]
    #[serde(rename = "specversion")]
    pub spec_version: SpecVersion,

    #[builder(default = "DEFAULT_TYPE.to_string()")]
    #[serde(rename = "type")]
    pub event_type: String,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<DateTime<FixedOffset>>,

    #[serde(flatten)]
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<Payload>,

    #[serde(flatten)]
    #[builder(default)]
    pub extensions: HashMap<String, String>,
}

#[allow(non_snake_case)]
impl Event {
    pub fn new() -> Event {
        return EventBuilder::default().build().unwrap();
    }
}
