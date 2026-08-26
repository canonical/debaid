//! The build + lintian snapshot emitted by `scripts/verify.sh`.
//!
//! This is the shape; `shared-context.md` § "Verify snapshot semantics"
//! carries the meaning. The script is stateless — it emits one
//! [`VerifySnapshot`] per run, and workers hold previous snapshots
//! themselves to compute progress across iterations.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifySnapshot {
    pub build: Build,
    pub lintian: Lintian,
    /// Changed lines under `debian/` relative to the git index; `None` when
    /// the source tree is not a git repository.
    #[serde(default)]
    pub diff_size_lines: Option<u32>,
}

impl VerifySnapshot {
    pub fn build_passed(&self) -> bool {
        self.build.ran && self.build.ok
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Build {
    pub tool: BuildTool,
    /// False when `--no-build` was passed or no builder was available.
    pub ran: bool,
    /// Meaningful only when `ran`.
    pub ok: bool,
    /// `verify.sh` always sets this, including when `!ran` — the log then
    /// records why. Optional only to tolerate other producers.
    #[serde(default)]
    pub log_path: Option<PathBuf>,
    pub exit_code: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BuildTool {
    Sbuild,
    #[serde(rename = "dpkg-buildpackage")]
    DpkgBuildpackage,
    None,
}

/// Tag arrays may repeat a tag that fired on several files; the repetition
/// is signal, not noise.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lintian {
    pub ran: bool,
    pub scope: LintianScope,
    /// Always set by `verify.sh`; see [`Build::log_path`].
    #[serde(default)]
    pub log_path: Option<PathBuf>,
    #[serde(default)]
    pub errors: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub infos: Vec<String>,
    #[serde(default)]
    pub pedantics: Vec<String>,
    /// `N: Overridden:` lines — tags an existing override already suppresses.
    #[serde(default)]
    pub overrides_applied: u32,
}

impl Lintian {
    /// Weight a clean result less when [`LintianScope::SourceTree`]:
    /// binary-only checks never fired.
    pub fn is_clean(&self) -> bool {
        self.ran && self.errors.is_empty() && self.warnings.is_empty()
    }
}

/// `Changes` is authoritative; `SourceTree` is degraded — binary-only
/// checks do not fire there.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LintianScope {
    Changes,
    Dsc,
    SourceTree,
    None,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_the_v1_schema() {
        let snap: VerifySnapshot = serde_json::from_value(serde_json::json!({
            "build": {
                "tool": "sbuild",
                "ran": true,
                "ok": true,
                "log_path": "/tmp/verify-build.abc.log",
                "exit_code": 0
            },
            "lintian": {
                "ran": true,
                "scope": "changes",
                "log_path": "/tmp/verify-lintian.abc.log",
                "errors": [],
                "warnings": ["package-contains-foo", "package-contains-foo"],
                "infos": [],
                "pedantics": [],
                "overrides_applied": 1
            },
            "diff_size_lines": 42
        }))
        .unwrap();

        assert_eq!(snap.build.tool, BuildTool::Sbuild);
        assert!(snap.build_passed());
        assert_eq!(snap.lintian.scope, LintianScope::Changes);
        assert_eq!(snap.lintian.warnings.len(), 2);
        assert!(!snap.lintian.is_clean());
        assert_eq!(snap.diff_size_lines, Some(42));
    }

    #[test]
    fn handles_no_build_and_non_git_tree() {
        let snap: VerifySnapshot = serde_json::from_value(serde_json::json!({
            "build": {"tool": "none", "ran": false, "ok": false, "exit_code": 0},
            "lintian": {"ran": true, "scope": "source-tree"}
        }))
        .unwrap();

        assert_eq!(snap.build.tool, BuildTool::None);
        assert!(!snap.build_passed());
        assert_eq!(snap.lintian.scope, LintianScope::SourceTree);
        assert!(snap.lintian.is_clean());
        assert!(snap.lintian.errors.is_empty());
        assert_eq!(snap.diff_size_lines, None);
    }

    #[test]
    fn dpkg_buildpackage_renames_correctly() {
        let build: Build = serde_json::from_value(serde_json::json!({
            "tool": "dpkg-buildpackage", "ran": true, "ok": false, "exit_code": 2
        }))
        .unwrap();
        assert_eq!(build.tool, BuildTool::DpkgBuildpackage);
        let back = serde_json::to_value(&build).unwrap();
        assert_eq!(back["tool"], "dpkg-buildpackage");
    }
}
