use openshell_ocsf::trace::TraceContext;
use openshell_otel::extract_sampled_trace_context;
use tracing::{info_span};
use std::fmt;

/// Reason for a network connection denial.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DenyReason {
    /// L4 policy denied the connection
    L4Policy { policy: String, destination: String },
    /// Mapping/SSRF protection denied the connection
    MappingSsrf { reason: String, destination: String },
    /// L7 HTTP policy denied the request
    L7Http { policy: String, reason: String, url: String },
    /// Custom denial reason
    Other { reason: String },
}

impl fmt::Display for DenyReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DenyReason::L4Policy { policy, destination } => {
                write!(f, "L4 policy '{}' denied connection to {}", policy, destination)
            }
            DenyReason::MappingSsrf { reason, destination } => {
                write!(f, "Mapping/SSRF denied connection to {}: {}", destination, reason)
            }
            DenyReason::L7Http { policy, reason, url } => {
                write!(f, "L7 HTTP policy '{}' denied {}: {}", policy, url, reason)
            }
            DenyReason::Other { reason } => {
                write!(f, "Denied: {}", reason)
            }
        }
    }
}

/// Execute a closure within a deny span at INFO level.
/// 
/// The span is named "supervisor.egress.deny" and carries attributes:
/// - deny.reason: human-readable reason
/// - deny.policy: policy name (if applicable)
/// - deny.destination: destination address/URL (if applicable)
pub fn with_deny_span<F, T>(reason: DenyReason, f: F) -> T
where
    F: FnOnce() -> T,
{
    let span = info_span!(
        "supervisor.egress.deny",
        "deny.reason" = %reason,
        "deny.policy" = reason.policy_name(),
        "deny.destination" = reason.destination(),
    );
    let _entered = span.enter();
    f()
}

/// Helper to emit an OCSF deny event within a deny span.
/// The span is entered around the emission so the trace context is available.
pub fn emit_deny_event<F, T>(reason: DenyReason, f: F) -> T
where
    F: FnOnce(Option<TraceContext>) -> T,
{
    with_deny_span(reason, || {
        let trace_ctx = extract_sampled_trace_context();
        f(trace_ctx)
    })
}

impl DenyReason {
    fn policy_name(&self) -> Option<String> {
        match self {
            DenyReason::L4Policy { policy, .. } => Some(policy.clone()),
            DenyReason::L7Http { policy, .. } => Some(policy.clone()),
            _ => None,
        }
    }

    fn destination(&self) -> Option<String> {
        match self {
            DenyReason::L4Policy { destination, .. } => Some(destination.clone()),
            DenyReason::MappingSsrf { destination, .. } => Some(destination.clone()),
            DenyReason::L7Http { url, .. } => Some(url.clone()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deny_reason_display() {
        let reason = DenyReason::L4Policy { 
            policy: "default-deny".to_string(), 
            destination: "api.suspicious.com:443".to_string() 
        };
        assert_eq!(reason.to_string(), "L4 policy 'default-deny' denied connection to api.suspicious.com:443");
    }

    #[test]
    fn test_deny_reason_policy_name() {
        let reason = DenyReason::L4Policy { 
            policy: "default-deny".to_string(), 
            destination: "api.suspicious.com:443".to_string() 
        };
        assert_eq!(reason.policy_name(), Some("default-deny".to_string()));
    }

    #[test]
    fn test_deny_reason_destination() {
        let reason = DenyReason::L7Http { 
            policy: "http-policy".to_string(),
            reason: "blocked path".to_string(),
            url: "https://api.example.com/admin".to_string()
        };
        assert_eq!(reason.destination(), Some("https://api.example.com/admin".to_string()));
    }

    #[test]
    fn test_emit_deny_event() {
        let reason = DenyReason::Other { reason: "test".to_string() };
        let result = emit_deny_event(reason, |trace_ctx| {
            // In test without OTel, trace_ctx will be None
            assert!(trace_ctx.is_none());
            42
        });
        assert_eq!(result, 42);
    }

    #[test]
    fn test_with_deny_span() {
        let reason = DenyReason::Other { reason: "test".to_string() };
        let result = with_deny_span(reason, || {
            42
        });
        assert_eq!(result, 42);
    }
}
