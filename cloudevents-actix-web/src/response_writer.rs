use actix_web::HttpResponse;
use cloudevents::http::*;
use cloudevents::Event;
use serde::Serialize;

pub fn write_cloud_event(ce: HttpCloudEvent) -> Result<HttpResponse, actix_web::Error> {
    return match ce {
        HttpCloudEvent::Binary(Some(e)) => write_binary(e),
        HttpCloudEvent::Structured(Some(e)) => serialize_and_write(e, CE_JSON_CONTENT_TYPE),
        HttpCloudEvent::Batch(vec) => serialize_and_write(vec, CE_BATCH_JSON_CONTENT_TYPE),
        _ => return Ok(HttpResponse::Accepted().finish())
    }
}

fn write_binary(event: Event) -> Result<HttpResponse, actix_web::Error> {
    // Write headers
    let mut builder = HttpResponse::Ok();
    builder.header(CE_ID_HEADER, event.id);
    builder.header(CE_SPECVERSION_HEADER, event.spec_version.to_string());
    builder.header(CE_SOURCE_HEADER, event.source);
    builder.header(CE_TYPE_HEADER, event.event_type);
    if let Some(sub) = event.subject {
        builder.header(CE_SUBJECT_HEADER, sub);
    }
    if let Some(time) = event.time {
        builder.header(CE_TIME_HEADER, time.to_rfc3339());
    }
    let result = if let Some(p) = event.payload {
        builder.content_type(p.content_type).body(p.data)
    } else {
        builder.finish()
    };

    Ok(result)
}

fn serialize_and_write<T: Serialize>(value: T, content_type: &str) -> Result<HttpResponse, actix_web::Error> {
    serde_json::to_vec(&value)
        .map(|j| {
            HttpResponse::Ok()
                .content_type(content_type)
                .body(j)
        })
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))
}
