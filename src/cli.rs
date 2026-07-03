use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "debaid", version, about = "Debian packaging assistant")]
pub struct Cli {
    /// The phase (or utility) to run
    #[command(subcommand)]
    pub command: Command,

    /// Flags shared by every subcommand
    #[command(flatten)]
    pub common: CommonArgs,
}

/// Flags accepted before or after the subcommand (clap `global`)
#[derive(Debug, Args)]
pub struct CommonArgs {
    /// Show what would change without writing to the source tree
    #[arg(long, global = true)]
    pub dry_run: bool,

    /// Path to a `house-style.md` that overrides the built-in one
    #[arg(long, value_name = "PATH", global = true)]
    pub house_style: Option<PathBuf>,

    /// Reference-corpus path, or the literal `none` to disable it
    #[arg(long, value_name = "PATH", global = true)]
    pub reference: Option<String>,

    /// Skip per-phase confirmation gates (workshop mode)
    #[arg(long, short = 'y', global = true)]
    pub yes: bool,

    /// Increase log verbosity (`-v` = debug, `-vv` = trace)
    #[arg(long, short = 'v', global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Full pipeline: detect, then bootstrap-or-refresh, lintian, autopkgtest
    Run(RunArgs),

    /// Create a debian/ from scratch for an unpackaged source tree
    Bootstrap,

    /// Modernise an existing debian/ (dry-run by default)
    Refresh,

    /// Drive a package toward lintian-clean, or justified overrides only
    Lintian,

    /// Add or improve autopkgtest coverage under debian/tests/
    Autopkgtest,

    /// Probe the source tree and tooling, then write the context JSON
    Detect(DetectArgs),

    /// Build + lintian snapshot (no LLM); prints the verify JSON to stdout
    Verify(VerifyArgs),
}

#[derive(Debug, Args)]
pub struct RunArgs {
    /// Restrict the pipeline to these phases (comma-separated)
    #[arg(long, value_delimiter = ',', value_name = "PHASE")]
    pub only: Vec<Phase>,

    /// Skip these phases (comma-separated)
    #[arg(long, value_delimiter = ',', value_name = "PHASE")]
    pub skip: Vec<Phase>,
}

/// A selectable pipeline phase for `--only` / `--skip`
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Phase {
    Bootstrap,
    Refresh,
    Lintian,
    Autopkgtest,
}

#[derive(Debug, Args)]
pub struct DetectArgs {
    /// Source tree to inspect (defaults to the current directory)
    #[arg(value_name = "PATH", default_value = ".")]
    pub path: PathBuf,
}

#[derive(Debug, Args)]
pub struct VerifyArgs {
    /// Skip the build step; run lintian only where possible
    #[arg(long)]
    pub no_build: bool,
}
