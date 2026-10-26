use openshell_ocsf::builders::{NetworkActivityBuilder, Metadata};
use openshell_ocsf::trace::TraceContext;
use openshell_ocsf::{downgrade_event, OcsfVersion};

#[test]
fn test_downgrade_to_1_8_preserves_trace() {
    let trace = TraceContext::new("0af7651916cd43dd8448eb211c80319c".to_string());
    let builder = NetworkActivityBuilder::new().trace_context(trace);
    let json = serde_json::to_value(&builder).unwrap();

    let downgraded = downgrade_event(json, OcsfVersion::V1_8);

    assert_eq!(downgraded["trace"]["uid"], "0af7651916cd43dd8448eb211c80319c");
    let profiles = downgraded["metadata"]["profiles"].as_array().unwrap();
    assert!(profiles.iter().any(|p| p == "trace"));
}

#[test]
fn test_downgrade_to_1_3_strips_trace() {
    let trace = TraceContext::new("0af7651916cd43dd8448eb211c80319c".to_string());
    let builder = NetworkActivityBuilder::new().trace_context(trace);
    let json = serde_json::to_value(&builder).unwrap();

    let downgraded = downgrade_event(json, OcsfVersion::V1_3);

    assert!(downgraded.get("trace").is_none() || downgraded["trace"].is_null());
    let profiles = downgraded["metadata"]["profiles"].as_array().unwrap();
    assert!(!profiles.iter().any(|p| p == "trace"), "trace profile should be stripped for 1.3: {:?}", profiles);
}

#[test]
fn test_downgrade_to_1_1_strips_trace() {
    let trace = TraceContext::new("0af7651916cd43dd8448eb211c80319c".to_string());
    let builder = NetworkActivityBuilder::new().trace_context(trace);
    let json = serde_json::to_value(&builder).unwrap();

    let downgraded = downgrade_event(json, OcsfVersion::V1_1);

    assert!(downgraded.get("trace").is_none() || downgraded["trace"].is_null());
    let profiles = downgraded["metadata"]["profiles"].as_array().unwrap();
    assert!(!profiles.iter().any(|p| p == "trace"), "trace profile should be stripped for 1.1: {:?}", profiles);
}

#[test]
fn test_downgrade_event_without_trace_unchanged() {
    let builder = NetworkActivityBuilder::new();
    let json = serde_json::to_value(&builder).unwrap();

    let downgraded_1_3 = downgrade_event(json.clone(), OcsfVersion::V1_3);
    let downgraded_1_1 = downgrade_event(json.clone(), OcsfVersion::V1_1);

    // Should be essentially the same (no trace to strip)
    assert_eq!(downgraded_1_3, json);
    assert_eq!(downgraded_1_1, json);
}

#[test]
fn test_downgrade_preserves_other_profiles() {
    let trace = TraceContext::new("0af7651916cd43dd8448eb211c80319c".to_string());
    let mut metadata = Metadata::with_profiles(&["security_control", "network_proxy", "container", "host"]);
    metadata.add_profile("trace");
    let builder = NetworkActivityBuilder::new().with_metadata(metadata).trace_context(trace);
    let json = serde_json::to_value(&builder).unwrap();

    let downgraded = downgrade_event(json, OcsfVersion::V1_3);

    // Other profiles should be preserved
    let profiles = downgraded["metadata"]["profiles"].as_array().unwrap();
    let profile_set: std::collections::HashSet<_> = profiles.iter().map(|p| p.as_str().unwrap()).collect();
    assert!(profile_set.contains("security_control"));
    assert!(profile_set.contains("network_proxy"));
    assert!(profile_set.contains("container"));
    assert!(profile_set.contains("host"));
    assert!(!profile_set.contains("trace"));
}

#[test]
fn test_downgrade_preserves_other_fields() {
    let trace = TraceContext::new("0af7651916cd43dd8448eb211c80319c".to_string());
    let builder = NetworkActivityBuilder::new().trace_context(trace);
    let json = serde_json::to_value(&builder).unwrap();

    let downgraded = downgrade_event(json.clone(), OcsfVersion::V1_3);

    // class_uid and activity_id should be preserved
    assert_eq!(downgraded["class_uid"], json["class_uid"]);
    assert_eq!(downgraded["activity_id"], json["activity_id"]);

    // metadata should exist (just without trace profile)
    assert!(downgraded.get("metadata").is_some());
}
