use crate::Event;

pub const CE_ID_HEADER: &str = "ce-id";
pub const CE_TYPE_HEADER: &str = "ce-type";
pub const CE_SOURCE_HEADER: &str = "ce-source";
pub const CE_SPECVERSION_HEADER: &str = "ce-specversion";

pub const CE_SUBJECT_HEADER: &str = "ce-subject";
pub const CE_TIME_HEADER: &str = "ce-time";

pub const CE_JSON_CONTENT_TYPE: &str = "application/cloudevents+json";
pub const CE_BATCH_JSON_CONTENT_TYPE: &str = "application/cloudevents-batch+json";

pub enum HttpCloudEvent {
    Binary(Option<Event>),
    Structured(Option<Event>),
    Batch(Vec<Event>)
}
