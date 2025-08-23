use crate::auth::User;
use crate::db::BearoData;
use crate::models::{Game, NewGame, UpdateGame};
use mongodb::bson;
use mongodb::{
    bson::{Document, doc, oid::ObjectId},
};
use rocket::form::FromForm;
use rocket::futures::TryStreamExt;
use rocket::serde::{Deserialize, Serialize, json::Json};
use rocket::{delete, get, http::Status, patch, post, put, routes};
use rocket_db_pools::mongodb::options::{FindOptions, UpdateOptions};
use rocket_db_pools::{Connection, mongodb::Collection};
use std::collections::HashMap;

#[derive(FromForm, Debug)]
pub struct GameQuery {
    title: Option<String>,
    developer: Option<String>,
    genre: Option<String>,
    tag: Option<String>,
    status: Option<String>,
    explicit: Option<String>,
    bad: Option<String>,
    #[field(name = "minProgress")]
    min_progress: Option<i32>,
    #[field(name = "maxProgress")]
    max_progress: Option<i32>,
    #[field(name = "exactProgress")]
    exact_progress: Option<i32>,
    #[field(name = "minRating")]
    min_rating: Option<i32>,
    #[field(name = "maxRating")]
    max_rating: Option<i32>,
    #[field(name = "exactRating")]
    exact_rating: Option<i32>,
    sort: Option<String>,
}

#[derive(Deserialize)]
pub struct BulkDeleteFilter {
    developer: Option<String>,
    status: Option<String>,
}

#[derive(Deserialize)]
pub struct BulkUpdatePayload {
    filter: HashMap<String, String>,
    update: HashMap<String, serde_json::Value>,
}

#[derive(Serialize)]
pub struct ApiResponse {
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    deleted: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    updated: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    count: Option<usize>,
}

#[get("/search?<query..>")]
pub async fn get_games(
    db: Connection<BearoData>,
    query: GameQuery,
) -> Result<Json<Vec<Game>>, Status> {
    let collection = db.database("bearodata").collection::<Game>("games");

    let mut filter = Document::new();
    let mut options = FindOptions::default();

    if let Some(title_filter) = &query.title {
        filter.insert("title", doc! { "$regex": title_filter, "$options": "i" });
    }

    if let Some(developer_filter) = &query.developer {
        filter.insert(
            "developer",
            doc! { "$regex": developer_filter, "$options": "i" },
        );
    }

    if let Some(status_filter) = &query.status {
        filter.insert("status", status_filter);
    }

    if let Some(bad_filter) = &query.bad {
        match bad_filter.as_str() {
            "true" => {
                filter.insert("bad", true);
            }
            "false" => {
                filter.insert("bad", false);
            }
            _ => {}
        }
    }

    if let Some(explicit_filter) = &query.explicit {
        match explicit_filter.as_str() {
            "true" => {
                filter.insert("explicit", true);
            }
            "false" => {
                filter.insert("explicit", false);
            }
            _ => {}
        }
    }

    if query.min_rating.is_some() || query.max_rating.is_some() || query.exact_rating.is_some() {
        let mut rating_filter = Document::new();

        if let Some(exact_rating) = query.exact_rating {
            rating_filter.insert("$eq", exact_rating);
        } else {
            if let Some(min_rating) = query.min_rating {
                rating_filter.insert("$gte", min_rating);
            }
            if let Some(max_rating) = query.max_rating {
                rating_filter.insert("$lte", max_rating);
            }
        }
        filter.insert("rating", rating_filter);
    }

    if query.min_progress.is_some()
        || query.max_progress.is_some()
        || query.exact_progress.is_some()
    {
        let mut progress_filter = Document::new();

        if let Some(exact_progress) = query.exact_progress {
            progress_filter.insert("$eq", exact_progress);
        } else {
            if let Some(min_progress) = query.min_progress {
                progress_filter.insert("$gte", min_progress);
            }
            if let Some(max_progress) = query.max_progress {
                progress_filter.insert("$lte", max_progress);
            }
        }
        filter.insert("percent", progress_filter);
    }

    if let Some(sort_by) = &query.sort {
        let sort_doc = match sort_by.as_str() {
            "title" => doc! { "title": 1 },
            "author" => doc! { "developer": 1 },
            "rating" => doc! { "rating": -1 },
            _ => Document::new(),
        };
        if !sort_doc.is_empty() {
            options.sort = Some(sort_doc);
        }
    }

    let mut cursor = collection
        .find(filter, options)
        .await
        .map_err(|_| Status::InternalServerError)?;

    let mut results = Vec::new();
    while let Some(game) = cursor
        .try_next()
        .await
        .map_err(|_| Status::InternalServerError)?
    {
        results.push(game);
    }

    if let Some(genre_filter) = &query.genre {
        results.retain(|game| {
            game.genres.iter().any(|g| {
                if let Some(genre) = g {
                    genre.to_lowercase().contains(&genre_filter.to_lowercase())
                } else {
                    false
                }
            })
        });
    }

    if let Some(tag_filter) = &query.tag {
        results.retain(|game| {
            game.tags.iter().any(|t| {
                if let Some(tag) = t {
                    tag.to_lowercase().contains(&tag_filter.to_lowercase())
                } else {
                    false
                }
            })
        });
    }

    Ok(Json(results))
}

#[get("/<game_id>")]
pub async fn get_game_by_id(
    db: Connection<BearoData>,
    game_id: String,
) -> Result<Json<Game>, Status> {
    let collection = db.database("bearodata").collection::<Game>("games");

    let oid = ObjectId::parse_str(&game_id).map_err(|_| Status::BadRequest)?;

    let game = collection
        .find_one(doc! { "_id": oid }, None)
        .await
        .map_err(|_| Status::InternalServerError)?
        .ok_or(Status::NotFound)?;

    Ok(Json(game))
}

#[post("/", format = "json", data = "<new_game>")]
pub async fn post_games(
    db: Connection<BearoData>,
    user: User,
    new_game: Json<NewGame>,
) -> Result<Json<Game>, Status> {
    user.require_admin().map_err(|_| Status::Forbidden)?;

    let collection: Collection<NewGame> = db.database("bearodata").collection("games");

    let result = collection
        .insert_one(new_game.into_inner(), None)
        .await
        .map_err(|_| Status::InternalServerError)?;

    let game_collection = db.database("bearodata").collection::<Game>("games");
    let inserted_game = game_collection
        .find_one(doc! { "_id": result.inserted_id }, None)
        .await
        .map_err(|_| Status::InternalServerError)?
        .ok_or(Status::InternalServerError)?;

    Ok(Json(inserted_game))
}

#[put("/<game_id>", format = "json", data = "<updated_game>")]
pub async fn update_game(
    db: Connection<BearoData>,
    user: User,
    game_id: String,
    updated_game: Json<UpdateGame>,
) -> Result<Json<Game>, Status> {
    user.require_admin().map_err(|_| Status::Forbidden)?;

    let collection = db.database("bearodata").collection::<Game>("games");
    let oid = ObjectId::parse_str(&game_id).map_err(|_| Status::BadRequest)?;

    let update_doc = mongodb::bson::to_document(&updated_game.into_inner())
        .map_err(|_| Status::InternalServerError)?;

    let options = UpdateOptions::builder().upsert(false).build();

    collection
        .update_one(doc! { "_id": oid }, doc! { "$set": update_doc }, options)
        .await
        .map_err(|_| Status::InternalServerError)?;

    let updated_game = collection
        .find_one(doc! { "_id": oid }, None)
        .await
        .map_err(|_| Status::InternalServerError)?
        .ok_or(Status::NotFound)?;

    Ok(Json(updated_game))
}

#[patch("/<game_id>", format = "json", data = "<patch_data>")]
pub async fn patch_game(
    db: Connection<BearoData>,
    user: User,
    game_id: String,
    patch_data: Json<UpdateGame>,
) -> Result<Json<Game>, Status> {
    user.require_admin().map_err(|_| Status::Forbidden)?;

    let collection = db.database("bearodata").collection::<Game>("games");
    let oid = ObjectId::parse_str(&game_id).map_err(|_| Status::BadRequest)?;

    let mut update_doc = Document::new();
    let patch = patch_data.into_inner();

    if let Some(title) = patch.title {
        update_doc.insert("title", title);
    }
    if let Some(developer) = patch.developer {
        update_doc.insert("developer", developer);
    }
    if let Some(genres) = patch.genres {
        update_doc.insert("genres", genres);
    }
    if let Some(tags) = patch.tags {
        update_doc.insert("tags", tags);
    }
    if let Some(rating) = patch.rating {
        update_doc.insert("rating", rating);
    }
    if let Some(status) = patch.status {
        update_doc.insert("status", status);
    }
    if let Some(description) = patch.description {
        update_doc.insert("description", description);
    }
    if let Some(my_thoughts) = patch.my_thoughts {
        update_doc.insert("my_thoughts", my_thoughts);
    }
    if let Some(links) = patch.links {
        update_doc.insert("links", bson::to_bson(&links).expect("Failed to parse"));
    }
    if let Some(cover_image) = patch.cover_image {
        update_doc.insert("cover_image", cover_image);
    }
    if let Some(explicit) = patch.explicit {
        update_doc.insert("explicit", explicit);
    }
    if let Some(percent) = patch.percent {
        update_doc.insert("percent", percent);
    }
    if let Some(bad) = patch.bad {
        update_doc.insert("bad", bad);
    }

    collection
        .update_one(doc! { "_id": oid }, doc! { "$set": update_doc }, None)
        .await
        .map_err(|_| Status::InternalServerError)?;

    let updated_game = collection
        .find_one(doc! { "_id": oid }, None)
        .await
        .map_err(|_| Status::InternalServerError)?
        .ok_or(Status::NotFound)?;

    Ok(Json(updated_game))
}

#[delete("/<game_id>")]
pub async fn delete_game(
    db: Connection<BearoData>,
    user: User,
    game_id: String,
) -> Result<Json<ApiResponse>, Status> {
    user.require_admin().map_err(|_| Status::Forbidden)?;

    let collection = db.database("bearodata").collection::<Game>("games");
    let oid = ObjectId::parse_str(&game_id).map_err(|_| Status::BadRequest)?;

    let result = collection
        .delete_one(doc! { "_id": oid }, None)
        .await
        .map_err(|_| Status::InternalServerError)?;

    if result.deleted_count > 0 {
        Ok(Json(ApiResponse {
            message: "game deleted".to_string(),
            deleted: None,
            updated: None,
            count: None,
        }))
    } else {
        Err(Status::NotFound)
    }
}

#[delete("/bulk", format = "json", data = "<filter>")]
pub async fn bulk_delete_games(
    db: Connection<BearoData>,
    user: User,
    filter: Json<BulkDeleteFilter>,
) -> Result<Json<ApiResponse>, Status> {
    user.require_admin().map_err(|_| Status::Forbidden)?;

    let collection = db.database("bearodata").collection::<Game>("games");
    let filter = filter.into_inner();

    let mut delete_filter = Document::new();

    if let Some(developer_filter) = &filter.developer {
        delete_filter.insert("developer", developer_filter);
    }

    if let Some(status_filter) = &filter.status {
        delete_filter.insert("status", status_filter);
    }

    let result = collection
        .delete_many(delete_filter, None)
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Json(ApiResponse {
        message: "bulk delete complete".to_string(),
        deleted: Some(result.deleted_count as i64),
        updated: None,
        count: None,
    }))
}

#[patch("/bulk", format = "json", data = "<payload>")]
pub async fn bulk_update_games(
    db: Connection<BearoData>,
    user: User,
    payload: Json<BulkUpdatePayload>,
) -> Result<Json<ApiResponse>, Status> {
    user.require_admin().map_err(|_| Status::Forbidden)?;

    let collection = db.database("bearodata").collection::<Game>("games");
    let payload = payload.into_inner();

    let mut filter_doc = Document::new();

    if let Some(developer_filter) = payload.filter.get("developer") {
        filter_doc.insert("developer", developer_filter);
    }

    if let Some(status_filter) = payload.filter.get("status") {
        filter_doc.insert("status", status_filter);
    }

    let mut update_doc = Document::new();

    if let Some(new_status) = payload.update.get("status").and_then(|v| v.as_str()) {
        update_doc.insert("status", new_status);
    }

    if let Some(new_rating) = payload.update.get("rating").and_then(|v| v.as_f64()) {
        update_doc.insert("rating", new_rating as i32);
    }

    let result = collection
        .update_many(filter_doc, doc! { "$set": update_doc }, None)
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Json(ApiResponse {
        message: "bulk update complete".to_string(),
        deleted: None,
        updated: Some(result.modified_count as i64),
        count: None,
    }))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_games,
        get_game_by_id,
        post_games,
        update_game,
        patch_game,
        delete_game,
        bulk_delete_games,
        bulk_update_games
    ]
}
