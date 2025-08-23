use chrono::NaiveDateTime;
use mongodb::bson::{DateTime as BsonDateTime, doc, oid::ObjectId};
use rocket::request::{FromRequest, Outcome, Request};
use rocket::{State, futures::StreamExt};
use sha2::{Digest, Sha256};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use std::{collections::HashMap, time::SystemTime};

use crate::db::BearoData;
use crate::errors::AuthError;
use crate::models::ApiKey;

#[derive(Clone, Debug)]
struct CacheEntry {
    api_key: ApiKey,
    cached_at: Instant,
}

#[derive(Default)]
pub struct ApiKeyCache {
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
}

impl ApiKeyCache {
    pub fn new() -> Self {
        Self::default()
    }

    fn get(&self, key_hash: &str) -> Option<ApiKey> {
        let cache = self.cache.read().ok()?;
        let entry = cache.get(key_hash)?;

        if entry.cached_at.elapsed() < Duration::from_secs(300) {
            Some(entry.api_key.clone())
        } else {
            None
        }
    }

    fn insert(&self, key_hash: String, api_key: ApiKey) {
        if let Ok(mut cache) = self.cache.write() {
            cache.insert(
                key_hash,
                CacheEntry {
                    api_key,
                    cached_at: Instant::now(),
                },
            );
        }
    }

    fn remove(&self, key_hash: &str) {
        if let Ok(mut cache) = self.cache.write() {
            cache.remove(key_hash);
        }
    }

    pub fn cleanup_expired(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.retain(|_, entry| entry.cached_at.elapsed() < Duration::from_secs(300));
        }
    }
}

#[derive(Debug, Clone)]
pub struct User {
    pub api_key: ApiKey,
}

impl User {
    pub fn id(&self) -> ObjectId {
        self.api_key.oid
    }

    pub fn is_admin(&self) -> bool {
        self.api_key.is_admin
    }

    pub fn created_at(&self) -> NaiveDateTime {
        self.api_key.created_at
    }

    pub fn last_used_at(&self) -> Option<NaiveDateTime> {
        self.api_key.last_used_at
    }

    pub fn require_admin(&self) -> Result<(), AuthError> {
        if self.is_admin() {
            Ok(())
        } else {
            Err(AuthError::InsufficientPermissions)
        }
    }

    pub fn as_api_key(&self) -> &ApiKey {
        &self.api_key
    }
}

pub struct AdminUser(pub User);

impl std::ops::Deref for AdminUser {
    type Target = User;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Default)]
pub struct AuthService {
    cache: ApiKeyCache,
}

impl AuthService {
    pub fn new() -> Self {
        Self {
            cache: ApiKeyCache::new(),
        }
    }

    fn hash_api_key(key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub async fn validate_api_key(&self, key: &str, db: &BearoData) -> Result<ApiKey, AuthError> {
        let key_hash = Self::hash_api_key(key);

        if let Some(cached_key) = self.cache.get(&key_hash) {
            return Ok(cached_key);
        }

        let collection = db.database("bearodata").collection::<ApiKey>("api_keys");
        let filter = doc! { "key_hash": &key_hash };

        let api_key = collection
            .find_one(filter, None)
            .await
            .map_err(|_| AuthError::Database)?
            .ok_or(AuthError::InvalidKey)?;

        self.cache.insert(key_hash, api_key.clone());

        Ok(api_key)
    }

    pub async fn update_last_used(
        &self,
        key_id: ObjectId,
        db: &BearoData,
    ) -> Result<(), AuthError> {
        let collection = db.database("bearodata").collection::<ApiKey>("api_keys");
        let filter = doc! { "_id": key_id };
        let update = doc! {
            "$set": {
                "last_used_at": BsonDateTime::from_system_time(SystemTime::now())
            }
        };

        collection
            .update_one(filter, update, None)
            .await
            .map_err(|_| AuthError::Database)?;

        Ok(())
    }

    pub fn generate_api_key() -> String {
        use uuid::Uuid;
        format!("ak_{}", Uuid::new_v4().simple())
    }

    pub async fn ensure_admin_exists(&self, db: &BearoData) -> Result<(), AuthError> {
        let collection = db.database("bearodata").collection::<ApiKey>("api_keys");

        let admin_count = collection
            .count_documents(doc! { "is_admin": true }, None)
            .await
            .map_err(|_| AuthError::Database)?;

        if admin_count == 0 {
            if let Ok(admin_key) = std::env::var("BOOTSTRAP_ADMIN_KEY") {
                println!("🔧 Creating bootstrap admin key...");
                match self.create_api_key(&admin_key, true, db).await {
                    Ok(api_key) => {
                        println!("✅ Bootstrap admin created with ID: {}", api_key.oid);
                        println!("⚠️  Remove BOOTSTRAP_ADMIN_KEY from environment after startup!");
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to create bootstrap admin: {}", e);
                        return Err(e);
                    }
                }
            } else {
                println!("⚠️  No admin users exist and no BOOTSTRAP_ADMIN_KEY provided");
                println!("   Set BOOTSTRAP_ADMIN_KEY environment variable or use CLI command");
            }
        }

        Ok(())
    }

    pub async fn create_api_key(
        &self,
        key: &str,
        is_admin: bool,
        db: &BearoData,
    ) -> Result<ApiKey, AuthError> {
        let collection = db.database("bearodata").collection::<ApiKey>("api_keys");
        let key_hash = Self::hash_api_key(key);

        let new_api_key = ApiKey {
            oid: ObjectId::new(),
            key_hash: key_hash.clone(),
            is_admin,
            created_at: chrono::Utc::now().naive_utc(),
            last_used_at: None,
        };

        collection
            .insert_one(&new_api_key, None)
            .await
            .map_err(|_| AuthError::Database)?;

        self.cache.insert(key_hash, new_api_key.clone());

        Ok(new_api_key)
    }

    pub async fn revoke_api_key(&self, key: &str, db: &BearoData) -> Result<(), AuthError> {
        let collection = db.database("bearodata").collection::<ApiKey>("api_keys");
        let key_hash = Self::hash_api_key(key);

        let filter = doc! { "key_hash": &key_hash };
        collection
            .delete_one(filter, None)
            .await
            .map_err(|_| AuthError::Database)?;

        self.cache.remove(&key_hash);

        Ok(())
    }

    pub async fn list_api_keys(&self, db: &BearoData) -> Result<Vec<ApiKey>, AuthError> {
        let collection = db.database("bearodata").collection::<ApiKey>("api_keys");
        let mut cursor = collection
            .find(doc! {}, None)
            .await
            .map_err(|_| AuthError::Database)?;

        let mut api_keys = Vec::new();

        while let Some(result) = cursor.next().await {
            match result {
                Ok(api_key) => api_keys.push(api_key),
                Err(_) => return Err(AuthError::Database),
            }
        }

        Ok(api_keys)
    }

    pub fn cleanup_cache(&self) {
        self.cache.cleanup_expired();
    }

    fn extract_bearer_token(auth_header: &str) -> Result<&str, AuthError> {
        auth_header
            .strip_prefix("Bearer ")
            .ok_or(AuthError::InvalidFormat)
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = AuthError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let auth_service = match request.guard::<&State<AuthService>>().await {
            Outcome::Success(service) => service,
            _ => {
                return Outcome::Error((
                    rocket::http::Status::InternalServerError,
                    AuthError::Database,
                ));
            }
        };

        let db = match request.guard::<&BearoData>().await {
            Outcome::Success(db) => db,
            _ => {
                return Outcome::Error((
                    rocket::http::Status::InternalServerError,
                    AuthError::Database,
                ));
            }
        };

        let auth_header = match request.headers().get_one("Authorization") {
            Some(header) => header,
            None => {
                return Outcome::Error((
                    rocket::http::Status::Unauthorized,
                    AuthError::MissingHeader,
                ));
            }
        };

        let api_key = match AuthService::extract_bearer_token(auth_header) {
            Ok(key) => key,
            Err(e) => return Outcome::Error((rocket::http::Status::Unauthorized, e)),
        };

        match auth_service.validate_api_key(api_key, db).await {
            Ok(key_record) => {
                let key_id = key_record.oid;
                let auth_service_clone = auth_service.inner().clone();

                let _ = auth_service_clone.update_last_used(key_id, db).await;

                Outcome::Success(User {
                    api_key: key_record,
                })
            }
            Err(e) => Outcome::Error((rocket::http::Status::Unauthorized, e)),
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminUser {
    type Error = AuthError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match User::from_request(request).await {
            Outcome::Success(user) => {
                if user.is_admin() {
                    Outcome::Success(AdminUser(user))
                } else {
                    Outcome::Error((
                        rocket::http::Status::Forbidden,
                        AuthError::InsufficientPermissions,
                    ))
                }
            }
            Outcome::Error((status, e)) => Outcome::Error((status, e)),
            Outcome::Forward(s) => Outcome::Forward(s),
        }
    }
}

impl Clone for AuthService {
    fn clone(&self) -> Self {
        Self {
            cache: ApiKeyCache {
                cache: Arc::clone(&self.cache.cache),
            },
        }
    }
}
