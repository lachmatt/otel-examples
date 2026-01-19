use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use opentelemetry_instrumentation_actix_web::RequestTracing;
use utils::{init_logging, init_tracing, build_default_resource};
use tracing::info;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_logging();

    // Initialize the OpenTelemetry tracer provider
    let resource = build_default_resource();
    let provider = init_tracing(resource);

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