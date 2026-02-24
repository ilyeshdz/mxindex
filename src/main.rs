#[macro_use] extern crate rocket;

mod db;
mod models;
mod routes;
mod schema;
mod services;

use db::{establish_connection, run_migrations};
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};
use rocket_okapi::openapi_get_routes;

#[launch]
fn rocket() -> _ {
    dotenvy::dotenv().ok();

    let mut conn = establish_connection();
    run_migrations(&mut conn);

    rocket::build()
        .mount("/", openapi_get_routes![
            routes::index,
            routes::server_info,
            routes::add_server,
            routes::list_servers
        ])
        .mount("/swagger", make_swagger_ui(&SwaggerUIConfig {
            url: "/openapi.json".to_string(),
            ..Default::default()
        }))
}
