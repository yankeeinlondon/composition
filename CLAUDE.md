# Composition Project

This repository contains documentation and research on various Rust topics.

## Skills

### rust-logging

Comprehensive guidance on logging and tracing in Rust applications:

- Choosing between `log` and `tracing` ecosystems
- Setting up `env_logger`, `fern`, and other `log` implementations
- Using `tracing` with spans, events, and the `#[instrument]` attribute
- Async-aware instrumentation with `.instrument()`
- File logging with rotation via `tracing-appender`
- Bridging `log` and `tracing` with `tracing-log`
- OpenTelemetry integration for distributed tracing
- Web server integration with `tower-http`

Key recommendations:
- New async applications: use `tracing` + `tracing-subscriber`
- Libraries: use `log` facade only
- Simple CLIs: `log` + `env_logger`
