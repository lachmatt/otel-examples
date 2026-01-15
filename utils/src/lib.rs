// OpenTelemetry imports
use opentelemetry::global;
use opentelemetry_otlp::SpanExporter;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::{propagation::TraceContextPropagator, Resource};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

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
        // .with_protocol(Protocol::HttpBinary)
        .build()
        .unwrap();

    // Build the tracer provider with the exporter and resource
    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource)
        .build();
    provider
}
