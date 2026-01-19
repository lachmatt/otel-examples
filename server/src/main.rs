use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use once_cell::sync::Lazy;
use opentelemetry_instrumentation_actix_web::RequestTracing;
use opentelemetry_sdk::Resource;
use utils::{init_logging, init_tracing};
use tracing::info;
use tokio::time::sleep;

static RESOURCE: Lazy<Resource> = Lazy::new(|| {
    Resource::builder()
        .build()
});


#[actix_web::main]
async fn main() -> std::io::Result<()> {

    init_logging();
    // Initialize the OpenTelemetry tracer provider
    let provider = init_tracing(RESOURCE.clone());

    HttpServer::new(move|| {
        App::new()
            //Add instrumentation for HTTP requests
            .wrap(RequestTracing::new())
            .service(get_posts)
    })
        .bind(("0.0.0.0",8080))?
        .run()
        .await?;

    // Shutdown OTel tracer
    info!("Shutting down server and tracing");
    provider.shutdown().expect("failed to shutdown server tracer");

    Ok(())
}

#[get("/posts")]
async fn get_posts() -> impl Responder{

    info!("Serving request to /posts endpoint");
    // Simulate doing some time-consuming work
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    HttpResponse::InternalServerError()
}