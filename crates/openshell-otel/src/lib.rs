#[cfg(feature = "otel")]
pub mod trace_context;

#[cfg(feature = "otel")]
pub use trace_context::extract_sampled_trace_context;
