use rocket::fairing::AdHoc;
use rocket::Rocket;
use rocket::Build;
use rocket_okapi::openapi;
use std::sync::{Arc, RwLock};

#[macro_use]
extern crate rocket;

mod app;
mod cache;
mod db;
mod federation_discovery;
mod http_client;
mod metrics;
mod models;
mod rate_limit;
mod routes;
mod schema;
mod services;

use cache::Cache;
use db::{create_pool, establish_connection, run_migrations};
use metrics::Metrics;
use rate_limit::rate_limiter_from_config;
use rocket_okapi::openapi_get_routes;
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};
use tracing::{info, warn};

use app::AppState;

#[openapi]
#[get("/metrics")]
fn metrics_endpoint(
    metrics: &rocket::State<Arc<RwLock<Metrics>>>,
) -> String {
    let metrics = metrics.read().unwrap();
    metrics.encode()
}

#[launch]
fn rocket() -> Rocket<Build> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    dotenvy::dotenv().ok();

    let mut conn = establish_connection();
    run_migrations(&mut conn);

    let db_pool = create_pool();
    let cache = Arc::new(Cache::new());
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let metrics = Metrics::new();
    let rate_limiter = rate_limiter_from_config();

    rocket::build()
        .manage(AppState {
            cache: cache.clone(),
            db_pool,
        })
        .manage(metrics)
        .manage(rate_limiter)
        .attach(AdHoc::on_liftoff("Redis Connection", move |_rocket| {
            let cache = cache.clone();
            Box::pin(async move {
                match cache.connect(&redis_url).await {
                    Ok(_) => info!("Connected to Redis at {}", redis_url),
                    Err(e) => warn!(
                        "Failed to connect to Redis: {}. Caching disabled.",
                        e
                    ),
                }
            })
        }))
        .mount(
            "/",
            openapi_get_routes![
                routes::index,
                routes::server_info,
                routes::add_server,
                routes::list_servers,
                routes::search_servers,
                routes::health,
                routes::discover_federation,
                metrics_endpoint
            ],
        )
        .mount(
            "/swagger",
            make_swagger_ui(&SwaggerUIConfig {
                url: "/openapi.json".to_string(),
                ..Default::default()
            }),
        )
}
