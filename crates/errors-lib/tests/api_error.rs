/*
 * Integration tests for ApiError serialization.
 * * This uses snapshot testing to ensure the JSON structure remains stable.
 */

use errors_lib::{perform_task, ReportExt};
use serde_json::Value;

#[test]
fn test_api_error_json_structure() {
    // 1. Generate an error from the lib
    let result = perform_task();
    assert!(result.is_err(), "perform_task should return an error");

    let lib_report = result.unwrap_err();
    let api_error = lib_report.to_api_error();

    // 2. Serialize to a JSON Value
    let json_value = serde_json::to_value(&api_error)
        .expect("Failed to serialize ApiError to JSON");

    // 3. Assert on specific stable fields
    assert_eq!(json_value["code"], "config::invalid_format");
    assert!(json_value["title"].as_str().unwrap().contains("Failed to parse config"));

    // 4. Verify correlation_id exists and is the correct length (from nanoid!(8))
    let id = json_value["correlation_id"].as_str().expect("correlation_id missing");
    assert_eq!(id.len(), 8);

    // 5. Verify version (git_hash) is present
    assert!(json_value.get("git_hash").is_some());
}

#[test]
fn test_snapshot_api_error() {
    let result = perform_task().unwrap_err();
    let api_error = result.to_api_error();

    // We use a redacted version for the snapshot because
    // git_hash and correlation_id change every run.
    let mut redacted = serde_json::to_value(&api_error).unwrap();
    redacted["correlation_id"] = Value::String("REDACTED_ID".to_string());
    redacted["git_hash"] = Value::String("REDACTED_HASH".to_string());

    // This will create/check a file in tests/snapshots/
    insta::assert_json_snapshot!(redacted);
}
