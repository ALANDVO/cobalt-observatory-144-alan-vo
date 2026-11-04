# Changelog

## 1.1.0 — work-in-progress checkpoint

Partial code published for continued development. Known issues: STATUS.md.

## 1.0.0 — work-in-progress checkpoint

Partial code published for continued development. Known issues: STATUS.md.

### Added

- **OCSF trace correlation** (`ocsf-trace-correlation`):
  - `TraceContext` type with W3C trace ID validation (32 lowercase hex chars)
  - Builder `trace_context()` setter adds `trace.uid` and `trace` profile to events
  - Automatic trace context injection via `register_trace_extractor()` and `ocsf_emit!` macro
  - Downgrade stripping of `trace` object and `trace` profile for OCSF 1.3 and 1.1
  - Vendored OCSF 1.8 Trace profile schema (`schemas/trace_profile/`)
  - `openshell-otel::extract_sampled_trace_context()` extracts active sampled OTel span trace ID
  - `openshell-supervisor-network::with_deny_span()` / `emit_deny_event()` for INFO-level deny spans

### Tests

- 41 tests passing across all crates
- Trace correlation: events with/without trace, explicit vs auto injection, invalid trace handling
- Downgrade: preserves trace for 1.8, strips for 1.3/1.1, preserves other profiles
- Deny span: reason display, policy name, destination, span emission


