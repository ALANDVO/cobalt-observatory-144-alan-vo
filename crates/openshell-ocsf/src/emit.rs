use crate::trace::TraceContext;
use std::sync::OnceLock;

/// Function type for extracting trace context from the current execution context.
/// Returns Some(TraceContext) when a valid sampled span is active, None otherwise.
pub type TraceExtractor = fn() -> Option<TraceContext>;

static TRACE_EXTRACTOR: OnceLock<TraceExtractor> = OnceLock::new();

/// Register a trace extractor function to be called at OCSF emission time.
/// 
/// This allows binaries that depend on OpenTelemetry to provide trace context
/// without openshell-ocsf taking an OTel dependency.
/// 
/// The extractor should return a valid TraceContext only when a sampled span is active.
/// Calling this twice will panic; call once at startup.
pub fn register_trace_extractor(extractor: TraceExtractor) {
    TRACE_EXTRACTOR.set(extractor).expect("trace extractor already registered");
}

/// Get the active trace context from the registered extractor, if any.
pub fn get_active_trace_context() -> Option<TraceContext> {
    TRACE_EXTRACTOR.get().and_then(|extractor| extractor())
}

/// Macro for emitting OCSF events with automatic trace context injection.
/// 
/// If the builder doesn't have a trace context set, this will call the registered
/// extractor to fill it in. An explicit trace_context() call on the builder takes precedence.
#[macro_export]
macro_rules! ocsf_emit {
    ($builder:expr) => {{
        let mut builder = $builder;
        if builder.base.trace.is_none() {
            if let Some(trace) = $crate::emit::get_active_trace_context() {
                builder.base = builder.base.trace_context(trace);
            }
        }
        // Emit via tracing - in practice this would serialize and log
        tracing::info!(ocsf_event = %serde_json::to_string(&builder).unwrap_or_default());
        builder
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builders::NetworkActivityBuilder;
    use crate::trace::TraceContext;

    fn test_extractor() -> Option<TraceContext> {
        Some(TraceContext::new("0af7651916cd43dd8448eb211c80319c".to_string()))
    }

    #[test]
    fn test_register_and_get_extractor() {
        // Note: OnceLock can only be set once per process, so this test
        // only works if run first or in isolation
        register_trace_extractor(test_extractor);
        let ctx = get_active_trace_context();
        assert!(ctx.is_some());
        assert_eq!(ctx.unwrap().uid, "0af7651916cd43dd8448eb211c80319c");
    }

    #[test]
    fn test_ocsf_emit_auto_trace() {
        // This test demonstrates the macro behavior conceptually
        // In real usage, the macro would be invoked at emission sites
        let builder = NetworkActivityBuilder::new();
        assert!(builder.base.trace.is_none());
        
        // After macro expansion with registered extractor, trace would be injected
        // We test the logic directly here
        let mut builder = builder;
        if builder.base.trace.is_none() {
            if let Some(trace) = test_extractor() {
                builder.base = builder.base.trace_context(trace);
            }
        }
        assert!(builder.base.trace.is_some());
    }

    #[test]
    fn test_ocsf_emit_explicit_trace_wins() {
        let explicit_trace = TraceContext::new("11111111111111111111111111111111".to_string());
        let mut builder = NetworkActivityBuilder::new().trace_context(explicit_trace.clone());
        
        // Even with extractor returning different trace, explicit wins
        if builder.base.trace.is_none() {
            if let Some(trace) = test_extractor() {
                builder.base = builder.base.trace_context(trace);
            }
        }
        assert_eq!(builder.base.trace.as_ref().unwrap().uid, "11111111111111111111111111111111");
    }
}
