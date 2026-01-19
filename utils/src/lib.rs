use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::SpanExporter;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::{propagation::TraceContextPropagator, Resource};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use opentelemetry_resource_detectors::HostResourceDetector;
use opentelemetry_resource_detectors::OsResourceDetector;
use opentelemetry_semantic_conventions::attribute::HOST_NAME;

pub fn init_logging(){
    tracing_subscriber::registry()
        .with(tracing_subscriber::filter::LevelFilter::from_level(tracing::Level::INFO))
        .with(tracing_subscriber::fmt::layer())
        .init();
}

pub fn init_tracing(resource: Resource) -> SdkTracerProvider {
    let provider = build_tracer_provider(resource);

    // Set the global tracer provider
    global::set_tracer_provider(provider.clone());

    // Set the global text map propagator for context propagation
    global::set_text_map_propagator(TraceContextPropagator::new());

    provider
}

pub fn build_tracer_provider(resource: Resource) -> SdkTracerProvider {
    let exporter = SpanExporter::builder()
        .with_http()
        .build()
        .unwrap();

    // Build the tracer provider with the exporter and resource
    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource)
        .build();
    provider
}

pub fn build_default_resource() -> Resource {
    // Use build with default set of detectors (e.g. to detect attributes from `OTEL_RESOURCE_ATTRIBUTES` env var)
    Resource::builder()
        // Add OS and Host resource detectors (for e.g. `host.arch` and `os.type`)
        .with_detector(Box::new(OsResourceDetector))
        .with_detector(Box::new(HostResourceDetector::default()))
        // Add custom attributes, e.g. `host.name` not added by the host detector
        .with_attribute(
            KeyValue::new(
                HOST_NAME,
                hostname::get()
                    .unwrap_or("unknown-host".into())
                    .to_string_lossy()
                    .into_owned()
            )
        )
        .build()
}