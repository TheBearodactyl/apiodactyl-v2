//! # Error handling module
//!
//! This module defines custom error types for the application, particularly
//! authentication errors, and their HTTP response representations.

use rocket::Request;
use rocket::http::Status;
use rocket::response::{self, Responder, Response};
use serde_json::json;
use thiserror::Error;

/// Authentication-related errors.
///
/// These errors are returned when authentication or authorization fails,
/// and are automatically converted to appropriate HTTP responses.
#[derive(Error, Debug)]
pub enum AuthError {
    /// The Authorization header is missing from the request
    #[error("Missing Authorization header")]
    MissingHeader,
    /// The Authorization header format is invalid (not "Bearer <token>")
    #[error("Invalid Authorization header format")]
    InvalidFormat,
    /// The provided API key is invalid or doesn't exist
    #[error("Invalid API key")]
    InvalidKey,
    /// The user lacks the required permissions for the operation
    #[error("Insufficient permissions")]
    InsufficientPermissions,
    /// A database operation failed
    #[error("Database error")]
    Database,
}

/// Converts AuthError to HTTP responses with appropriate status codes and JSON bodies.
///
/// # Response Format
///
/// ```json
/// {
///   "error": "Error message",
///   "status": 401
/// }
/// ```
impl<'r> Responder<'r, 'static> for AuthError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let (status, message) = match self {
            AuthError::MissingHeader => (Status::Unauthorized, "Missing Authorization header"),
            AuthError::InvalidFormat => {
                (Status::Unauthorized, "Invalid Authorization header format")
            }
            AuthError::InvalidKey => (Status::Unauthorized, "Invalid API key"),
            AuthError::InsufficientPermissions => (Status::Forbidden, "Insufficient permissions"),
            AuthError::Database => (Status::InternalServerError, "Internal server error"),
        };

        let body = json!({
            "error": message,
            "status": status.code
        })
        .to_string();

        Response::build()
            .status(status)
            .header(rocket::http::ContentType::JSON)
            .sized_body(body.len(), std::io::Cursor::new(body))
            .ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::local::blocking::Client;
    use rocket::{get, routes};

    #[get("/missing-header")]
    fn missing_header_route() -> Result<&'static str, AuthError> {
        Err(AuthError::MissingHeader)
    }

    #[get("/invalid-format")]
    fn invalid_format_route() -> Result<&'static str, AuthError> {
        Err(AuthError::InvalidFormat)
    }

    #[get("/invalid-key")]
    fn invalid_key_route() -> Result<&'static str, AuthError> {
        Err(AuthError::InvalidKey)
    }

    #[get("/insufficient-perms")]
    fn insufficient_perms_route() -> Result<&'static str, AuthError> {
        Err(AuthError::InsufficientPermissions)
    }

    #[get("/database-error")]
    fn database_error_route() -> Result<&'static str, AuthError> {
        Err(AuthError::Database)
    }

    #[test]
    fn test_auth_error_status_codes() {
        let rocket = rocket::build().mount(
            "/",
            routes![
                missing_header_route,
                invalid_format_route,
                invalid_key_route,
                insufficient_perms_route,
                database_error_route
            ],
        );
        let client = Client::tracked(rocket).expect("valid rocket");

        let response = client.get("/missing-header").dispatch();
        assert_eq!(response.status(), Status::Unauthorized);

        let response = client.get("/invalid-format").dispatch();
        assert_eq!(response.status(), Status::Unauthorized);

        let response = client.get("/invalid-key").dispatch();
        assert_eq!(response.status(), Status::Unauthorized);

        let response = client.get("/insufficient-perms").dispatch();
        assert_eq!(response.status(), Status::Forbidden);

        let response = client.get("/database-error").dispatch();
        assert_eq!(response.status(), Status::InternalServerError);
    }

    #[test]
    fn test_auth_error_messages() {
        assert_eq!(
            AuthError::MissingHeader.to_string(),
            "Missing Authorization header"
        );
        assert_eq!(
            AuthError::InvalidFormat.to_string(),
            "Invalid Authorization header format"
        );
        assert_eq!(AuthError::InvalidKey.to_string(), "Invalid API key");
        assert_eq!(
            AuthError::InsufficientPermissions.to_string(),
            "Insufficient permissions"
        );
        assert_eq!(AuthError::Database.to_string(), "Database error");
    }

    #[test]
    fn test_auth_error_json_response() {
        let rocket = rocket::build().mount("/", routes![missing_header_route]);
        let client = Client::tracked(rocket).expect("valid rocket");

        let response = client.get("/missing-header").dispatch();
        let body = response.into_string().unwrap();
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();

        assert_eq!(json["error"], "Missing Authorization header");
        assert_eq!(json["status"], 401);
    }
}
