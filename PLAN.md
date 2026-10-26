# Product development plan

Each round completes one feature or repair. Future items are planned, not implemented claims.

## Current round

ocsf-trace-correlation

## Roadmap

### ocsf-trace-correlation — Add trace_id/span_id correlation fields to OCSF event builders

Status: in_progress

Acceptance:
- OCSF events emitted inside a sampled OTel span include trace.uid (32 lowercase hex characters) and list trace in metadata.profiles
- OCSF events emitted without an active sampled span omit the trace object and do not list the profile
- openshell-ocsf has no OpenTelemetry dependency
- The Trace profile schema is vendored and schema validation tests cover events with and without trace

Implementation: `crates/openshell-ocsf/src/trace.rs`, `crates/openshell-ocsf/src/builders/mod.rs`, `crates/openshell-ocsf/src/emit.rs`, `crates/openshell-ocsf/src/downgrade.rs`, `crates/openshell-ocsf/schemas/trace_profile/`, `crates/openshell-otel/src/trace_context.rs`, `crates/openshell-supervisor-network/src/deny_span.rs`

Tests: `crates/openshell-ocsf/tests/trace_correlation.rs`, `crates/openshell-ocsf/tests/downgrade_trace.rs`, `crates/openshell-otel/src/trace_context.rs`, `crates/openshell-supervisor-network/src/deny_span.rs`

