use axum::{routing, Router};
use axum_prometheus::{metrics_exporter_prometheus::PrometheusHandle, PrometheusMetricLayer};
use dotenvy::dotenv;
use tower_http::cors::{Any, CorsLayer};
use tracing::*;

mod listen;

pub type Result<T, E = anyhow::Error> = std::result::Result<T, E>;

#[tokio::main]
async fn main() -> Result<()> {
    // Tracing
    register_tracing();

    // Now trace the started message
    info!("Started Appointment Scheduler");

    // Metrics
    let (prometheus_layer, metric_handle) = define_metrics();

    // Do DB stuff (but does nothing at the moment I guess)
    dotenv().ok();
    //db_infra_pool::init();

    // Do Route stuff
    let router = create_router_with_prometheus(prometheus_layer, metric_handle);

    // Start Web Server at port 8080
    use tokio::signal::unix as usig;
    let mut shutdown = usig::signal(usig::SignalKind::terminate())?;
    let server = axum::Server::bind(&std::net::SocketAddr::from(([0, 0, 0, 0], 8081)))
        .serve(router.into_make_service())
        .with_graceful_shutdown(async move {
            shutdown.recv().await;
        });

    // Wait for either Watcher or WebServer to exit and write log hwo exited (the Watcher should never exit by its own)
    tokio::select! {
        _ = server => info!("axum server exited"),
    }

    // Finish
    Ok(())
}

fn register_tracing() {
    tracing_subscriber::fmt()
        .json()
        .flatten_event(true)
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_level(true)
        .with_max_level(Level::INFO)
        .init();
}

fn define_metrics() -> (PrometheusMetricLayer<'static>, PrometheusHandle) {
    axum_prometheus::PrometheusMetricLayer::pair()
}

fn create_router_with_prometheus(
    prometheus_layer: PrometheusMetricLayer<'static>,
    metric_handle: PrometheusHandle,
) -> Router {
    create_router()
        .route(
            "/actuator/prometheus",
            routing::get(|| async move { metric_handle.render() }),
        )
        .layer(prometheus_layer)
}

fn create_router() -> Router {
    let cors = CorsLayer::new().allow_origin(Any).allow_headers(Any);

    Router::new()
        // Here the business routes later
        //.layer(Extension(reader_deployment))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        // Reminder: routes added *after* TraceLayer are not subject to its logging behavior
        .route("/actuator/health", routing::get(listen::health))
        .layer(cors)
}
