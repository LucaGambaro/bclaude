use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum BcError {
    #[error("I/O error on {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid config {path}: {source}")]
    ConfigParse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("cannot serialize config: {0}")]
    ConfigSerialize(#[from] toml::ser::Error),
    #[error("invalid plugin registry {path}: {source}")]
    InstalledPlugins {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("invalid MCP server JSON {path}: {source}")]
    McpJson {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("cannot serialize session: {0}")]
    SessionSerialize(#[from] serde_json::Error),
    #[error("HOME is not set (and BC_HOME is not provided)")]
    HomeNotSet,
    #[error("{0}")]
    Invalid(String),
    #[error("cannot launch `{bin}`: {source}")]
    Launch {
        bin: String,
        #[source]
        source: std::io::Error,
    },
}

pub type Result<T> = std::result::Result<T, BcError>;

impl BcError {
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        BcError::Io {
            path: path.into(),
            source,
        }
    }
}
