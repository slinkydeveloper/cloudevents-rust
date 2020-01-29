extern crate chrono;
extern crate hostname;
extern crate serde;
extern crate serde_json;
extern crate uuid;

use serde::{Deserialize, Serialize};

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


pub type PayloadResult<T, E> = Option<Result<T, E>>;

pub trait PayloadWriter<T: Sized, E: std::error::Error>
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

pub trait PayloadReader<T: Sized, E: std::error::Error> {
    fn read_payload(&self) -> PayloadResult<T, E> {
        self.read_content_type_and_payload()
            .map(|res| res.map(|(_, val)| val))
    }

    fn read_content_type_and_payload(&self) -> PayloadResult<(String, T), E>;
}

pub trait PayloadMapper<T: Sized, E: std::error::Error, F: Fn(T) -> T>
    where
        Self: Sized,
{
    fn map_payload(&self, f: F) -> Result<Self, E>;
}

impl<
    T: Sized,
    E: std::error::Error,
    F: Fn(T) -> T,
    S: PayloadWriter<T, E> + PayloadReader<T, E> + Clone + Sized,
> PayloadMapper<T, E, F> for S
{
    fn map_payload(&self, f: F) -> Result<Self, E> {
        if let Some(Ok((ct, value))) = self.read_content_type_and_payload() {
            let mut new = self.clone();
            new.write_payload(&ct, f(value))?;
            return Ok(new);
        } else {
            return Ok(self.clone());
        }
    }
}
