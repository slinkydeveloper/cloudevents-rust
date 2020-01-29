use cloudevents::{SpecVersion, Event, Payload};
use chrono::DateTime;
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
