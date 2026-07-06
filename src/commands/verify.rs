use anyhow::Result;

use crate::cli::{CommonArgs, VerifyArgs};
use crate::error::Error;

pub(crate) fn execute(_common: &CommonArgs, args: &VerifyArgs) -> Result<()> {
    tracing::debug!(no_build = args.no_build, "verify");
    Err(Error::NotImplemented("verify").into())
}
