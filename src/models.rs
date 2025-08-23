use {
    chrono::NaiveDateTime,
    mongodb::bson::{doc, oid::ObjectId},
    serde::{Deserialize, Serialize}, std::collections::HashMap,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct Review {
    #[serde(rename = "_id")]
    pub oid: ObjectId,
    pub chapter: i32,
    pub description: String,
    pub rating: i32,
    pub thoughts: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct NewReview {
    pub chapter: i32,
    pub description: String,
    pub rating: i32,
    pub thoughts: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct UpdateReview {
    pub chapter: Option<i32>,
    pub description: Option<String>,
    pub rating: Option<i32>,
    pub thoughts: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct WplaceScreenshot {
    #[serde(rename = "_id")]
    pub oid: ObjectId,
    pub alt: String,
    pub cover_image: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct NewWplaceScreenshot {
    pub alt: String,
    pub cover_image: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct UpdateWplaceScreenshot {
    alt: Option<String>,
    cover_image: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Project {
    #[serde(rename = "_id")]
    pub oid: ObjectId,
    pub name: String,
    pub description: String,
    pub tags: Option<Vec<Option<String>>>,
    pub source: String,
    pub cover_image: Option<String>,
    pub install_command: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct NewProject {
    pub name: String,
    pub description: String,
    pub tags: Option<Vec<Option<String>>>,
    pub source: String,
    pub cover_image: Option<String>,
    pub install_command: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct UpdateProject {
    pub name: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<Option<String>>>,
    pub source: Option<String>,
    pub cover_image: Option<String>,
    pub install_command: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Book {
    #[serde(rename = "_id")]
    pub oid: ObjectId,
    pub title: String,
    pub author: String,
    pub genres: Vec<Option<String>>,
    pub tags: Vec<Option<String>>,
    pub rating: i32,
    pub status: String,
    pub description: String,
    pub my_thoughts: String,
    pub links: Option<HashMap<String, String>>,
    pub cover_image: String,
    pub explicit: bool,
    pub color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct NewBook {
    pub title: String,
    pub author: String,
    pub genres: Vec<String>,
    pub tags: Vec<String>,
    pub rating: i32,
    pub status: String,
    pub description: String,
    pub my_thoughts: String,
    pub links: Option<HashMap<String, String>>,
    pub cover_image: String,
    pub explicit: bool,
    pub color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct UpdateBook {
    pub title: Option<String>,
    pub author: Option<String>,
    pub genres: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub rating: Option<i32>,
    pub status: Option<String>,
    pub description: Option<String>,
    pub my_thoughts: Option<String>,
    pub links: Option<HashMap<String, String>>,
    pub cover_image: Option<String>,
    pub explicit: Option<bool>,
    pub color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Game {
    #[serde(rename = "_id")]
    pub oid: ObjectId,
    pub title: String,
    pub developer: String,
    pub genres: Vec<Option<String>>,
    pub tags: Vec<Option<String>>,
    pub rating: i32,
    pub status: String,
    pub description: String,
    pub my_thoughts: String,
    pub links: Option<HashMap<String, String>>,
    pub cover_image: String,
    pub explicit: bool,
    pub percent: i32,
    pub bad: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct NewGame {
    pub title: String,
    pub developer: String,
    pub genres: Vec<String>,
    pub tags: Vec<String>,
    pub rating: i32,
    pub status: String,
    pub description: String,
    pub my_thoughts: String,
    pub links: Option<HashMap<String, String>>,
    pub cover_image: String,
    pub explicit: bool,
    pub percent: i32,
    pub bad: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct UpdateGame {
    pub title: Option<String>,
    pub developer: Option<String>,
    pub genres: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub rating: Option<i32>,
    pub status: Option<String>,
    pub description: Option<String>,
    pub my_thoughts: Option<String>,
    pub links: Option<HashMap<String, String>>,
    pub cover_image: Option<String>,
    pub explicit: Option<bool>,
    pub percent: Option<i32>,
    pub bad: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct ApiKey {
    #[serde(rename = "_id")]
    pub oid: ObjectId,
    pub key_hash: String,
    pub is_admin: bool,
    pub created_at: NaiveDateTime,
    pub last_used_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct NewApiKey {
    pub key_hash: String,
    pub is_admin: bool,
}

impl NewBook {
    pub fn to_book_with_id(&self, oid: ObjectId) -> Book {
        Book {
            oid,
            title: self.title.clone(),
            author: self.author.clone(),
            genres: self
                .genres
                .iter()
                .map(|g| Some(g.to_owned()))
                .collect::<Vec<Option<String>>>(),
            tags: self
                .tags
                .iter()
                .map(|t| Some(t.to_owned()))
                .collect::<Vec<Option<String>>>(),
            rating: self.rating,
            status: self.status.clone(),
            description: self.description.clone(),
            my_thoughts: self.my_thoughts.clone(),
            links: self.links.clone(),
            cover_image: self.cover_image.clone(),
            explicit: self.explicit,
            color: self.color.clone(),
        }
    }
}
