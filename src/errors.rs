use rocket::http::Status;
use rocket::response::{self, Responder, Response};
use rocket::Request;
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Missing Authorization header")]
    MissingHeader,
    #[error("Invalid Authorization header format")]
    InvalidFormat,
    #[error("Invalid API key")]
    InvalidKey,
    #[error("Insufficient permissions")]
    InsufficientPermissions,
    #[error("Database error")]
    Database,
}

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
