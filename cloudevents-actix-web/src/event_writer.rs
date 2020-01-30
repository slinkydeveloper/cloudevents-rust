use actix_web::HttpResponse;
use cloudevents::http;
use cloudevents::Event;
use serde::Serialize;

pub struct EventWriter {}

impl http::EventWriter<HttpResponse> for EventWriter {
    fn write_cloud_event(res: http::HttpEvent) -> Result<HttpResponse, http::WriterError> {
        match res {
            http::HttpEvent::Binary(e) => write_binary(e),
            http::HttpEvent::Structured(e) => serialize_and_write(e, http::CE_JSON_CONTENT_TYPE),
            http::HttpEvent::Batch(vec) => serialize_and_write(vec, http::CE_BATCH_JSON_CONTENT_TYPE),
        }
    }
}

fn write_binary(event: Event) -> Result<HttpResponse, http::WriterError> {
    // Write headers
    let mut builder = HttpResponse::Ok();
    builder.header(http::CE_ID_HEADER, event.id);
    builder.header(http::CE_SPECVERSION_HEADER, event.spec_version.to_string());
    builder.header(http::CE_SOURCE_HEADER, event.source);
    builder.header(http::CE_TYPE_HEADER, event.event_type);
    if let Some(sub) = event.subject {
        builder.header(http::CE_SUBJECT_HEADER, sub);
    }
    if let Some(time) = event.time {
        builder.header(http::CE_TIME_HEADER, time.to_rfc3339());
    }
    let result = if let Some(p) = event.payload {
        builder.content_type(p.content_type).body(p.data)
    } else {
        builder.finish()
    };

    Ok(result)
}

fn serialize_and_write<T: Serialize>(value: T, content_type: &str) -> Result<HttpResponse, http::WriterError> {
    Ok(serde_json::to_vec(&value)
        .map(|j| {
            HttpResponse::Ok()
                .content_type(content_type)
                .body(j)
        })?
    )
}
