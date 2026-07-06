//! Typed errors for the library modules; the binary boundary wraps them in
//! `anyhow` to attach context

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An underlying I/O failure
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// A required external tool (lintian, sbuild, …) was not found on `PATH`
    #[error("required tool not available: {0}")]
    MissingTool(String),

    /// The source tree does not contain a `debian/` directory
    #[error("source tree at {0} has no debian/ directory")]
    NoDebianDir(PathBuf),

    /// Functionality that is scaffolded but not yet implemented
    #[error("not yet implemented: {0}")]
    NotImplemented(&'static str),
}

pub type Result<T> = std::result::Result<T, Error>;
