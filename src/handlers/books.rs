use {
    crate::{
        auth::User,
        db::BearoData,
        models::{Book, NewBook, UpdateBook},
    },
    mongodb::bson::{self, Document, doc, oid::ObjectId},
    rocket::{
        Route, delete,
        form::FromForm,
        futures::TryStreamExt,
        get,
        http::Status,
        patch, post, put, routes,
        serde::{Deserialize, Serialize, json::Json},
    },
    rocket_db_pools::{
        Connection,
        mongodb::{
            Collection,
            options::{FindOptions, UpdateOptions},
        },
    },
    std::collections::HashMap,
};

#[derive(FromForm, Debug)]
pub struct BookQuery {
    title: Option<String>,
    author: Option<String>,
    genre: Option<String>,
    tag: Option<String>,
    status: Option<String>,
    explicit: Option<String>,
    #[field(name = "minRating")]
    min_rating: Option<i32>,
    #[field(name = "maxRating")]
    max_rating: Option<i32>,
    sort: Option<String>,
}

#[derive(Deserialize)]
pub struct BulkDeleteFilter {
    author: Option<String>,
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
pub async fn get_books(
    db: Connection<BearoData>,
    query: BookQuery,
) -> Result<Json<Vec<Book>>, Status> {
    let collection: Collection<Book> = db.database("bearodata").collection("books");
    let mut filter = Document::new();
    let mut options = FindOptions::default();

    if let Some(title_filter) = &query.title {
        filter.insert("title", doc! { "$regex": title_filter, "$options": "i" });
    }

    if let Some(author_filter) = &query.author {
        filter.insert("author", doc! { "$regex": author_filter, "$options": "i" });
    }

    if let Some(status_filter) = &query.status {
        filter.insert("status", status_filter);
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

    if query.min_rating.is_some() || query.max_rating.is_some() {
        let mut rating_filter = Document::new();
        if let Some(min_rating) = query.min_rating {
            rating_filter.insert("$gte", min_rating);
        }
        if let Some(max_rating) = query.max_rating {
            rating_filter.insert("$lte", max_rating);
        }
        filter.insert("rating", rating_filter);
    }

    if let Some(sort_by) = &query.sort {
        let sort_doc = match sort_by.as_str() {
            "title" => doc! { "title": 1 },
            "author" => doc! { "author": 1 },
            "rating" => doc! { "rating": -1 },
            _ => Document::new(),
        };
        if !sort_doc.is_empty() {
            options.sort = Some(sort_doc);
        }
    }

    let mut cursor = collection
        .find(filter, Some(options))
        .await
        .map_err(|_| Status::InternalServerError)?;

    let mut results = Vec::new();
    while let Some(book) = cursor
        .try_next()
        .await
        .map_err(|_| Status::InternalServerError)?
    {
        results.push(book);
    }

    if let Some(genre_filter) = &query.genre {
        results.retain(|book| {
            book.genres.iter().any(|g| {
                if let Some(genre) = g {
                    genre.to_lowercase().contains(&genre_filter.to_lowercase())
                } else {
                    false
                }
            })
        });
    }

    if let Some(tag_filter) = &query.tag {
        results.retain(|book| {
            book.tags.iter().any(|t| {
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

#[get("/<book_id>")]
pub async fn get_book_by_id(
    db: Connection<BearoData>,
    book_id: String,
) -> Result<Json<Book>, Status> {
    let collection = db.database("bearodata").collection::<Book>("books");

    let oid = ObjectId::parse_str(&book_id).map_err(|_| Status::BadRequest)?;

    let book = collection
        .find_one(doc! { "_id": oid }, None)
        .await
        .map_err(|_| Status::InternalServerError)?
        .ok_or(Status::NotFound)?;

    Ok(Json(book))
}

#[post("/", format = "json", data = "<new_book>")]
pub async fn post_books(
    db: Connection<BearoData>,
    user: User,
    new_book: Json<NewBook>,
) -> Result<Json<Book>, Status> {
    user.require_admin().map_err(|_| Status::Forbidden)?;

    let collection: Collection<NewBook> = db.database("bearodata").collection("books");

    let result = collection
        .insert_one(new_book.into_inner(), None)
        .await
        .map_err(|_| Status::InternalServerError)?;

    let book_collection = db.database("bearodata").collection::<Book>("books");
    let inserted_book = book_collection
        .find_one(doc! { "_id": result.inserted_id }, None)
        .await
        .map_err(|_| Status::InternalServerError)?
        .ok_or(Status::InternalServerError)?;

    Ok(Json(inserted_book))
}

#[put("/<book_id>", format = "json", data = "<updated_book>")]
pub async fn update_book(
    db: Connection<BearoData>,
    user: User,
    book_id: String,
    updated_book: Json<UpdateBook>,
) -> Result<Json<Book>, Status> {
    user.require_admin().map_err(|_| Status::Forbidden)?;

    let collection = db.database("bearodata").collection::<Book>("books");
    let oid = ObjectId::parse_str(&book_id).map_err(|_| Status::BadRequest)?;

    let update_doc = mongodb::bson::to_document(&updated_book.into_inner())
        .map_err(|_| Status::InternalServerError)?;

    let options = UpdateOptions::builder().upsert(false).build();

    collection
        .update_one(doc! { "_id": oid }, doc! { "$set": update_doc }, options)
        .await
        .map_err(|_| Status::InternalServerError)?;

    let updated_book = collection
        .find_one(doc! { "_id": oid }, None)
        .await
        .map_err(|_| Status::InternalServerError)?
        .ok_or(Status::NotFound)?;

    Ok(Json(updated_book))
}

#[patch("/<book_id>", format = "json", data = "<patch_data>")]
pub async fn patch_book(
    db: Connection<BearoData>,
    user: User,
    book_id: String,
    patch_data: Json<UpdateBook>,
) -> Result<Json<Book>, Status> {
    user.require_admin().map_err(|_| Status::Forbidden)?;

    let collection = db.database("bearodata").collection::<Book>("books");
    let oid = ObjectId::parse_str(&book_id).map_err(|_| Status::BadRequest)?;

    let mut update_doc = Document::new();
    let patch = patch_data.into_inner();

    if let Some(title) = patch.title {
        update_doc.insert("title", title);
    }
    if let Some(author) = patch.author {
        update_doc.insert("author", author);
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
        update_doc.insert(
            "links",
            bson::to_bson(&links).expect("Failed to convert links to `bson`"),
        );
    }
    if let Some(cover_image) = patch.cover_image {
        update_doc.insert("cover_image", cover_image);
    }
    if let Some(explicit) = patch.explicit {
        update_doc.insert("explicit", explicit);
    }
    if let Some(color) = patch.color {
        update_doc.insert("color", color);
    }

    collection
        .update_one(doc! { "_id": oid }, doc! { "$set": update_doc }, None)
        .await
        .map_err(|_| Status::InternalServerError)?;

    let updated_book = collection
        .find_one(doc! { "_id": oid }, None)
        .await
        .map_err(|_| Status::InternalServerError)?
        .ok_or(Status::NotFound)?;

    Ok(Json(updated_book))
}

#[delete("/<book_id>")]
pub async fn delete_book(
    db: Connection<BearoData>,
    _user: User,
    book_id: String,
) -> Result<Json<ApiResponse>, Status> {
    _user.require_admin().map_err(|_| Status::Forbidden)?;

    let collection = db.database("bearodata").collection::<Book>("books");
    let oid = ObjectId::parse_str(&book_id).map_err(|_| Status::BadRequest)?;

    let result = collection
        .delete_one(doc! { "_id": oid }, None)
        .await
        .map_err(|_| Status::InternalServerError)?;

    if result.deleted_count > 0 {
        Ok(Json(ApiResponse {
            message: "book deleted".to_string(),
            deleted: None,
            updated: None,
            count: None,
        }))
    } else {
        Err(Status::NotFound)
    }
}

#[delete("/bulk", format = "json", data = "<filter>")]
pub async fn bulk_delete_books(
    db: Connection<BearoData>,
    _user: User,
    filter: Json<BulkDeleteFilter>,
) -> Result<Json<ApiResponse>, Status> {
    _user.require_admin().map_err(|_| Status::Forbidden)?;

    let collection = db.database("bearodata").collection::<Book>("books");
    let filter = filter.into_inner();

    let mut delete_filter = Document::new();

    if let Some(author_filter) = &filter.author {
        delete_filter.insert("author", author_filter);
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
pub async fn bulk_update_books(
    db: Connection<BearoData>,
    _user: User,
    payload: Json<BulkUpdatePayload>,
) -> Result<Json<ApiResponse>, Status> {
    _user.require_admin().map_err(|_| Status::Forbidden)?;

    let collection = db.database("bearodata").collection::<Book>("books");
    let payload = payload.into_inner();

    let mut filter_doc = Document::new();

    if let Some(author_filter) = payload.filter.get("author") {
        filter_doc.insert("author", author_filter);
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

pub fn routes() -> Vec<Route> {
    routes![
        get_books,
        get_book_by_id,
        post_books,
        update_book,
        patch_book,
        delete_book,
        bulk_delete_books,
        bulk_update_books
    ]
}
