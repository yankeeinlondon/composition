# OpenTelemetry Integration

Export traces to distributed tracing backends like Jaeger, Tempo, Zipkin, or Honeycomb using `tracing-opentelemetry`.

## Overview

```
Your App (tracing) -> tracing-opentelemetry -> OpenTelemetry SDK -> Exporter -> Backend
                                                                          |
                                                            Jaeger/Tempo/OTLP/etc.
```

## Basic Setup

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = "0.3"
tracing-opentelemetry = "0.28"
opentelemetry = { version = "0.24", features = ["trace"] }
opentelemetry-otlp = { version = "0.18", features = ["tonic"] }
opentelemetry-stdout = { version = "0.7", features = ["trace"] }  # For debugging
```

## Stdout Exporter (Development)

Good for debugging - see traces in terminal:

```rust
use opentelemetry::trace::TracerProvider;
use opentelemetry_stdout::SpanExporter;
use tracing_subscriber::{layer::SubscriberExt, Registry};
use tracing_opentelemetry::OpenTelemetryLayer;

fn init_tracing() -> anyhow::Result<()> {
    let exporter = SpanExporter::new(std::io::stdout());
    let provider = opentelemetry::sdk::trace::TracerProvider::builder()
        .with_simple_exporter(exporter)
        .build();
    let tracer = provider.tracer("my-service");
    opentelemetry::global::set_tracer_provider(provider);

    let otel_layer = OpenTelemetryLayer::new(tracer);

    let subscriber = Registry::default()
        .with(tracing_subscriber::fmt::layer())  // Console output
        .with(otel_layer);                        // OpenTelemetry export

    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing()?;

    let span = tracing::info_span!("request", request_id = 42);
    async {
        tracing::info!("Processing request");
    }
    .instrument(span)
    .await;

    opentelemetry::global::shutdown_tracer_provider();
    Ok(())
}
```

## OTLP Exporter (Production)

Export to any OTLP-compatible backend (Jaeger, Tempo, etc.):

```rust
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry::runtime::Tokio;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Registry, EnvFilter};
use tracing_opentelemetry::OpenTelemetryLayer;

fn init_tracing() -> anyhow::Result<()> {
    let otlp_exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint("http://localhost:4317");  // OTLP gRPC endpoint

    let tracer_provider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(otlp_exporter)
        .with_trace_config(
            opentelemetry::sdk::trace::Config::default()
                .with_resource(opentelemetry::sdk::Resource::new(vec![
                    opentelemetry::KeyValue::new("service.name", "my-service"),
                    opentelemetry::KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                ]))
        )
        .install_batch(Tokio)?;

    let tracer = tracer_provider.tracer("my-service");

    let otel_layer = OpenTelemetryLayer::new(tracer);

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .with(otel_layer)
        .init();

    Ok(())
}
```

## Jaeger Setup

Run Jaeger locally:
```bash
docker run -d --name jaeger \
  -p 16686:16686 \
  -p 4317:4317 \
  jaegertracing/all-in-one:latest
```

Then configure the OTLP exporter to `http://localhost:4317`.

View traces at `http://localhost:16686`.

## Context Propagation

For distributed tracing across services, propagate trace context via HTTP headers:

```rust
use opentelemetry::propagation::TextMapPropagator;
use opentelemetry_http::HeaderExtractor;
use tracing_opentelemetry::OpenTelemetrySpanExt;

// Extract context from incoming request
fn extract_context(headers: &http::HeaderMap) -> tracing::Span {
    let propagator = opentelemetry::sdk::propagation::TraceContextPropagator::new();
    let context = propagator.extract(&HeaderExtractor(headers));

    let span = tracing::info_span!("incoming_request");
    span.set_parent(context);
    span
}

// Inject context into outgoing request
fn inject_context(span: &tracing::Span, headers: &mut http::HeaderMap) {
    let propagator = opentelemetry::sdk::propagation::TraceContextPropagator::new();
    let context = span.context();
    propagator.inject_context(&context, &mut HeaderInjector(headers));
}
```

## Sampling

Control trace volume in production:

```rust
use opentelemetry::sdk::trace::Sampler;

let config = opentelemetry::sdk::trace::Config::default()
    .with_sampler(Sampler::TraceIdRatioBased(0.1));  // Sample 10%
```

**Sampler options:**
- `AlwaysOn` - Export all traces
- `AlwaysOff` - Export nothing
- `TraceIdRatioBased(f64)` - Probabilistic sampling
- `ParentBased` - Follow parent's decision

## Combining with Metrics

```toml
[dependencies]
opentelemetry = { version = "0.24", features = ["trace", "metrics"] }
opentelemetry-prometheus = "0.16"
```

```rust
use opentelemetry::metrics::MeterProvider;

// Create metrics alongside traces
let meter = opentelemetry::global::meter("my-service");
let counter = meter.u64_counter("requests_total").build();

counter.add(1, &[opentelemetry::KeyValue::new("method", "GET")]);
```

## Shutdown

Always shutdown properly to flush pending exports:

```rust
async fn main() {
    init_tracing()?;

    // ... application code ...

    // Shutdown on exit
    opentelemetry::global::shutdown_tracer_provider();
}
```

## Common Backends

| Backend | Protocol | Endpoint |
|---------|----------|----------|
| Jaeger | OTLP gRPC | localhost:4317 |
| Tempo (Grafana) | OTLP gRPC | localhost:4317 |
| Honeycomb | OTLP HTTP | api.honeycomb.io:443 |
| Datadog | OTLP gRPC | localhost:4317 (agent) |
| Zipkin | Zipkin | localhost:9411 |

## Best Practices

1. **Set service.name resource** - Identifies your service in the backend
2. **Use sampling in production** - Full traces can be expensive
3. **Propagate context** - For cross-service tracing
4. **Batch exports** - Use `install_batch()` not `install_simple()`
5. **Handle shutdown** - Flush traces before exit
