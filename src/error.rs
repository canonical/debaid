//! Typed errors for the library modules; the binary boundary wraps them in
//! `anyhow` to attach context

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("required tool not available: {0}")]
    MissingTool(String),

    #[error("source tree at {0} has no debian/ directory")]
    NoDebianDir(PathBuf),

    #[error("not yet implemented: {0}")]
    NotImplemented(&'static str),
}

pub type Result<T> = std::result::Result<T, Error>;
