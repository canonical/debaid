//! Serde data model for the two documents that cross the boundary between
//! the deterministic core and the agent layer.
//!
//! These types *are* the schema: `shared-context.md` documents what the
//! fields mean and deliberately does not restate their shape. They carry no
//! behaviour beyond a few accessors, so either side can depend on them
//! without coupling.

mod context;
mod verify;

pub use context::{
    BranchLayout, Budget, BuildSystem, Context, Distro, FreezeState, Language, Pocket, Source,
    Target, ToolStatus, Tooling, UpstreamVcs, User,
};
pub use verify::{Build, BuildTool, Lintian, LintianScope, VerifySnapshot};
