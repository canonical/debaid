use anyhow::Result;

use crate::cli::{CommonArgs, DetectArgs};
use crate::error::Error;

pub(crate) fn execute(_common: &CommonArgs, args: &DetectArgs) -> Result<()> {
    tracing::debug!(path = %args.path.display(), "detect");
    Err(Error::NotImplemented("detect").into())
}
