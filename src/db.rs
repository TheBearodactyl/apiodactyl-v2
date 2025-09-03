//! # Database connection module
//!
//! This module provides the database connection pool for MongoDB using rocket_db_pools.
//!
//! ## Configuration
//!
//! The database connection is configured in Rocket.toml or via environment variables:
//! - `DATABASE_URL` or `MONGODB_URL`: MongoDB connection string
//!
//! ## Usage
//!
//! The `BearoData` struct is automatically managed by Rocket and can be injected
//! into request handlers using the `Connection<BearoData>` guard.

use rocket_db_pools::{Database, mongodb::Client};

/// MongoDB database connection pool.
///
/// This struct wraps the MongoDB client and is managed by Rocket's connection pool.
/// It provides automatic connection pooling and lifecycle management.
///
/// # Example
///
/// ```rust,no_run
/// use rocket_db_pools::Connection;
///
/// #[get("/")]
/// async fn handler(db: Connection<BearoData>) -> &'static str {
///     // Use db to interact with MongoDB
///     "Hello"
/// }
/// ```
#[derive(Database)]
#[database("bearodata")]
pub struct BearoData(Client);
