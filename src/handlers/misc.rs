use mongodb::bson::doc;
use rocket::{get, http::Status, routes as rocket_routes, serde::json::Json};
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};

use crate::{auth::User, db::BearoData};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct CollectionStatus {
    pub books: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct HealthStatus {
    pub db_status: String,
    pub collections_status: CollectionStatus,
}

#[derive(Serialize)]
pub struct CheckStatusResponse {
    valid: bool,
    is_admin: bool,
    user_id: String,
    created_at: String,
    last_used_at: Option<String>,
}

#[get("/check-health")]
pub async fn health(db: Connection<BearoData>) -> Result<Json<HealthStatus>, Status> {
    let mut collections_status = CollectionStatus::default();
    let mut health = HealthStatus::default();

    if db
        .database("bearodata")
        .list_collection_names(doc! {})
        .await
        .is_ok()
    {
        health.db_status = "database online!".to_string();

        let collections = db
            .database("bearodata")
            .list_collection_names(doc! {})
            .await
            .expect("Failed to get collection names");

        collections.clone().iter().for_each(|col| {
            let collection = col.to_owned();
            if collection == "books" {
                collections_status.books = "Books collection online!".to_string();
            }
        });
    } else {
        health.db_status = "database offline :(".to_string();
    }

    Ok(Json(health))
}

#[get("/check-login")]
pub async fn check_admin_status(user: User) -> Json<CheckStatusResponse> {
    Json(CheckStatusResponse {
        valid: true,
        is_admin: user.is_admin(),
        user_id: user.id().to_string(),
        created_at: user.created_at().format("%Y-%m-%d %H:%M:%S").to_string(),
        last_used_at: user
            .last_used_at()
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
    })
}

pub fn routes() -> Vec<rocket::Route> {
    rocket_routes![health, check_admin_status]
}
