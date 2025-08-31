use mongodb::bson::doc;
use rocket::{get, http::Status, routes as rocket_routes, serde::json::Json};
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};

use crate::db::BearoData;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct CollectionStatus {
    pub books: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct HealthStatus {
    pub db_status: String,
    pub collections_status: CollectionStatus,
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

pub fn routes() -> Vec<rocket::Route> {
    rocket_routes![health]
}
