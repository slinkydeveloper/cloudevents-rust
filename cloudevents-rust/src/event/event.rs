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

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use crate::event::SpecVersion;
    use crate::event::Payload;
    use serde_json::json;

    #[test]
    fn test_serialize_no_payload_no_extensions() {
        let expected_id = "A234-1234-1234";
        let expected_spec_version = SpecVersion::V10;
        let expected_type = "com.github.pull.create";
        let expected_source = "https://github.com/cloudevents/spec/pull";
        let expected_subject = "123";
        let expected_time = DateTime::parse_from_rfc3339("2018-04-05T17:31:00Z").unwrap();

        let j = json!({
            "id" : expected_id,
            "specversion" : expected_spec_version.to_string(),
            "type" : expected_type,
            "source" : expected_source,
            "subject" : expected_subject,
            "time" : expected_time.to_rfc3339()
        });

        let v: Event = serde_json::from_value(j).unwrap();

        assert_eq!(v.id, expected_id);
        assert_eq!(v.spec_version, expected_spec_version);
        assert_eq!(v.event_type, expected_type);
        assert_eq!(v.source, expected_source);
        assert_eq!(v.subject, Some(expected_subject.to_string()));
        assert_eq!(v.time, Some(expected_time));
        assert_eq!(v.payload, None);
        assert!(v.extensions.is_empty());
    }

    #[test]
    fn test_serialize_no_payload_with_extensions() {
        let expected_id = "A234-1234-1234";
        let expected_spec_version = SpecVersion::V10;
        let expected_type = "com.github.pull.create";
        let expected_source = "https://github.com/cloudevents/spec/pull";
        let expected_subject = "123";
        let expected_time = DateTime::parse_from_rfc3339("2018-04-05T17:31:00Z").unwrap();
        let expected_stuff = "aaa";

        let j = json!({
            "id" : expected_id,
            "specversion" : expected_spec_version.to_string(),
            "type" : expected_type,
            "source" : expected_source,
            "subject" : expected_subject,
            "time" : expected_time.to_rfc3339(),
            "stuff": expected_stuff
        });

        let v: Event = serde_json::from_value(j).unwrap();

        assert_eq!(v.id, expected_id);
        assert_eq!(v.spec_version, expected_spec_version);
        assert_eq!(v.event_type, expected_type);
        assert_eq!(v.source, expected_source);
        assert_eq!(v.subject, Some(expected_subject.to_string()));
        assert_eq!(v.time, Some(expected_time));
        assert_eq!(v.payload, None);
        assert!(!v.extensions.is_empty());
        assert_eq!(v.extensions.get("stuff"), Some(&expected_stuff.to_string()));
    }

    #[test]
    fn test_serialize_with_payload_with_extensions() {
        let expected_id = "A234-1234-1234";
        let expected_spec_version = SpecVersion::V10;
        let expected_type = "com.github.pull.create";
        let expected_source = "https://github.com/cloudevents/spec/pull";
        let expected_subject = "123";
        let expected_time = DateTime::parse_from_rfc3339("2018-04-05T17:31:00Z").unwrap();
        let expected_stuff = "aaa";
        let expected_content_type = "application/json";
        let expected_data = r#"{"hello":"world"}"#;

        let j = json!({
            "id" : expected_id,
            "specversion" : expected_spec_version.to_string(),
            "type" : expected_type,
            "source" : expected_source,
            "subject" : expected_subject,
            "time" : expected_time.to_rfc3339(),
            "stuff": expected_stuff,
            "datacontenttype": expected_content_type,
            "data": expected_data
        });

        let v: Event = serde_json::from_value(j).unwrap();

        assert_eq!(v.id, expected_id);
        assert_eq!(v.spec_version, expected_spec_version);
        assert_eq!(v.event_type, expected_type);
        assert_eq!(v.source, expected_source);
        assert_eq!(v.subject, Some(expected_subject.to_string()));
        assert_eq!(v.time, Some(expected_time));
        assert_eq!(
            v.payload,
            Some(Payload {
                content_type: expected_content_type.to_string(),
                data: expected_data.as_bytes().into()
            })
        );
        assert!(!v.extensions.is_empty());
        assert_eq!(v.extensions.get("stuff"), Some(&expected_stuff.to_string()));
    }
}

