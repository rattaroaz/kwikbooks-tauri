//! Errors returned to the frontend via Tauri invoke (`Serialize` JSON payloads).

#[derive(Debug, serde::Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum DbCommandError {
    PathResolution { message: String },
    DatabaseOpen { message: String },
    Migration { version: i32, message: String },
    Sql { message: String },
    Validation { message: String },
    Invariant { message: String },
    Conflict { message: String },
    NotFound { entity: String, id: i64 },
}

impl From<rusqlite::Error> for DbCommandError {
    fn from(err: rusqlite::Error) -> Self {
        Self::Sql {
            message: err.to_string(),
        }
    }
}

impl From<std::io::Error> for DbCommandError {
    fn from(err: std::io::Error) -> Self {
        Self::DatabaseOpen {
            message: err.to_string(),
        }
    }
}

impl std::fmt::Display for DbCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PathResolution { message }
            | Self::DatabaseOpen { message }
            | Self::Sql { message }
            | Self::Validation { message }
            | Self::Invariant { message }
            | Self::Conflict { message } => write!(f, "{message}"),
            Self::Migration { version, message } => {
                write!(f, "migration {version}: {message}")
            }
            Self::NotFound { entity, id } => write!(f, "{entity} {id} not found"),
        }
    }
}

impl std::error::Error for DbCommandError {}
