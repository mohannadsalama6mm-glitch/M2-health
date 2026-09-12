use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ErrorCode {
    Database,
    Migration,
    Validation,
    NotFound,
    Conflict,
    Io,
    Internal,
}

/// IPC-safe error; low-level details belong in local diagnostics, not UI responses.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: ErrorCode,
    pub message: String,
}
impl AppError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
    pub fn migration(error: impl fmt::Display) -> Self {
        eprintln!("[database migration] {error}");
        Self::new(ErrorCode::Migration, "The local database could not be upgraded. Existing data was retained. Please contact support.")
    }
}
impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl std::error::Error for AppError {}
impl From<rusqlite::Error> for AppError {
    fn from(error: rusqlite::Error) -> Self {
        eprintln!("[local database] {error}");
        match &error {
            rusqlite::Error::SqliteFailure(e, _)
                if e.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE =>
            {
                Self::new(
                    ErrorCode::Conflict,
                    "A record with this unique value already exists, or a primary/default record is already assigned.",
                )
            }
            rusqlite::Error::SqliteFailure(e, _) if e.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_FOREIGNKEY => Self::new(ErrorCode::Validation,"A referenced record does not exist or is still in use."),
            rusqlite::Error::SqliteFailure(e, _) if e.code == rusqlite::ErrorCode::ConstraintViolation => Self::new(ErrorCode::Validation,"The operation violates a catalog data constraint."),
            _ => Self::new(
                ErrorCode::Database,
                "The local database operation failed. Please retry or contact support.",
            ),
        }
    }
}
impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        eprintln!("[local storage] {error}");
        Self::new(
            ErrorCode::Io,
            "The application data directory could not be accessed. Check local access permissions.",
        )
    }
}
