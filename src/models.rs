use {
    chrono::NaiveDateTime,
    mongodb::bson::{doc, oid::ObjectId},
    rocket::{
        Request,
        request::{FromRequest, Outcome},
    },
    serde::{Deserialize, Serialize},
    std::collections::HashMap,
};

pub struct Locale(pub Option<String>);

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum LocalizedString {
    Simple(String),
    Localized(HashMap<String, String>),
}

impl LocalizedString {
    pub fn get_text(&self, locale: Option<&str>) -> String {
        match self {
            LocalizedString::Simple(text) => text.clone(),
            LocalizedString::Localized(map) => {
                if let Some(locale) = locale {
                    if let Some(text) = map.get(locale) {
                        return text.clone();
                    }

                    if let Some(lang) = locale.split('-').next()
                        && let Some(text) = map.get(lang)
                    {
                        return text.clone();
                    }
                }

                map.get("en")
                    .or_else(|| map.values().next())
                    .cloned()
                    .unwrap_or_default()
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum LocalizedStringArray {
    Simple(Vec<String>),
    Localized(Vec<LocalizedString>),
}

impl LocalizedStringArray {
    pub fn get_texts(&self, locale: Option<&str>) -> Vec<String> {
        match self {
            LocalizedStringArray::Simple(texts) => texts.clone(),
            LocalizedStringArray::Localized(localized_texts) => localized_texts
                .iter()
                .map(|ls| ls.get_text(locale))
                .collect(),
        }
    }
}

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
    pub title: LocalizedString,
    pub author: LocalizedString,
    pub genres: LocalizedStringArray,
    pub tags: LocalizedStringArray,
    pub rating: i32,
    pub status: LocalizedString,
    pub description: LocalizedString,
    pub my_thoughts: LocalizedString,
    pub links: Option<HashMap<String, String>>,
    pub cover_image: String,
    pub explicit: bool,
    pub color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct NewBook {
    pub title: LocalizedString,
    pub author: LocalizedString,
    pub genres: LocalizedStringArray,
    pub tags: LocalizedStringArray,
    pub rating: i32,
    pub status: LocalizedString,
    pub description: LocalizedString,
    pub my_thoughts: LocalizedString,
    pub links: Option<HashMap<String, String>>,
    pub cover_image: String,
    pub explicit: bool,
    pub color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct UpdateBook {
    pub title: Option<LocalizedString>,
    pub author: Option<LocalizedString>,
    pub genres: Option<LocalizedStringArray>,
    pub tags: Option<LocalizedStringArray>,
    pub rating: Option<i32>,
    pub status: Option<LocalizedString>,
    pub description: Option<LocalizedString>,
    pub my_thoughts: Option<LocalizedString>,
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct LocalizedBook {
    #[serde(rename = "_id")]
    pub oid: ObjectId,
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

impl Book {
    pub fn localize(&self, locale: Option<&str>) -> LocalizedBook {
        LocalizedBook {
            oid: self.oid,
            title: self.title.get_text(locale),
            author: self.author.get_text(locale),
            genres: self.genres.get_texts(locale),
            tags: self.tags.get_texts(locale),
            rating: self.rating,
            status: self.status.get_text(locale),
            description: self.description.get_text(locale),
            my_thoughts: self.my_thoughts.get_text(locale),
            links: self.links.clone(),
            cover_image: self.cover_image.clone(),
            explicit: self.explicit,
            color: self.color.clone(),
        }
    }
}

impl NewBook {
    pub fn to_book_with_id(&self, oid: ObjectId) -> Book {
        Book {
            oid,
            title: self.title.clone(),
            author: self.author.clone(),
            genres: self.genres.clone(),
            tags: self.tags.clone(),
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

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Locale {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let locale = request
            .headers()
            .get_one("Accept-Language")
            .and_then(|header| {
                // Parse Accept-Language header and get the first locale
                header.split(',').next().map(|s| s.trim().to_string())
            });
        Outcome::Success(Locale(locale))
    }
}
