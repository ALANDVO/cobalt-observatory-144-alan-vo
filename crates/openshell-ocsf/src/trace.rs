use serde::{Deserialize, Serialize};

/// Trace correlation context for OCSF events.
/// 
/// When a sampled OpenTelemetry span is active at emission time,
/// the event carries `trace.uid` and lists `trace` in `metadata.profiles`.
/// Without an active sampled span, the `trace` object is omitted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TraceContext {
    /// W3C trace ID as 32 lowercase hex characters
    pub uid: String,
}

impl TraceContext {
    /// Create a new trace context with the given trace ID.
    pub fn new(uid: String) -> Self {
        Self { uid }
    }

    /// Validate that the UID is a valid W3C trace ID (32 lowercase hex chars).
    /// Digits 0-9 are allowed (they are neither upper nor lower case).
    pub fn is_valid(&self) -> bool {
        self.uid.len() == 32 && self.uid.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
    }
}

/// Optional span context for future use when span timing is available.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SpanContext {
    /// W3C span ID as 16 lowercase hex characters
    pub span_id: String,
    /// Span start time as Unix timestamp in milliseconds
    pub start_time: u64,
    /// Span end time as Unix timestamp in milliseconds
    pub end_time: u64,
}

impl SpanContext {
    pub fn new(span_id: String, start_time: u64, end_time: u64) -> Self {
        Self { span_id, start_time, end_time }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_context_valid() {
        let ctx = TraceContext::new("0af7651916cd43dd8448eb211c80319c".to_string());
        println!("uid: '{}'", ctx.uid);
        println!("len: {}", ctx.uid.len());
        for (i, c) in ctx.uid.chars().enumerate() {
            println!("  char {}: '{}' is_ascii_hexdigit={} is_uppercase={}", i, c, c.is_ascii_hexdigit(), c.is_ascii_uppercase());
        }
        println!("is_valid: {}", ctx.is_valid());
        assert!(ctx.is_valid());
        assert_eq!(ctx.uid, "0af7651916cd43dd8448eb211c80319c");
    }

    #[test]
    fn test_trace_context_invalid_length() {
        let ctx = TraceContext::new("0af7651916cd43dd8448eb211c80319".to_string()); // 31 chars
        assert!(!ctx.is_valid());
    }

    #[test]
    fn test_trace_context_invalid_chars() {
        let ctx = TraceContext::new("0af7651916cd43dd8448eb211c80319G".to_string()); // uppercase G
        assert!(!ctx.is_valid());
    }

    #[test]
    fn test_trace_context_invalid_uppercase() {
        let ctx = TraceContext::new("0AF7651916CD43DD8448EB211C80319C".to_string()); // all uppercase
        assert!(!ctx.is_valid());
    }

    #[test]
    fn test_trace_context_serialization() {
        let ctx = TraceContext::new("0af7651916cd43dd8448eb211c80319c".to_string());
        let json = serde_json::to_string(&ctx).unwrap();
        assert_eq!(json, r#"{"uid":"0af7651916cd43dd8448eb211c80319c"}"#);
    }
}
