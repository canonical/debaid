use anyhow::Result;

use crate::cli::CommonArgs;
use crate::error::Error;

pub(crate) fn execute(common: &CommonArgs) -> Result<()> {
    tracing::debug!(dry_run = common.dry_run, "refresh");
    Err(Error::NotImplemented("refresh").into())
}
