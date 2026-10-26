use openshell_ocsf::builders::NetworkActivityBuilder;
use openshell_ocsf::trace::TraceContext;
use openshell_ocsf::emit::{register_trace_extractor, get_active_trace_context};

fn test_extractor() -> Option<TraceContext> {
    Some(TraceContext::new("0af7651916cd43dd8448eb211c80319c".to_string()))
}

#[test]
fn test_event_with_trace_context_has_trace_profile() {
    let trace = TraceContext::new("0af7651916cd43dd8448eb211c80319c".to_string());
    let builder = NetworkActivityBuilder::new().trace_context(trace);
    
    let json = serde_json::to_value(&builder).unwrap();
    
    // Verify trace object is present
    assert_eq!(json["trace"]["uid"], "0af7651916cd43dd8448eb211c80319c");
    
    // Verify trace profile is in metadata.profiles
    let profiles = json["metadata"]["profiles"].as_array().unwrap();
    assert!(profiles.iter().any(|p| p == "trace"), "trace profile missing from metadata.profiles: {:?}", profiles);
}

#[test]
fn test_event_without_trace_context_omits_trace() {
    let builder = NetworkActivityBuilder::new();
    
    let json = serde_json::to_value(&builder).unwrap();
    
    // Verify trace object is absent
    assert!(json.get("trace").is_none() || json["trace"].is_null(), "trace object should be absent");
    
    // Verify trace profile is not in metadata.profiles (may be missing entirely when empty)
    let profiles = json["metadata"]["profiles"].as_array();
    assert!(profiles.is_none() || !profiles.unwrap().iter().any(|p| p == "trace"), "trace profile should not be in metadata.profiles: {:?}", profiles);
}

#[test]
fn test_event_with_invalid_trace_omits_trace() {
    let trace = TraceContext::new("invalid".to_string()); // Invalid: not 32 hex chars
    let builder = NetworkActivityBuilder::new().trace_context(trace);
    
    let json = serde_json::to_value(&builder).unwrap();
    
    // Invalid trace should be rejected
    assert!(json.get("trace").is_none() || json["trace"].is_null());
    let profiles = json["metadata"]["profiles"].as_array();
    assert!(profiles.is_none() || !profiles.unwrap().iter().any(|p| p == "trace"));
}

#[test]
fn test_manual_extractor_registration() {
    // Register extractor and verify it returns the expected context
    register_trace_extractor(test_extractor);
    let ctx = get_active_trace_context();
    assert!(ctx.is_some());
    assert_eq!(ctx.unwrap().uid, "0af7651916cd43dd8448eb211c80319c");
}

#[test]
fn test_ocsf_emit_injects_trace_when_builder_has_none() {
    // This tests the logic that ocsf_emit! macro implements
    let mut builder = NetworkActivityBuilder::new();
    assert!(builder.base.trace.is_none());
    
    // Simulate what ocsf_emit! does
    if builder.base.trace.is_none() {
        if let Some(trace) = get_active_trace_context() {
            builder.base = builder.base.trace_context(trace);
        }
    }
    
    assert!(builder.base.trace.is_some());
    assert_eq!(builder.base.trace.as_ref().unwrap().uid, "0af7651916cd43dd8448eb211c80319c");
    assert!(builder.base.metadata.has_profile("trace"));
}

#[test]
fn test_ocsf_emit_respects_explicit_trace() {
    let explicit_trace = TraceContext::new("11111111111111111111111111111111".to_string());
    let mut builder = NetworkActivityBuilder::new().trace_context(explicit_trace.clone());
    
    // Simulate what ocsf_emit! does - should NOT overwrite explicit trace
    if builder.base.trace.is_none() {
        if let Some(trace) = get_active_trace_context() {
            builder.base = builder.base.trace_context(trace);
        }
    }
    
    // Explicit trace should be preserved
    assert_eq!(builder.base.trace.as_ref().unwrap().uid, "11111111111111111111111111111111");
}

#[test]
fn test_http_builder_trace_correlation() {
    use openshell_ocsf::builders::HttpActivityBuilder;
    
    let trace = TraceContext::new("0af7651916cd43dd8448eb211c80319c".to_string());
    let builder = HttpActivityBuilder::new().trace_context(trace);
    
    let json = serde_json::to_value(&builder).unwrap();
    assert_eq!(json["trace"]["uid"], "0af7651916cd43dd8448eb211c80319c");
    let profiles = json["metadata"]["profiles"].as_array().unwrap();
    assert!(profiles.iter().any(|p| p == "trace"));
}

#[test]
fn test_clear_trace_removes_profile() {
    let trace = TraceContext::new("0af7651916cd43dd8448eb211c80319c".to_string());
    let builder = NetworkActivityBuilder::new().trace_context(trace).clear_trace();
    
    let json = serde_json::to_value(&builder).unwrap();
    assert!(json.get("trace").is_none() || json["trace"].is_null());
    let profiles = json["metadata"]["profiles"].as_array();
    assert!(profiles.is_none() || !profiles.unwrap().iter().any(|p| p == "trace"));
}

#[test]
fn test_trace_uid_format_32_lowercase_hex() {
    let trace = TraceContext::new("0af7651916cd43dd8448eb211c80319c".to_string());
    assert!(trace.is_valid());
    assert_eq!(trace.uid.len(), 32);
    // Digits 0-9 are neither upper nor lower case, so check for no uppercase
    assert!(trace.uid.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    
    let builder = NetworkActivityBuilder::new().trace_context(trace);
    let json = serde_json::to_value(&builder).unwrap();
    let uid = json["trace"]["uid"].as_str().unwrap();
    assert_eq!(uid.len(), 32);
    assert!(uid.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
}
