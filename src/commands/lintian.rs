use anyhow::Result;

use crate::cli::CommonArgs;
use crate::error::Error;

pub(crate) fn execute(common: &CommonArgs) -> Result<()> {
    tracing::debug!(dry_run = common.dry_run, "lintian");
    Err(Error::NotImplemented("lintian").into())
}
