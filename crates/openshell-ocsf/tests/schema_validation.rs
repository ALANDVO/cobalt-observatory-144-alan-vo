// Schema validation tests for OCSF events with Trace profile.
// Validates events against the vendored OCSF 1.8 Trace profile schema.

use openshell_ocsf::builders::{NetworkActivityBuilder, Metadata};
use openshell_ocsf::trace::TraceContext;
use serde_json::Value;
use std::fs;

/// Load a JSON schema from the vendored schemas directory.
fn load_schema(name: &str) -> Value {
    let path = format!("schemas/trace_profile/{name}");
    let content = fs::read_to_string(&path).expect(&format!("Failed to read schema: {path}"));
    // Parse and add base URI for resolving relative $ref references
    let mut schema: Value = serde_json::from_str(&content).expect(&format!("Failed to parse schema: {path}"));
    // Set base URI so relative $ref can resolve within the schemas directory
    if let Some(obj) = schema.as_object_mut() {
        obj.insert("$base_uri".to_string(), Value::String(format!("file://{}/crates/openshell-ocsf/", std::env!("CARGO_MANIFEST_DIR"))));
    }
    schema
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
