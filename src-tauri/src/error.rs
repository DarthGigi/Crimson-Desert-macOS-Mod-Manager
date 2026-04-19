use std::io;

use lz4_flex::block::DecompressError;
use rusqlite::Error as SqlError;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Database error: {0}")]
    Db(#[from] SqlError),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("LZ4 error: {0}")]
    Lz4(#[from] DecompressError),

    #[error("Invalid game install: {0}")]
    InvalidGameInstall(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid mod file: {0}")]
    InvalidMod(String),

    #[error("Patch failed: {0}")]
    Patch(String),

    #[error("Operation failed: {0}")]
    Other(String),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorPayload {
    pub message: String,
}

impl From<AppError> for ErrorPayload {
    fn from(value: AppError) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;
