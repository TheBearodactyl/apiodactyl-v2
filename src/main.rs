//! # Apiodactyl - A RESTful API for data management on `https://bearodactyl.dev`
//!
//! ## Environment Variables
//!
//! - `DATABASE_URL` or `MONGODB_URL`: MongoDB connection string
//! - `BOOTSTRAP_ADMIN_KEY`: Initial admin API key (optional, for first-time setup)
#![feature(duration_constructors, str_as_str)]

use rocket::{catchers, http::Method, launch, routes};
use rocket_cors::{AllowedOrigins, CorsOptions};
use rocket_db_pools::Database;

use crate::{auth::AuthService, db::BearoData};

pub mod auth;
pub mod cli;
pub mod db;
pub mod errors;
pub mod handlers;
pub mod models;

/// Main entry point for the Rocket application.
///
/// Initializes the authentication service, database connection, and CORS configuration.
/// If command-line arguments are provided, handles CLI commands before starting the server.
///
/// # Returns
///
/// A configured Rocket instance ready for launch.
#[launch]
async fn rocket() -> _ {
    let auth_service = AuthService::new();
    let db = BearoData::init();
    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(
            vec![
                Method::Get,
                Method::Post,
                Method::Patch,
                Method::Put,
                Method::Delete,
            ]
            .into_iter()
            .map(From::from)
            .collect(),
        )
        .allow_credentials(true);

    if std::env::args().len() > 1 {
        cli::handle_cli(auth_service.clone())
            .await
            .expect("Failed to handle CLI");
    }

    rocket::build()
        .manage(auth_service)
        .attach(db)
        .attach(cors.to_cors().expect("Failed to build cors"))
        .register(
            "/",
            catchers![handlers::catch401, handlers::catch404, handlers::catch500],
        )
        .mount("/", routes![handlers::index])
        .mount("/reviews", handlers::reviews::routes())
        .mount(
            "/wplace",
            routes![
                handlers::wplace::get_screenshot_by_id,
                handlers::wplace::get_screenshots,
                handlers::wplace::create_screenshot,
                handlers::wplace::delete_screenshot
            ],
        )
        .mount("/read-watch", handlers::books::routes())
        .mount("/games", handlers::games::routes())
        .mount("/projects", handlers::projects::routes())
        .mount("/misc", handlers::misc::routes())
}
