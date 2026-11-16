use opentelemetry::trace::{SpanContext as OtelSpanContext, TraceContextExt};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use openshell_ocsf::trace::TraceContext;









#[cfg(feature = "otel")]
pub fn extract_sampled_trace_context() -> Option<TraceContext> {
    let span = tracing::Span::current();

    let otel_cx = OpenTelemetrySpanExt::context(&span);

    let otel_span_context: OtelSpanContext = otel_cx.span().span_context().clone();
    

    if !otel_span_context.is_valid() {
        return None;
    }
    

    if !otel_span_context.is_sampled() {
        return None;
    }
    

    let trace_id = otel_span_context.trace_id();
    let trace_id_hex = format!("{:032x}", u128::from_be_bytes(trace_id.to_bytes()));
    
    Some(TraceContext::new(trace_id_hex))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_sampled_trace_context_no_span() {


        let result = extract_sampled_trace_context();


assert!(result.is_none());
    }
}
