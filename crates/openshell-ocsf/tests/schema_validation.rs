//! Schema validation tests for OCSF events with Trace profile.
//! Validates events against the vendored OCSF 1.8 Trace profile schema.

use openshell_ocsf::builders::{NetworkActivityBuilder, Metadata};
use openshell_ocsf::trace::TraceContext;
use serde_json::Value;
use std::fs;

/// Load a JSON schema from the vendored schemas directory.
fn load_schema(name: &str) -> Value {
    let path = format!("schemas/trace_profile/{name}");
    let content = fs::read_to_string(&path).expect(&format!("Failed to read schema: {path}"));
    serde_json::from_str(&content).expect(&format!("Failed to parse schema: {path}"))
}

/// Validate a JSON value against a schema.
fn validate(schema: &Value, instance: &Value) -> Result<(), Vec<String>> {
    let compiled = jsonschema::Validator::new(schema).expect("Schema compilation failed");
    let errors: Vec<String> = compiled
        .iter_errors(instance)
        .map(|e| format!("{e}"))
        .collect();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[test]
fn test_network_event_with_trace_validates_against_trace_profile() {
    let trace = TraceContext::new("0af7651916cd43dd8448eb211c80319c".to_string());
    let builder = NetworkActivityBuilder::new().trace_context(trace);
    let event_json = serde_json::to_value(&builder).unwrap();

    // Load the trace profile schema
    let trace_profile_schema = load_schema("profile.json");
    
    // The trace profile only validates metadata.profiles and trace object
    // We need to extract just those parts for validation
    let mut validation_instance = serde_json::json!({});
    if let Some(metadata) = event_json.get("metadata") {
        validation_instance["metadata"] = metadata.clone();
    }
    if let Some(trace_obj) = event_json.get("trace") {
        validation_instance["trace"] = trace_obj.clone();
    }

    let result = validate(&trace_profile_schema, &validation_instance);
    assert!(result.is_ok(), "Validation failed: {:?}", result.err());
}

#[test]
fn test_network_event_without_trace_validates_against_trace_profile() {
    let builder = NetworkActivityBuilder::new();
    let event_json = serde_json::to_value(&builder).unwrap();

    let trace_profile_schema = load_schema("profile.json");
    
    let mut validation_instance = serde_json::json!({});
    if let Some(metadata) = event_json.get("metadata") {
        validation_instance["metadata"] = metadata.clone();
    }
    // No trace object

    let result = validate(&trace_profile_schema, &validation_instance);
    assert!(result.is_ok(), "Validation failed: {:?}", result.err());
}

#[test]
fn test_http_event_with_trace_validates_against_trace_profile() {
    let trace = TraceContext::new("11111111111111111111111111111111".to_string());
    let builder = openshell_ocsf::builders::HttpActivityBuilder::new().trace_context(trace);
    let event_json = serde_json::to_value(&builder).unwrap();

    let trace_profile_schema = load_schema("profile.json");
    
    let mut validation_instance = serde_json::json!({});
    if let Some(metadata) = event_json.get("metadata") {
        validation_instance["metadata"] = metadata.clone();
    }
    if let Some(trace_obj) = event_json.get("trace") {
        validation_instance["trace"] = trace_obj.clone();
    }

    let result = validate(&trace_profile_schema, &validation_instance);
    assert!(result.is_ok(), "Validation failed: {:?}", result.err());
}

#[test]
fn test_trace_object_validates_against_trace_schema() {
    let trace = TraceContext::new("0af7651916cd43dd8448eb211c80319c".to_string());
    let builder = NetworkActivityBuilder::new().trace_context(trace);
    let event_json = serde_json::to_value(&builder).unwrap();

    let trace_schema = load_schema("trace.json");
    
    if let Some(trace_obj) = event_json.get("trace") {
        let result = validate(&trace_schema, trace_obj);
        assert!(result.is_ok(), "Trace object validation failed: {:?}", result.err());
    } else {
        panic!("trace object missing from event");
    }
}

#[test]
fn test_trace_object_with_invalid_uid_fails_trace_schema() {
    // Create an event with an invalid trace UID (uppercase)
    let mut builder = NetworkActivityBuilder::new();
    builder.base.trace = Some(TraceContext::new("0AF7651916CD43DD8448EB211C80319C".to_string()));
    builder.base.metadata.add_profile("trace");
    let event_json = serde_json::to_value(&builder).unwrap();

    let trace_schema = load_schema("trace.json");
    
    if let Some(trace_obj) = event_json.get("trace") {
        let result = validate(&trace_schema, trace_obj);
        assert!(result.is_err(), "Expected validation to fail for uppercase trace ID");
    } else {
        panic!("trace object missing from event");
    }
}

#[test]
fn test_trace_object_with_invalid_length_fails_trace_schema() {
    // Create an event with an invalid trace UID (31 chars)
    let mut builder = NetworkActivityBuilder::new();
    builder.base.trace = Some(TraceContext::new("0af7651916cd43dd8448eb211c80319".to_string()));
    builder.base.metadata.add_profile("trace");
    let event_json = serde_json::to_value(&builder).unwrap();

    let trace_schema = load_schema("trace.json");
    
    if let Some(trace_obj) = event_json.get("trace") {
        let result = validate(&trace_schema, trace_obj);
        assert!(result.is_err(), "Expected validation to fail for short trace ID");
    } else {
        panic!("trace object missing from event");
    }
}

#[test]
fn test_event_with_multiple_profiles_including_trace_validates() {
    let trace = TraceContext::new("0af7651916cd43dd8448eb211c80319c".to_string());
    let mut metadata = Metadata::with_profiles(&["security_control", "network_proxy", "container", "host"]);
    metadata.add_profile("trace");
    let builder = NetworkActivityBuilder::new().with_metadata(metadata).trace_context(trace);
    let event_json = serde_json::to_value(&builder).unwrap();

    // Validate the trace object against the trace schema
    let trace_schema = load_schema("trace.json");
    let trace_obj = event_json.get("trace").expect("trace object missing");
    let result = validate(&trace_schema, trace_obj);
    assert!(result.is_ok(), "Trace object validation failed: {:?}", result.err());

    // Verify trace profile is in metadata.profiles
    let profiles = event_json["metadata"]["profiles"].as_array().unwrap();
    assert!(profiles.iter().any(|p| p == "trace"), "trace profile missing from metadata.profiles");
}
