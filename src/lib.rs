//! debaid: a self-contained Debian packaging assistant
//!
//! Split into a library plus a thin binary so tests and later modules can
//! call [`run`] and the command handlers directly

pub mod cli;
pub mod error;
pub mod model;
pub mod tool;

mod commands;
mod logging;

use anyhow::Result;
use clap::Parser;

use crate::cli::Cli;

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    logging::init(cli.common.verbose);
    commands::dispatch(&cli)
}
