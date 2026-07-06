//! debaid: a self-contained Debian packaging assistant
//!
//! Split into a library plus a thin binary so tests and later modules can call
//! [`run`] and the command handlers directly

pub mod cli;
pub mod error;

mod commands;
mod logging;

use anyhow::Result;
use clap::Parser;

use crate::cli::Cli;

/// Shared entry point for the binary and integration tests
pub fn run() -> Result<()> {
    let cli = Cli::parse();
    logging::init(cli.common.verbose);
    commands::dispatch(&cli)
}
