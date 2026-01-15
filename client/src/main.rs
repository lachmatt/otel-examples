use std::env;
use std::time::Duration;
use utils::{init_logging, init_tracing};
use opentelemetry::{trace::{SpanKind, TraceContextExt, Tracer}, Value};
use opentelemetry_http::HeaderInjector;
use hyper_util::{client::legacy::Client, rt::TokioExecutor};
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper_rustls::HttpsConnectorBuilder;
use tracing::info;
use opentelemetry::{global, Context, KeyValue};
use opentelemetry_sdk::Resource;
use once_cell::sync::Lazy;
use opentelemetry::trace::Status;
use url::Url;

static RESOURCE: Lazy<Resource> = Lazy::new(|| {
    Resource::builder()
        .build()
});

#[tokio::main]
async fn main()-> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>  {
    let server_url = env::var("SERVER_URL").unwrap_or("http://localhost:8080".to_string());
    init_logging();
    let provider = init_tracing(RESOURCE.clone());

    let url = format!("{}/posts", server_url);
    for _i in 0..50 {
        send_request(&url).await?;
        tokio::time::sleep(Duration::from_secs(5)).await;
    }

    // Shutdown OTel tracer
    info!("Shutting down server and tracing");
    provider.shutdown().expect("failed to shutdown client tracer");
    Ok(())
}


async fn send_request(url: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let connector = HttpsConnectorBuilder::new()
        .with_native_roots()?
        .https_or_http()
        .enable_http1()
        .build();
    let client = Client::builder(TokioExecutor::new()).build(connector);
    let tracer = global::tracer("example/client");
    let u = Url::parse(url)?;
    let attributes = vec![
        KeyValue::new("http.request.method", "GET"),
        KeyValue::new("url.full", u.to_string()),
        KeyValue::new("server.address", u.host().unwrap().to_string()),
        KeyValue::new("server.port", Value::I64(u.port().unwrap() as i64)),
    ];
    let span = tracer
        .span_builder("GET")
        .with_kind(SpanKind::Client)
        .with_attributes(attributes)
        .start(&tracer);
    let cx = Context::current_with_span(span);

    let mut req = hyper::Request::builder().uri(url);
    global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&cx, &mut HeaderInjector(req.headers_mut().unwrap()))
    });

    let mut res = client
        .request(req.body(Full::new(Bytes::new()))?)
        .await?;

    let res_status = res.status();
    let collect = res.body_mut().collect().await?.to_bytes();

    let span = cx.span();
    span.set_attribute(KeyValue::new("http.response.status_code", res_status.as_u16() as i64));
    if !res_status.is_success(){
        span.set_status(Status::error(format!("HTTP request failed with status code {}", res_status)));
    }
    let response_body = String::from_utf8(collect.to_vec())?.to_string();

    info!(name: "ResponseReceived", status = res_status.to_string(), message = response_body);
    Ok(())
}