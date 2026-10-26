use crate::trace::TraceContext;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Metadata for OCSF events including profiles.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Metadata {
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    pub profiles: HashSet<String>,
}

impl Metadata {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_profiles(profiles: &[&str]) -> Self {
        let mut set = HashSet::new();
        for p in profiles {
            set.insert(p.to_string());
        }
        Self { profiles: set }
    }

    pub fn add_profile(&mut self, profile: &str) {
        self.profiles.insert(profile.to_string());
    }

    pub fn has_profile(&self, profile: &str) -> bool {
        self.profiles.contains(profile)
    }
}

/// Base builder for OCSF events with trace correlation support.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BaseEventBuilder {
    pub class_uid: u16,
    pub activity_id: u16,
    pub metadata: Metadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<TraceContext>,
}

impl BaseEventBuilder {
    pub fn new(class_uid: u16, activity_id: u16) -> Self {
        Self {
            class_uid,
            activity_id,
            metadata: Metadata::new(),
            trace: None,
        }
    }

    pub fn with_metadata(mut self, metadata: Metadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Set the trace context for this event.
    /// This adds the "trace" profile to metadata and serializes the trace object.
    pub fn trace_context(mut self, trace: TraceContext) -> Self {
        if trace.is_valid() {
            self.trace = Some(trace);
            self.metadata.add_profile("trace");
        }
        self
    }

    /// Clear the trace context (removes trace object and profile).
    pub fn clear_trace(mut self) -> Self {
        self.trace = None;
        self.metadata.profiles.remove("trace");
        self
    }
}

/// Network activity event builder (class_uid: 4001, activity_id: 1 for connection events).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct NetworkActivityBuilder {
    #[serde(flatten)]
    pub base: BaseEventBuilder,
    // Network-specific fields would go here
}

impl NetworkActivityBuilder {
    pub fn new() -> Self {
        Self {
            base: BaseEventBuilder::new(4001, 1),
        }
    }

    pub fn with_metadata(mut self, metadata: Metadata) -> Self {
        self.base = self.base.with_metadata(metadata);
        self
    }

    pub fn trace_context(self, trace: TraceContext) -> Self {
        Self {
            base: self.base.trace_context(trace),
        }
    }

    pub fn clear_trace(self) -> Self {
        Self {
            base: self.base.clear_trace(),
        }
    }
}

impl Default for NetworkActivityBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// HTTP activity event builder (class_uid: 4002).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct HttpActivityBuilder {
    #[serde(flatten)]
    pub base: BaseEventBuilder,
}

impl HttpActivityBuilder {
    pub fn new() -> Self {
        Self {
            base: BaseEventBuilder::new(4002, 1),
        }
    }

    pub fn with_metadata(mut self, metadata: Metadata) -> Self {
        self.base = self.base.with_metadata(metadata);
        self
    }

    pub fn trace_context(self, trace: TraceContext) -> Self {
        Self {
            base: self.base.trace_context(trace),
        }
    }

    pub fn clear_trace(self) -> Self {
        Self {
            base: self.base.clear_trace(),
        }
    }
}

impl Default for HttpActivityBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_builder_with_trace() {
        let trace = TraceContext::new("0af7651916cd43dd8448eb211c80319c".to_string());
        let builder = NetworkActivityBuilder::new().trace_context(trace);
        
        assert!(builder.base.trace.is_some());
        assert!(builder.base.metadata.has_profile("trace"));
        assert_eq!(builder.base.trace.as_ref().unwrap().uid, "0af7651916cd43dd8448eb211c80319c");
    }

    #[test]
    fn test_network_builder_without_trace() {
        let builder = NetworkActivityBuilder::new();
        
        assert!(builder.base.trace.is_none());
        assert!(!builder.base.metadata.has_profile("trace"));
    }

    #[test]
    fn test_network_builder_invalid_trace() {
        let trace = TraceContext::new("invalid".to_string());
        let builder = NetworkActivityBuilder::new().trace_context(trace);
        
        assert!(builder.base.trace.is_none());
        assert!(!builder.base.metadata.has_profile("trace"));
    }

    #[test]
    fn test_clear_trace() {
        let trace = TraceContext::new("0af7651916cd43dd8448eb211c80319c".to_string());
        let builder = NetworkActivityBuilder::new().trace_context(trace).clear_trace();
        
        assert!(builder.base.trace.is_none());
        assert!(!builder.base.metadata.has_profile("trace"));
    }

    #[test]
    fn test_serialization_with_trace() {
        let trace = TraceContext::new("0af7651916cd43dd8448eb211c80319c".to_string());
        let builder = NetworkActivityBuilder::new().trace_context(trace);
        
        let json = serde_json::to_value(&builder).unwrap();
        assert_eq!(json["trace"]["uid"], "0af7651916cd43dd8448eb211c80319c");
        assert!(json["metadata"]["profiles"].as_array().unwrap().contains(&serde_json::Value::String("trace".to_string())));
    }

    #[test]
    fn test_serialization_without_trace() {
        let builder = NetworkActivityBuilder::new();
        
        let json = serde_json::to_value(&builder).unwrap();
        assert!(json.get("trace").is_none() || json["trace"].is_null());
        // profiles may be missing entirely when empty due to skip_serializing_if
        let profiles = json["metadata"]["profiles"].as_array();
        assert!(profiles.is_none() || !profiles.unwrap().contains(&serde_json::Value::String("trace".to_string())));
    }
}
