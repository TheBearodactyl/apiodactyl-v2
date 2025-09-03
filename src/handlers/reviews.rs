use {
    crate::{
        db::BearoData,
        models::{NewReview, Review, UpdateReview},
    },
    mongodb::bson::{self, doc, oid::ObjectId},
    rocket::{
        delete, futures::StreamExt, get, http::Status, patch, post, response::status, routes,
        serde::json::Json,
    },
    rocket_db_pools::Connection,
};

#[post("/", data = "<review>")]
pub async fn create_review(
    db: Connection<BearoData>,
    review: Json<NewReview>,
) -> Result<Json<Review>, status::Custom<String>> {
    let collection = db.database("bearodata").collection::<Review>("reviews");

    let new_review = Review {
        oid: ObjectId::new(),
        chapter: review.chapter,
        description: review.description.clone(),
        rating: review.rating,
        thoughts: review.thoughts.clone(),
    };

    if collection
        .find_one(doc! { "chapter": new_review.chapter }, None)
        .await
        .expect("Failed")
        .is_none()
    {
        collection
            .insert_one(new_review.clone(), None)
            .await
            .expect("Failed to insert review");

        Ok(Json(new_review))
    } else {
        Err(status::Custom(
            Status::Conflict,
            format!("Review for chapter {} already found", new_review.chapter),
        ))
    }
}

#[get("/<id>")]
pub async fn get_review_by_oid(db: Connection<BearoData>, id: &str) -> Option<Json<Review>> {
    let collection = db.database("bearodata").collection::<Review>("reviews");
    let oid = ObjectId::parse_str(id).expect("Failed to parse oid");
    let found_review = collection
        .find_one(doc! { "_id": oid }, None)
        .await
        .unwrap();

    Some(Json(found_review.unwrap()))
}

#[get("/<chapter>", rank = 2)]
pub async fn get_review_by_chapter(
    db: Connection<BearoData>,
    chapter: i32,
) -> Option<Json<Review>> {
    let collection = db.database("bearodata").collection::<Review>("reviews");
    let found_review = collection
        .find_one(doc! { "chapter": chapter }, None)
        .await
        .unwrap();

    Some(Json(found_review.unwrap()))
}

#[get("/")]
pub async fn get_reviews(db: Connection<BearoData>) -> Option<Json<Vec<Review>>> {
    let collection = db.database("bearodata").collection::<Review>("reviews");

    let mut cursor = match collection.find(doc! {}, None).await {
        Ok(cursor) => cursor,
        Err(_) => return None,
    };

    let mut reviews = Vec::new();
    while let Some(result) = cursor.next().await {
        match result {
            Ok(review) => reviews.push(review),
            Err(_) => return None,
        }
    }

    Some(Json(reviews))
}

#[patch("/<chapter>", format = "json", data = "<update_data>")]
pub async fn patch_review_by_chapter(
    db: Connection<BearoData>,
    chapter: i32,
    update_data: Json<UpdateReview>,
) -> Result<Json<Review>, status::Custom<String>> {
    let collection = db.database("bearodata").collection::<Review>("reviews");

    let update_doc = match bson::to_document(&update_data.into_inner()) {
        Ok(doc) => doc,
        Err(e) => {
            return Err(status::Custom(
                Status::BadRequest,
                format!("Invalid update data: {}", e),
            ));
        }
    };

    match collection
        .find_one_and_update(
            doc! { "chapter": chapter },
            doc! { "$set": update_doc },
            None,
        )
        .await
    {
        Ok(Some(updated_review)) => Ok(Json(updated_review)),
        Ok(None) => Err(status::Custom(
            Status::NotFound,
            "No review found for this chapter".into(),
        )),
        Err(e) => Err(status::Custom(
            Status::InternalServerError,
            format!("Failed to update review: {}", e),
        )),
    }
}

#[patch("/<id>", format = "json", data = "<update_data>", rank = 2)]
pub async fn patch_review_by_id(
    db: Connection<BearoData>,
    id: &str,
    update_data: Json<UpdateReview>,
) -> Result<Json<Review>, status::Custom<String>> {
    let collection = db.database("bearodata").collection::<Review>("reviews");
    let oid = ObjectId::parse_str(id).expect("Failed to parse oid");
    let update_doc = match bson::to_document(&update_data.into_inner()) {
        Ok(doc) => doc,
        Err(e) => {
            return Err(status::Custom(
                Status::BadRequest,
                format!("Invalid update data: {}", e),
            ));
        }
    };

    match collection
        .find_one_and_update(doc! { "_id": oid }, doc! { "$set": update_doc }, None)
        .await
    {
        Ok(Some(updated_review)) => Ok(Json(updated_review)),
        Ok(None) => Err(status::Custom(
            Status::NotFound,
            "No review found for this chapter".into(),
        )),
        Err(e) => Err(status::Custom(
            Status::InternalServerError,
            format!("Failed to update review: {}", e),
        )),
    }
}

#[delete("/batch/<chapters>")]
pub async fn batch_delete_reviews(
    db: Connection<BearoData>,
    chapters: &str,
) -> Result<status::NoContent, status::Custom<String>> {
    let collection = db.database("bearodata").collection::<Review>("reviews");
    let chapters: Vec<i32> = chapters
        .split(',')
        .map(|chapter| {
            chapter
                .parse::<i32>()
                .expect("Failed to parse i32 from str")
        })
        .collect::<Vec<i32>>();

    let filter = doc! { "chapter": { "$in": &chapters } };

    match collection.delete_many(filter, None).await {
        Ok(delete_result) => {
            if delete_result.deleted_count > 0 {
                println!(
                    "Deleted {} reviews for chapters {:?}",
                    delete_result.deleted_count, chapters
                );
            } else {
                println!("No reviews found for chapters {:?}", chapters);
            }
        }
        Err(e) => {
            return Err(status::Custom(
                Status::InternalServerError,
                format!("failed to delete reviews: {}", e),
            ));
        }
    }

    Ok(status::NoContent)
}

#[delete("/<chapter>")]
pub async fn delete_review(
    db: Connection<BearoData>,
    chapter: i32,
) -> Result<status::NoContent, status::Custom<String>> {
    let collection = db.database("bearodata").collection::<Review>("reviews");

    match collection
        .delete_many(doc! { "chapter": chapter }, None)
        .await
    {
        Ok(delete_result) => {
            if delete_result.deleted_count > 0 {
                Ok(status::NoContent)
            } else {
                Err(status::Custom(
                    Status::NotFound,
                    "No review found for this chapter".into(),
                ))
            }
        }
        Err(e) => Err(status::Custom(
            Status::InternalServerError,
            format!("failed to delete review: {}", e),
        )),
    }
}
#[delete("/<id>", rank = 2)]
pub async fn delete_review_by_id(
    db: Connection<BearoData>,
    id: &str,
) -> Result<status::NoContent, status::Custom<String>> {
    let collection = db.database("bearodata").collection::<Review>("reviews");
    let oid = ObjectId::parse_str(id).expect("Failed to parse oid");

    match collection.delete_many(doc! { "_id": oid }, None).await {
        Ok(delete_result) => {
            if delete_result.deleted_count > 0 {
                Ok(status::NoContent)
            } else {
                Err(status::Custom(
                    Status::NotFound,
                    "No review found with this ID".into(),
                ))
            }
        }
        Err(e) => Err(status::Custom(
            Status::InternalServerError,
            format!("failed to delete review: {}", e),
        )),
    }
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_review_by_chapter,
        get_review_by_oid,
        get_reviews,
        create_review,
        delete_review,
        delete_review_by_id,
        patch_review_by_chapter,
        batch_delete_reviews,
        patch_review_by_id
    ]
}
