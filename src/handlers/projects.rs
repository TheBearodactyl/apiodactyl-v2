use crate::auth::User;
use crate::db::BearoData;
use crate::models::{NewProject, Project, UpdateProject};
use mongodb::bson::{Document, doc, oid::ObjectId};
use rocket::futures::TryStreamExt;
use rocket::serde::json::Json;
use rocket::{delete, get, http::Status, patch, post, put, routes};
use rocket_db_pools::mongodb::options::UpdateOptions;
use rocket_db_pools::{Connection, mongodb::Collection};

#[post("/", format = "json", data = "<new_project>")]
pub async fn create_project(
    db: Connection<BearoData>,
    user: User,
    new_project: Json<NewProject>,
) -> Result<Json<Project>, Status> {
    user.require_admin().map_err(|_| Status::Forbidden)?;

    let collection: Collection<NewProject> = db.database("bearodata").collection("projects");

    let result = collection
        .insert_one(new_project.into_inner(), None)
        .await
        .map_err(|_| Status::InternalServerError)?;

    let project_collection = db.database("bearodata").collection::<Project>("projects");
    let inserted_project = project_collection
        .find_one(doc! { "_id": result.inserted_id }, None)
        .await
        .map_err(|_| Status::InternalServerError)?
        .ok_or(Status::InternalServerError)?;

    Ok(Json(inserted_project))
}

#[get("/<project_id>")]
pub async fn get_project(
    db: Connection<BearoData>,
    project_id: String,
) -> Result<Json<Project>, Status> {
    let collection = db.database("bearodata").collection::<Project>("projects");

    let oid = ObjectId::parse_str(&project_id).map_err(|_| Status::BadRequest)?;

    let project = collection
        .find_one(doc! { "_id": oid }, None)
        .await
        .map_err(|_| Status::InternalServerError)?
        .ok_or(Status::NotFound)?;

    Ok(Json(project))
}

#[get("/")]
pub async fn get_projects(db: Connection<BearoData>) -> Result<Json<Vec<Project>>, Status> {
    let collection = db.database("bearodata").collection::<Project>("projects");

    let mut cursor = collection
        .find(doc! {}, None)
        .await
        .map_err(|_| Status::InternalServerError)?;

    let mut projects = Vec::new();
    while let Some(project) = cursor
        .try_next()
        .await
        .map_err(|_| Status::InternalServerError)?
    {
        projects.push(project);
    }

    Ok(Json(projects))
}

#[put("/<project_id>", format = "json", data = "<update_data>")]
pub async fn update_project(
    db: Connection<BearoData>,
    user: User,
    project_id: String,
    update_data: Json<UpdateProject>,
) -> Result<Json<Project>, Status> {
    user.require_admin().map_err(|_| Status::Forbidden)?;

    let collection = db.database("bearodata").collection::<Project>("projects");
    let oid = ObjectId::parse_str(&project_id).map_err(|_| Status::BadRequest)?;

    let update_doc = mongodb::bson::to_document(&update_data.into_inner())
        .map_err(|_| Status::InternalServerError)?;

    let options = UpdateOptions::builder().upsert(false).build();

    collection
        .update_one(doc! { "_id": oid }, doc! { "$set": update_doc }, options)
        .await
        .map_err(|_| Status::InternalServerError)?;

    let updated_project = collection
        .find_one(doc! { "_id": oid }, None)
        .await
        .map_err(|_| Status::InternalServerError)?
        .ok_or(Status::NotFound)?;

    Ok(Json(updated_project))
}

#[delete("/<project_id>")]
pub async fn delete_project(
    db: Connection<BearoData>,
    user: User,
    project_id: String,
) -> Result<Json<Project>, Status> {
    user.require_admin().map_err(|_| Status::Forbidden)?;

    let collection = db.database("bearodata").collection::<Project>("projects");
    let oid = ObjectId::parse_str(&project_id).map_err(|_| Status::BadRequest)?;

    let project = collection
        .find_one(doc! { "_id": oid }, None)
        .await
        .map_err(|_| Status::InternalServerError)?
        .ok_or(Status::NotFound)?;

    let result = collection
        .delete_one(doc! { "_id": oid }, None)
        .await
        .map_err(|_| Status::InternalServerError)?;

    if result.deleted_count > 0 {
        Ok(Json(project))
    } else {
        Err(Status::NotFound)
    }
}

#[patch("/<project_id>", format = "json", data = "<update_data>")]
pub async fn patch_project(
    db: Connection<BearoData>,
    user: User,
    project_id: String,
    update_data: Json<UpdateProject>,
) -> Result<Json<Project>, Status> {
    user.require_admin().map_err(|_| Status::Forbidden)?;

    let collection = db.database("bearodata").collection::<Project>("projects");
    let oid = ObjectId::parse_str(&project_id).map_err(|_| Status::BadRequest)?;

    let mut update_doc = Document::new();
    let patch = update_data.into_inner();

    if let Some(name) = patch.name {
        update_doc.insert("name", name);
    }
    if let Some(description) = patch.description {
        update_doc.insert("description", description);
    }
    if let Some(tags) = patch.tags {
        update_doc.insert("tags", tags);
    }
    if let Some(source) = patch.source {
        update_doc.insert("source", source);
    }
    if let Some(cover_image) = patch.cover_image {
        update_doc.insert("cover_image", cover_image);
    }
    if let Some(install_command) = patch.install_command {
        update_doc.insert("install_command", install_command);
    }

    collection
        .update_one(doc! { "_id": oid }, doc! { "$set": update_doc }, None)
        .await
        .map_err(|_| Status::InternalServerError)?;

    let updated_project = collection
        .find_one(doc! { "_id": oid }, None)
        .await
        .map_err(|_| Status::InternalServerError)?
        .ok_or(Status::NotFound)?;

    Ok(Json(updated_project))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        create_project,
        get_project,
        get_projects,
        update_project,
        delete_project,
        patch_project,
    ]
}
