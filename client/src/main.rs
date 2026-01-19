use std::env;
use std::time::Duration;
use utils::{init_logging, init_tracing, build_default_resource};
use opentelemetry::{trace::{SpanKind, TraceContextExt, Tracer}, Value};
use opentelemetry_http::HeaderInjector;
use hyper_util::{client::legacy::Client, rt::TokioExecutor};
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::Method;
use tracing::info;
use opentelemetry::{global, Context, KeyValue};
use opentelemetry::trace::Status;
use opentelemetry_semantic_conventions::trace::{HTTP_REQUEST_METHOD, HTTP_RESPONSE_STATUS_CODE, SERVER_ADDRESS, SERVER_PORT, URL_FULL};
use url::Url;

#[tokio::main]
async fn main()-> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>  {
    init_logging();

    let server_url = env::var("SERVER_URL").unwrap_or("http://localhost:8080".to_string());
    let url = format!("{}/posts", server_url);
    info!("Starting client, sending requests to {}", url);

    // Initialize the OpenTelemetry tracer provider
    let resource = build_default_resource();
    let provider = init_tracing(resource);

    for _i in 0..50 {
        send_request(&url, Method::GET).await?;
        tokio::time::sleep(Duration::from_secs(5)).await;
    }

    // Shutdown OTel tracer
    info!("Shutting down server and tracing");
    provider.shutdown().expect("failed to shutdown client tracer");
    Ok(())
}


async fn send_request(url: &str, method: Method) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let client = Client::builder(TokioExecutor::new()).build_http();
    let tracer = global::tracer("example/client");
    let parsed_url = Url::parse(url)?;

    // Create span for the outgoing HTTP request, with attributes required by the semantic conventions:
    // https://opentelemetry.io/docs/specs/semconv/http/http-spans/#http-client-span

    let required_http_client_attributes = vec![
        KeyValue::new(HTTP_REQUEST_METHOD, method.to_string()),
        KeyValue::new(URL_FULL, parsed_url.to_string()),
        KeyValue::new(SERVER_ADDRESS, parsed_url.host().unwrap().to_string()),
        KeyValue::new(SERVER_PORT, Value::I64(parsed_url.port().unwrap() as i64)),
    ];

    let span = tracer
        .span_builder(method.to_string())
        .with_kind(SpanKind::Client)
        .with_attributes(required_http_client_attributes)
        .start(&tracer);
    let cx = Context::current_with_span(span);

    let mut req = hyper::Request::builder()
        .method(method)
        .uri(url);

    global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&cx, &mut HeaderInjector(req.headers_mut().unwrap()))
    });

    let mut res = client
        .request(req.body(Full::new(Bytes::new()))?)
        .await?;

    let res_status = res.status();
    let res_body = res.body_mut().collect().await?.to_bytes();

    let span = cx.span();
    span.set_attribute(KeyValue::new(HTTP_RESPONSE_STATUS_CODE, res_status.as_u16() as i64));

    if !res_status.is_success(){
        span.set_status(Status::error(format!("HTTP request failed with status code {}", res_status)));
    }
    let response_body = String::from_utf8(res_body.to_vec())?.to_string();

    info!(name: "ResponseReceived", status = res_status.to_string(), message = response_body);
    Ok(())
}