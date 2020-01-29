extern crate chrono;
extern crate hostname;
extern crate serde;
extern crate serde_json;
extern crate uuid;

#[macro_use]
extern crate derive_builder;

pub mod http;

use chrono::{DateTime, FixedOffset};
use hostname::get_hostname;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use uuid::Uuid;

const DEFAULT_TYPE: &str = "generated.cloudevents-sdk";
const DEFAULT_SOURCE: &str = "cloudevents.io";

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum SpecVersion {
    #[serde(rename = "0.2")]
    V02,
    #[serde(rename = "0.3")]
    V03,
    #[serde(rename = "1.0")]
    V10,
}

impl fmt::Display for SpecVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpecVersion::V02 => write!(f, "0.2"),
            SpecVersion::V03 => write!(f, "0.3"),
            SpecVersion::V10 => write!(f, "1.0"),
        }
    }
}

impl TryFrom<String> for SpecVersion {
    type Error = String;

    fn try_from(value: String) -> Result<Self, String> {
        match value.as_str() {
            "0.2" => Ok(SpecVersion::V02),
            "0.3" => Ok(SpecVersion::V03),
            "1.0" => Ok(SpecVersion::V10),
            _ => Err(format!("Invalid specversion {}", value)),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Payload {
    #[serde(rename = "datacontenttype")]
    pub content_type: String,

    #[serde(with = "bytes_to_string")]
    pub data: Vec<u8>,
}

mod bytes_to_string {
    use serde::de::Visitor;
    use serde::{de, ser, Deserializer, Serializer};
    use std::fmt;

    pub fn serialize<S>(data: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(std::str::from_utf8(data).map_err(ser::Error::custom)?)
    }

    struct BytesBufferVisitor;

    impl<'de> Visitor<'de> for BytesBufferVisitor {
        type Value = Vec<u8>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string")
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value.into_bytes())
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(BytesBufferVisitor {})
    }
}

type PayloadResult<T, E> = Option<Result<T, E>>;

pub trait Writer<T: Sized, E: std::error::Error>
where
    Self: Sized + Clone,
{
    fn write_payload(&mut self, content_type: &str, value: T) -> Result<(), E>;

    fn clone_with_new_payload(&self, content_type: &str, value: T) -> Result<Self, E> {
        let mut new = self.clone();
        new.write_payload(content_type, value)?;
        Ok(new)
    }
}

pub trait Reader<T: Sized, E: std::error::Error> {
    fn read_payload(&self) -> PayloadResult<T, E> {
        self.read_payload_with_content_type()
            .map(|res| res.map(|(_, val)| val))
    }

    fn read_payload_with_content_type(&self) -> PayloadResult<(String, T), E>;
}

pub trait Mapper<T: Sized, E: std::error::Error, F: Fn(T) -> T>
where
    Self: Sized,
{
    fn map_payload(&self, f: F) -> Result<Self, E>;
}

impl<
        T: Sized,
        E: std::error::Error,
        F: Fn(T) -> T,
        S: Writer<T, E> + Reader<T, E> + Clone + Sized,
    > Mapper<T, E, F> for S
{
    fn map_payload(&self, f: F) -> Result<Self, E> {
        if let Some(Ok((ct, value))) = self.read_payload_with_content_type() {
            let mut new = self.clone();
            new.write_payload(&ct, f(value))?;
            return Ok(new);
        } else {
            return Ok(self.clone());
        }
    }
}

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

impl Writer<serde_json::Value, serde_json::Error> for Event {
    fn write_payload(
        &mut self,
        content_type: &str,
        value: serde_json::Value,
    ) -> Result<(), serde_json::Error> {
        let serialized = serde_json::to_vec(&value)?;
        self.payload = Some(Payload {
            content_type: String::from(content_type),
            data: serialized,
        });
        Ok(())
    }
}

impl Reader<serde_json::Value, serde_json::Error> for Event {
    fn read_payload_with_content_type(
        &self,
    ) -> PayloadResult<(String, serde_json::Value), serde_json::Error> {
        if self.payload.is_none() {
            return None;
        }

        let p = self.payload.as_ref().unwrap();
        Some(
            serde_json::from_slice::<serde_json::Value>(&p.data[..])
                .map(|j| (p.content_type.clone(), j)),
        )
    }
}

impl Reader<serde_json::Value, serde_json::Error> for Option<Event> {
    fn read_payload_with_content_type(
        &self,
    ) -> PayloadResult<(String, serde_json::Value), serde_json::Error> {
        if let Some(r) = self {
            r.read_payload_with_content_type()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
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
