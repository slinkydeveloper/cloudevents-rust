extern crate serde_json;

use crate::{Event, PayloadReader, PayloadWriter, PayloadResult, Payload};

impl PayloadWriter<serde_json::Value, serde_json::Error> for Event {
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

impl PayloadReader<serde_json::Value, serde_json::Error> for Event {
    fn read_content_type_and_payload(
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
