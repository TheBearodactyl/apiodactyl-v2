//! # Data models for the Apiodactyl API
//!
//! This module contains all the data structures used throughout the application,
//! including database models, request/response DTOs, and localization support.

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

/// Wrapper for the locale extracted from the request headers.
///
/// This struct is used to extract the Accept-Language header from incoming requests
/// and make it available for localization purposes.
pub struct Locale(pub Option<String>);

/// A string that can be either simple or localized to multiple languages.
///
/// # Examples
///
/// Simple string:
/// ```json
/// "Hello World"
/// ```
///
/// Localized string:
/// ```json
/// {
///   "en": "Hello World",
///   "es": "Hola Mundo",
///   "fr": "Bonjour le monde"
/// }
/// ```
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum LocalizedString {
    /// A simple non-localized string
    Simple(String),
    /// A map of locale codes to localized strings
    Localized(HashMap<String, String>),
}

impl LocalizedString {
    /// Retrieves the text in the specified locale, falling back to English or any available language.
    ///
    /// # Arguments
    ///
    /// * `locale` - The preferred locale (e.g., "en-US", "jp", "ar")
    ///
    /// # Returns
    ///
    /// The localized text or a fallback value if the locale is not available.
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

/// An array of strings that can be either simple or localized.
///
/// Similar to LocalizedString but for arrays of strings.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum LocalizedStringArray {
    /// A simple array of non-localized strings
    Simple(Vec<String>),
    /// An array of localized strings
    Localized(Vec<LocalizedString>),
}

impl LocalizedStringArray {
    /// Retrieves all texts in the specified locale.
    ///
    /// # Arguments
    ///
    /// * `locale` - The preferred locale
    ///
    /// # Returns
    ///
    /// A vector of localized strings.
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

/// Represents a review in the database.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct Review {
    /// MongoDB ObjectId
    #[serde(rename = "_id")]
    pub oid: ObjectId,
    /// Chapter number being reviewed
    pub chapter: i32,
    /// Description of the review
    pub description: String,
    /// Rating (1-5)
    pub rating: i32,
    /// Personal thoughts about the chapter
    pub thoughts: String,
}

/// Data transfer object for creating a new review.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct NewReview {
    /// Chapter number being reviewed
    pub chapter: i32,
    /// Description of the review
    pub description: String,
    /// Rating (1-5)
    pub rating: i32,
    /// Personal thoughts about the chapter
    pub thoughts: String,
}

/// Data transfer object for updating an existing review.
/// All fields are optional to support partial updates.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct UpdateReview {
    /// Updated chapter number
    pub chapter: Option<i32>,
    /// Updated description
    pub description: Option<String>,
    /// Updated rating
    pub rating: Option<i32>,
    /// Updated thoughts
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

/// Represents a book in the database with full localization support.
#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Book {
    /// MongoDB ObjectId
    #[serde(rename = "_id")]
    pub oid: ObjectId,
    /// Book title (localized)
    pub title: LocalizedString,
    /// Book author (localized)
    pub author: LocalizedString,
    /// List of genres (localized)
    pub genres: LocalizedStringArray,
    /// List of tags (localized)
    pub tags: LocalizedStringArray,
    /// Book rating (1-5)
    pub rating: i32,
    /// Reading status (e.g., "reading", "completed", "planned") (localized)
    pub status: LocalizedString,
    /// Book description (localized)
    pub description: LocalizedString,
    /// Personal thoughts about the book (localized)
    pub my_thoughts: LocalizedString,
    /// External links (e.g., purchase links, reviews)
    pub links: Option<HashMap<String, String>>,
    /// URL to the book cover image
    pub cover_image: String,
    /// Whether the book contains explicit content
    pub explicit: bool,
    /// Optional color theme for UI display
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
    /// Converts a Book with localized fields into a LocalizedBook with resolved strings.
    ///
    /// # Arguments
    ///
    /// * `locale` - The preferred locale for text resolution
    ///
    /// # Returns
    ///
    /// A LocalizedBook with all text fields resolved to the specified locale.
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
    /// Converts a NewBook DTO into a Book entity with the given ObjectId.
    ///
    /// # Arguments
    ///
    /// * `oid` - The MongoDB ObjectId to assign to the book
    ///
    /// # Returns
    ///
    /// A Book entity ready for database insertion.
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

/// Extracts the Accept-Language header from incoming requests and parses
/// the first locale preference.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_localized_string_simple() {
        let simple = LocalizedString::Simple("Hello".to_string());
        assert_eq!(simple.get_text(None), "Hello");
        assert_eq!(simple.get_text(Some("en")), "Hello");
        assert_eq!(simple.get_text(Some("es")), "Hello");
    }

    #[test]
    fn test_localized_string_localized() {
        let mut map = HashMap::new();
        map.insert("en".to_string(), "Hello".to_string());
        map.insert("es".to_string(), "Hola".to_string());
        map.insert("fr".to_string(), "Bonjour".to_string());

        let localized = LocalizedString::Localized(map);

        assert_eq!(localized.get_text(Some("en")), "Hello");
        assert_eq!(localized.get_text(Some("es")), "Hola");
        assert_eq!(localized.get_text(Some("fr")), "Bonjour");
        assert_eq!(localized.get_text(Some("de")), "Hello");
        assert_eq!(localized.get_text(None), "Hello");
    }

    #[test]
    fn test_localized_string_locale_fallback() {
        let mut map = HashMap::new();
        map.insert("en".to_string(), "Hello".to_string());
        map.insert("es".to_string(), "Hola".to_string());

        let localized = LocalizedString::Localized(map);

        assert_eq!(localized.get_text(Some("en-US")), "Hello");
        assert_eq!(localized.get_text(Some("es-MX")), "Hola");
        assert_eq!(localized.get_text(Some("fr-FR")), "Hello");
    }

    #[test]
    fn test_localized_string_array_simple() {
        let simple =
            LocalizedStringArray::Simple(vec!["Action".to_string(), "Adventure".to_string()]);

        let texts = simple.get_texts(None);
        assert_eq!(texts.len(), 2);
        assert_eq!(texts[0], "Action");
        assert_eq!(texts[1], "Adventure");
    }

    #[test]
    fn test_localized_string_array_localized() {
        let mut map1 = HashMap::new();
        map1.insert("en".to_string(), "Action".to_string());
        map1.insert("es".to_string(), "Acción".to_string());

        let mut map2 = HashMap::new();
        map2.insert("en".to_string(), "Adventure".to_string());
        map2.insert("es".to_string(), "Aventura".to_string());

        let localized = LocalizedStringArray::Localized(vec![
            LocalizedString::Localized(map1),
            LocalizedString::Localized(map2),
        ]);

        let texts_en = localized.get_texts(Some("en"));
        assert_eq!(texts_en.len(), 2);
        assert_eq!(texts_en[0], "Action");
        assert_eq!(texts_en[1], "Adventure");

        let texts_es = localized.get_texts(Some("es"));
        assert_eq!(texts_es.len(), 2);
        assert_eq!(texts_es[0], "Acción");
        assert_eq!(texts_es[1], "Aventura");
    }

    #[test]
    fn test_book_localization() {
        let mut title_map = HashMap::new();
        title_map.insert("en".to_string(), "The Great Book".to_string());
        title_map.insert("es".to_string(), "El Gran Libro".to_string());

        let mut author_map = HashMap::new();
        author_map.insert("en".to_string(), "John Doe".to_string());
        author_map.insert("es".to_string(), "Juan Pérez".to_string());

        let book = Book {
            oid: ObjectId::new(),
            title: LocalizedString::Localized(title_map),
            author: LocalizedString::Localized(author_map),
            genres: LocalizedStringArray::Simple(vec!["Fiction".to_string()]),
            tags: LocalizedStringArray::Simple(vec!["Classic".to_string()]),
            rating: 5,
            status: LocalizedString::Simple("Completed".to_string()),
            description: LocalizedString::Simple("A great book".to_string()),
            my_thoughts: LocalizedString::Simple("Loved it".to_string()),
            links: None,
            cover_image: "https://example.com/cover.jpg".to_string(),
            explicit: false,
            color: Some("#FF0000".to_string()),
        };

        let localized_en = book.localize(Some("en"));
        assert_eq!(localized_en.title, "The Great Book");
        assert_eq!(localized_en.author, "John Doe");

        let localized_es = book.localize(Some("es"));
        assert_eq!(localized_es.title, "El Gran Libro");
        assert_eq!(localized_es.author, "Juan Pérez");
    }

    #[test]
    fn test_new_book_to_book_with_id() {
        let new_book = NewBook {
            title: LocalizedString::Simple("Test Book".to_string()),
            author: LocalizedString::Simple("Test Author".to_string()),
            genres: LocalizedStringArray::Simple(vec!["Test Genre".to_string()]),
            tags: LocalizedStringArray::Simple(vec!["Test Tag".to_string()]),
            rating: 4,
            status: LocalizedString::Simple("Reading".to_string()),
            description: LocalizedString::Simple("Test Description".to_string()),
            my_thoughts: LocalizedString::Simple("Test Thoughts".to_string()),
            links: Some(HashMap::new()),
            cover_image: "test.jpg".to_string(),
            explicit: false,
            color: None,
        };

        let oid = ObjectId::new();
        let book = new_book.to_book_with_id(oid);

        assert_eq!(book.oid, oid);
        assert_eq!(book.rating, 4);
        assert_eq!(book.cover_image, "test.jpg");
        assert_eq!(book.explicit, false);
    }
}
