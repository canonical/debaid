//! The runtime context every worker consumes.
//!
//! This is the shape; `shared-context.md` carries the semantics. Two
//! deliberate departures from a literal reading of that document:
//!
//! - `tooling` is a map keyed by executable name, not one field per tool.
//!   The set of Debian helpers is large and mostly Ubuntu-merge-specific;
//!   enumerating them in the type bought nothing and drifted from what
//!   `tooling-probe.sh` actually probes.
//! - `release` and `host_arch` are free-form strings: new suites and ports
//!   appear on their own schedule, so enums would only add churn.
//!
//! Enums describing detected facts carry an `Unknown` variant marked
//! `#[serde(other)]`: unrecognised input degrades instead of failing the
//! parse, so a context written by a newer debaid stays readable by an older
//! one.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// The context document written to `${DEBAID_CONTEXT}` /
/// `./.debaid/context.json` and read by every worker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    pub schema_version: u32,
    /// ISO 8601 UTC.
    pub generated_at: String,
    pub source: Source,
    pub tooling: Tooling,
    pub target: Target,
    pub user: User,
    #[serde(default)]
    pub budget: Budget,
    #[serde(default)]
    pub reference_corpus: Option<PathBuf>,
    pub house_style: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub path: PathBuf,
    pub language: Language,
    pub build_system: BuildSystem,
    pub has_debian_dir: bool,
    pub has_quilt_patches: bool,
    pub debian_branch_layout: BranchLayout,
    pub upstream_vcs: UpstreamVcs,
    /// `Some(true)` if `debian/changelog` carries a non-`buildN` Ubuntu
    /// revision, `Some(false)` if in Ubuntu with no delta, `None` when the
    /// target is not Ubuntu. See `docs/references/ubuntu-merges-syncs.md`.
    #[serde(default)]
    pub ubuntu_delta: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Go,
    Rust,
    Python,
    C,
    Cpp,
    Java,
    Nodejs,
    Perl,
    Ruby,
    Haskell,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BuildSystem {
    Autotools,
    Meson,
    Cmake,
    Cargo,
    #[serde(rename = "go-mod")]
    GoMod,
    Setuptools,
    Pyproject,
    Nodejs,
    Make,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BranchLayout {
    Monorepo,
    Dep14,
    SeparateBranch,
    None,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UpstreamVcs {
    Git,
    Hg,
    Svn,
    Tarball,
    None,
    #[serde(other)]
    Unknown,
}

/// Keyed by executable name. An absent key means "not probed", which is
/// distinct from a present entry with `available: false`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Tooling(pub BTreeMap<String, ToolStatus>);

impl Tooling {
    pub fn get(&self, tool: &str) -> Option<&ToolStatus> {
        self.0.get(tool)
    }

    /// Probed *and* found on `PATH`.
    pub fn available(&self, tool: &str) -> bool {
        self.0.get(tool).is_some_and(|status| status.available)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolStatus {
    pub available: bool,
    #[serde(default)]
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    pub distro: Distro,
    /// Suite or codename (`unstable`, `trixie`, `noble`, …); open set.
    pub release: String,
    #[serde(default)]
    pub pocket: Pocket,
    #[serde(default)]
    pub freeze_state: FreezeState,
    /// dpkg host architecture (`amd64`, `arm64`, …); open set.
    pub host_arch: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Distro {
    Debian,
    Ubuntu,
}

/// Anything other than [`Pocket::Dev`] pulls in the SRU workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Pocket {
    #[default]
    Dev,
    Proposed,
    Updates,
    Security,
    Backports,
}

/// Detection MAY leave this [`FreezeState::Unknown`]; workers then default
/// to cautious behaviour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FreezeState {
    None,
    DebianImportFreeze,
    FeatureFreeze,
    FinalFreeze,
    #[default]
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// `DEBFULLNAME`, else `git config user.name`.
    pub debfullname: String,
    /// `DEBEMAIL`, else `git config user.email`.
    pub debemail: String,
}

/// The iteration-budget envelope every mutating worker must honour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Budget {
    /// Attempts at one error class before bailing to the maintainer.
    pub max_attempts_per_error_class: u32,
    /// Diff size above which a worker must ask before proceeding.
    pub diff_threshold_lines: u32,
    /// Identical error recurrences before bailing regardless of attempts.
    pub repeat_budget: u32,
}

impl Default for Budget {
    fn default() -> Self {
        Self {
            max_attempts_per_error_class: 3,
            diff_threshold_lines: 200,
            repeat_budget: 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_json() -> serde_json::Value {
        serde_json::json!({
            "schema_version": 1,
            "generated_at": "2026-08-25T12:00:00Z",
            "source": {
                "path": "/home/dev/pkg",
                "language": "rust",
                "build_system": "cargo",
                "has_debian_dir": true,
                "has_quilt_patches": false,
                "debian_branch_layout": "separate-branch",
                "upstream_vcs": "git",
                "ubuntu_delta": null
            },
            "tooling": {
                "sbuild":  {"available": true,  "version": "0.85"},
                "lintian": {"available": true,  "version": "2.117"},
                "debputy": {"available": false, "version": null}
            },
            "target": {
                "distro": "debian",
                "release": "unstable",
                "pocket": "dev",
                "freeze_state": "none",
                "host_arch": "amd64"
            },
            "user": {
                "debfullname": "Ada Lovelace",
                "debemail": "ada@example.org"
            },
            "budget": {
                "max_attempts_per_error_class": 3,
                "diff_threshold_lines": 200,
                "repeat_budget": 2
            },
            "reference_corpus": null,
            "house_style": "/opt/debaid/house-style.md"
        })
    }

    #[test]
    fn deserializes_the_v1_schema() {
        let ctx: Context = serde_json::from_value(sample_json()).unwrap();
        assert_eq!(ctx.schema_version, 1);
        assert_eq!(ctx.source.language, Language::Rust);
        assert_eq!(ctx.source.build_system, BuildSystem::Cargo);
        assert_eq!(
            ctx.source.debian_branch_layout,
            BranchLayout::SeparateBranch
        );
        assert!(ctx.tooling.available("sbuild"));
        assert!(!ctx.tooling.available("debputy"));
        assert!(!ctx.tooling.available("never-probed"));
        assert_eq!(ctx.target.distro, Distro::Debian);
        assert_eq!(ctx.target.pocket, Pocket::Dev);
        assert_eq!(ctx.source.ubuntu_delta, None);
        assert!(ctx.reference_corpus.is_none());
    }

    #[test]
    fn round_trips_through_json() {
        let ctx: Context = serde_json::from_value(sample_json()).unwrap();
        let back = serde_json::to_value(&ctx).unwrap();
        let ctx2: Context = serde_json::from_value(back).unwrap();
        assert_eq!(ctx.source.language, ctx2.source.language);
        assert_eq!(ctx.target.release, ctx2.target.release);
    }

    #[test]
    fn unknown_enum_values_fall_back_rather_than_erroring() {
        let ctx: Context = serde_json::from_value(serde_json::json!({
            "schema_version": 1,
            "generated_at": "2026-08-25T12:00:00Z",
            "source": {
                "path": "/x",
                "language": "brainfuck",
                "build_system": "waf",
                "has_debian_dir": false,
                "has_quilt_patches": false,
                "debian_branch_layout": "wat",
                "upstream_vcs": "fossil",
                "ubuntu_delta": true
            },
            "tooling": {},
            "target": {
                "distro": "ubuntu",
                "release": "resolute",
                "host_arch": "riscv64"
            },
            "user": {"debfullname": "x", "debemail": "y"},
            "house_style": "/hs.md"
        }))
        .unwrap();
        assert_eq!(ctx.source.language, Language::Unknown);
        assert_eq!(ctx.source.build_system, BuildSystem::Unknown);
        assert_eq!(ctx.source.debian_branch_layout, BranchLayout::Unknown);
        assert_eq!(ctx.source.upstream_vcs, UpstreamVcs::Unknown);
        // Omitted optional fields take their defaults.
        assert_eq!(ctx.target.pocket, Pocket::Dev);
        assert_eq!(ctx.target.freeze_state, FreezeState::Unknown);
        assert_eq!(ctx.budget, Budget::default());
    }
}
