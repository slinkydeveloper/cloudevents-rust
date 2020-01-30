use actix_web::http::{HeaderMap, header};
use actix_web::web::Bytes;
use actix_web::HttpRequest;
use chrono::DateTime;
use cloudevents::http;
use cloudevents::{Event, Payload};
use std::convert::TryInto;

macro_rules! unwrap_header {
    ($headers:expr, $key:expr, $meta_name:expr) => {
        $headers
            .get($key)
            .ok_or(cloudevents::http::ReaderError::InvalidMetadata{name: $meta_name.into(), reason: format!(
                "Expecting header {}",
                $key
            )})
            .and_then(|ce| {
                ce.to_str().map(|s| String::from(s)).map_err(|e| {
                    cloudevents::http::ReaderError::InvalidMetadata{name: $meta_name.into(), reason: format!(
                        "Error while parsing header {}: {}",
                        $key, e
                    )}
                })
            })
    };
}

macro_rules! unwrap_and_remove_header {
    ($headers:expr, $key:expr, $meta_name:expr) => {{
        let v = unwrap_header!($headers, $key, $meta_name);
        $headers.remove($key);
        v
    }};
}

pub struct EventReader {}

impl http::EventReader<(HttpRequest, Bytes)> for EventReader {

    // Possible cases:
    // 1. Content-type exists:
    // 1.1 If application/cloudevents+json -> parse structured
    // 1.2 If application/cloudevents-batch+json -> parse batch
    // 1.4 If other -> parse binary
    // 2. Content-type doesn't exist:
    // 2.1 If CE id header, then it's an empty payload cloud event -> parse binary
    // 2.2 If no CE header -> None
    fn read_cloud_event(r: (HttpRequest, Bytes)) -> Result<Option<http::HttpEvent>, http::ReaderError> {
        let (req, payload) = r;
        let mut headers: HeaderMap = req.headers().clone();

        if let Ok(ct) = unwrap_and_remove_header!(headers, "content-type", "datacontenttype") {
            // Payload at this point can't be None
            if payload.is_empty() {
                return Err(http::ReaderError::InvalidEncoding {
                    content_type: ct,
                    reason: "No body but content type is not null".to_string()
                });
            }

            // Try structured, batch and binary
            if ct.contains(http::CE_JSON_CONTENT_TYPE) {
                return Ok(Some(
                    parse_structured(payload)
                        .map(http::HttpEvent::Structured)?
                ));
            } else if ct.contains(http::CE_BATCH_JSON_CONTENT_TYPE) {
                return Ok(Some(
                    parse_batch(payload)
                        .map(http::HttpEvent::Batch)?
                ));
            } else if headers.contains_key(http::CE_ID_HEADER) {
                return Ok(Some(
                    parse_binary(headers, Some((ct, payload)))
                        .map(http::HttpEvent::Binary)?
                ));
            } else {
                return Err(http::ReaderError::InvalidEncoding {
                    content_type: ct,
                    reason: "Unrecognized encoding".to_string()
                });
            }
        }

        // Empty payload event
        if headers.contains_key(http::CE_ID_HEADER) {
            return Ok(Some(
                parse_binary(headers, None)
                    .map(http::HttpEvent::Binary)?
            ));
        }

        return Ok(None);
    }
}

fn parse_structured(payload: Bytes) -> Result<Event, http::ReaderError> {
    return Ok(serde_json::from_slice::<Event>(&payload)?)
}

fn parse_batch(payload: Bytes) -> Result<Vec<Event>, http::ReaderError> {
    return Ok(serde_json::from_slice::<Vec<Event>>(&payload)?)
}

fn parse_binary(
    headers: HeaderMap,
    payload: Option<(String, Bytes)>,
) -> Result<Event, http::ReaderError> {
    let mut ce = Event::new();
    read_ce_headers(headers, &mut ce)?;

    if payload.is_some() {
        let (ct, p) = payload.unwrap();
        ce.payload = Some(Payload {
            content_type: ct,
            data: p.to_vec(),
        });
    }

    Ok(ce)
}

fn read_ce_headers(mut headers: HeaderMap, ce: &mut Event) -> Result<(), http::ReaderError> {
    ce.id = unwrap_and_remove_header!(headers, http::CE_ID_HEADER, "id")?;
    ce.event_type = unwrap_and_remove_header!(headers, http::CE_TYPE_HEADER, "type")?;
    ce.spec_version =
        unwrap_and_remove_header!(headers, http::CE_SPECVERSION_HEADER, "specversion").and_then(|sv| {
            sv.try_into()
                .map_err(|e| http::ReaderError::InvalidMetadata {name: "specversion".to_string(), reason: e})
        })?;
    ce.source = unwrap_and_remove_header!(headers, http::CE_SOURCE_HEADER, "source")?;
    ce.subject = unwrap_and_remove_header!(headers, http::CE_SUBJECT_HEADER, "subject").ok();
    ce.time = unwrap_and_remove_header!(headers, http::CE_TIME_HEADER, "time")
        .and_then(|t| {
            DateTime::parse_from_rfc3339(&t)
                .map_err(|e| http::ReaderError::InvalidMetadata {name: "time".to_string(), reason: e.to_string()})
        })
        .ok();

    let extensions = headers
        .iter()
        .map(|(name, value)| (name.as_str(), value))
        .filter(|(name, _)| name.starts_with("ce-"))
        .map(|(name, value)| Ok((name.to_string(), value.to_str()?.to_string())))
        .collect::<Result<Vec<(String, String)>, header::ToStrError>>()
        .map_err(|e| http::ReaderError::InvalidMetadata {name: "extensions".to_string(), reason: e.to_string()})?;

    ce.extensions = extensions.into_iter().collect();

    Ok(())
}
