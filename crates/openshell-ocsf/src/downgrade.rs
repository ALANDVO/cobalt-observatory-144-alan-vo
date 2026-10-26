/// OCSF version enum for downgrade functionality.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OcsfVersion {
    V1_1,
    V1_3,
    V1_8,
}

impl OcsfVersion {
    /// Check if this version supports the Trace profile (introduced in 1.4).
    pub fn supports_trace_profile(&self) -> bool {
        matches!(self, OcsfVersion::V1_8)
    }
}

/// Downgrade an OCSF event to a specific version.
/// Strips the trace object and Trace profile for versions before 1.4.
pub fn downgrade_event(value: serde_json::Value, target: OcsfVersion) -> serde_json::Value {
    let mut value = value;

    if !target.supports_trace_profile() {
        // Remove trace object
        if let Some(obj) = value.as_object_mut() {
            obj.remove("trace");

            // Remove "trace" from metadata.profiles
            if let Some(metadata) = obj.get_mut("metadata").and_then(|m| m.as_object_mut()) {
                if let Some(profiles) = metadata.get_mut("profiles").and_then(|p| p.as_array_mut()) {
                    profiles.retain(|p| p != "trace");
                }
            }
        }
    }

    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builders::NetworkActivityBuilder;
    use crate::trace::TraceContext;

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

        assert!(downgraded.get("trace").is_none());
        let profiles = downgraded["metadata"]["profiles"].as_array().unwrap();
        assert!(!profiles.iter().any(|p| p == "trace"));
    }

    #[test]
    fn test_downgrade_to_1_1_strips_trace() {
        let trace = TraceContext::new("0af7651916cd43dd8448eb211c80319c".to_string());
        let builder = NetworkActivityBuilder::new().trace_context(trace);
        let json = serde_json::to_value(&builder).unwrap();

        let downgraded = downgrade_event(json, OcsfVersion::V1_1);

        assert!(downgraded.get("trace").is_none());
        let profiles = downgraded["metadata"]["profiles"].as_array().unwrap();
        assert!(!profiles.iter().any(|p| p == "trace"));
    }

    #[test]
    fn test_downgrade_event_without_trace_unchanged() {
        let builder = NetworkActivityBuilder::new();
        let json = serde_json::to_value(&builder).unwrap();

        let downgraded = downgrade_event(json, OcsfVersion::V1_3);

        assert_eq!(downgraded["class_uid"], 4001);
        assert_eq!(downgraded["activity_id"], 1);
        assert!(downgraded.get("trace").is_none());
    }

    #[test]
    fn test_downgrade_preserves_other_profiles() {
        let trace = TraceContext::new("0af7651916cd43dd8448eb211c80319c".to_string());
        let mut builder = NetworkActivityBuilder::new().trace_context(trace);
        builder.base.metadata.add_profile("security_control");
        builder.base.metadata.add_profile("network_proxy");
        let json = serde_json::to_value(&builder).unwrap();

        let downgraded = downgrade_event(json, OcsfVersion::V1_3);

        let profiles = downgraded["metadata"]["profiles"].as_array().unwrap();
        assert!(profiles.iter().any(|p| p == "security_control"));
        assert!(profiles.iter().any(|p| p == "network_proxy"));
        assert!(!profiles.iter().any(|p| p == "trace"));
    }

    #[test]
    fn test_downgrade_preserves_other_fields() {
        let trace = TraceContext::new("0af7651916cd43dd8448eb211c80319c".to_string());
        let builder = NetworkActivityBuilder::new().trace_context(trace);
        let json = serde_json::to_value(&builder).unwrap();

        let downgraded = downgrade_event(json, OcsfVersion::V1_3);

        assert_eq!(downgraded["class_uid"], 4001);
        assert_eq!(downgraded["activity_id"], 1);
    }
}
