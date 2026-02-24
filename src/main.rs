use rocket::fairing::AdHoc;

#[macro_use]
extern crate rocket;

mod app;
mod cache;
mod db;
mod models;
mod routes;
mod schema;
mod services;

use cache::Cache;
use db::{establish_connection, run_migrations};
use rocket_okapi::openapi_get_routes;
use rocket_okapi::swagger_ui::{SwaggerUIConfig, make_swagger_ui};
use std::sync::Arc;

use app::AppState;

#[launch]
fn rocket() -> _ {
    dotenvy::dotenv().ok();

    let mut conn = establish_connection();
    run_migrations(&mut conn);

    let cache = Arc::new(Cache::new());
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    rocket::build()
        .manage(AppState {
            cache: cache.clone(),
        })
        .attach(AdHoc::on_liftoff("Redis Connection", move |_rocket| {
            let cache = cache.clone();
            Box::pin(async move {
                match cache.connect(&redis_url).await {
                    Ok(_) => println!("Connected to Redis at {}", redis_url),
                    Err(e) => eprintln!(
                        "Warning: Failed to connect to Redis: {}. Caching disabled.",
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
                routes::list_servers
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
