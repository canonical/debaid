use anyhow::Result;

use crate::cli::{CommonArgs, RunArgs};
use crate::error::Error;

pub(crate) fn execute(common: &CommonArgs, args: &RunArgs) -> Result<()> {
    tracing::debug!(
        dry_run = common.dry_run,
        only = ?args.only,
        skip = ?args.skip,
        "run",
    );
    Err(Error::NotImplemented("run").into())
}
