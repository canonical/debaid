mod autopkgtest;
mod bootstrap;
mod detect;
mod lintian;
mod refresh;
mod run;
mod verify;

use anyhow::Result;

use crate::cli::{Cli, Command};

/// Route a parsed [`Cli`] to the handler for its subcommand.
pub(crate) fn dispatch(cli: &Cli) -> Result<()> {
    match &cli.command {
        Command::Run(args) => run::execute(&cli.common, args),
        Command::Bootstrap => bootstrap::execute(&cli.common),
        Command::Refresh => refresh::execute(&cli.common),
        Command::Lintian => lintian::execute(&cli.common),
        Command::Autopkgtest => autopkgtest::execute(&cli.common),
        Command::Detect(args) => detect::execute(&cli.common, args),
        Command::Verify(args) => verify::execute(&cli.common, args),
    }
}
