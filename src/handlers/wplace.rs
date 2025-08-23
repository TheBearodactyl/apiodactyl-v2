use {
    crate::{
        auth::User,
        db::BearoData,
        models::{NewWplaceScreenshot, WplaceScreenshot},
    },
    mongodb::bson::{doc, oid::ObjectId},
    rocket::{
        delete, futures::StreamExt, get, http::Status, post, response::status, serde::json::Json,
    },
    rocket_db_pools::Connection,
};

#[post("/", data = "<screenshot>", format = "json")]
pub async fn create_screenshot(
    user: User,
    db: Connection<BearoData>,
    screenshot: Json<NewWplaceScreenshot>,
) -> Result<Json<WplaceScreenshot>, status::Custom<String>> {
    user.require_admin().expect("User is not an admin");
    let collection = db
        .database("bearodata")
        .collection::<WplaceScreenshot>("wplace_screenshots");

    let new_screenshot = WplaceScreenshot {
        oid: ObjectId::new(),
        cover_image: screenshot.cover_image.clone(),
        alt: screenshot.alt.clone(),
    };

    match collection.insert_one(new_screenshot.clone(), None).await {
        Ok(_) => {
            println!(
                "Created wplace screenshot with data {:?}",
                new_screenshot.clone()
            );
            Ok(Json(new_screenshot))
        }
        Err(e) => Err(status::Custom(
            Status::Conflict,
            format!(
                "screenshot with data {:?} already found: {}",
                new_screenshot, e
            ),
        )),
    }
}

#[get("/<id>")]
pub async fn get_screenshot_by_id(
    db: Connection<BearoData>,
    id: &str,
) -> Option<Json<WplaceScreenshot>> {
    let collection = db
        .database("bearodata")
        .collection::<WplaceScreenshot>("wplace_screenshots");
    let oid = ObjectId::parse_str(id).expect("Failed to parse oid");
    let found_screenshot = collection
        .find_one(doc! { "_id": oid }, None)
        .await
        .unwrap();

    Some(Json(found_screenshot.unwrap()))
}

#[get("/")]
pub async fn get_screenshots(db: Connection<BearoData>) -> Option<Json<Vec<WplaceScreenshot>>> {
    let collection = db
        .database("bearodata")
        .collection::<WplaceScreenshot>("wplace_screenshots");

    let mut cursor = match collection.find(doc! {}, None).await {
        Ok(cursor) => cursor,
        Err(_) => return None,
    };

    let mut screenshots = Vec::new();
    while let Some(result) = cursor.next().await {
        match result {
            Ok(screenshot) => screenshots.push(screenshot),
            Err(_) => return None,
        }
    }

    Some(Json(screenshots))
}

#[delete("/<id>")]
pub async fn delete_screenshot(
    db: Connection<BearoData>,
    id: &str,
) -> Result<status::NoContent, status::Custom<String>> {
    let collection = db
        .database("bearodata")
        .collection::<WplaceScreenshot>("wplace_screenshots");

    if let Ok(ss_id) = ObjectId::parse_str(id) {
        match collection.delete_many(doc! { "_id": ss_id }, None).await {
            Ok(delete_result) => {
                if delete_result.deleted_count > 0 {
                    Ok(status::NoContent)
                } else {
                    Err(status::Custom(
                        Status::NotFound,
                        format!("No screenshot found with the id {}", id),
                    ))
                }
            }
            Err(e) => Err(status::Custom(
                Status::InternalServerError,
                format!("failed to delete screenshot: {}", e),
            )),
        }
    } else {
        Err(status::Custom(
            Status::NotFound,
            format!("failed to find screenshot with id {}", id),
        ))
    }
}
