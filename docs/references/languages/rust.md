# Rust — debaid overlay

Debian Rust packaging splits sharply into two paths depending on
whether the upstream is a **library crate** (consumed by other
Rust packages) or an **application binary** (consumed by end
users). Read alongside `${CLAUDE_PLUGIN_ROOT}/docs/house-style.md`;
this file documents what is *Rust-specific*.

Authoritative upstream sources:

- Debian Rust Team: https://salsa.debian.org/rust-team
- `debcargo`: https://salsa.debian.org/rust-team/debcargo
- `debcargo-conf`: https://salsa.debian.org/rust-team/debcargo-conf
- `man dh-cargo`, `man debcargo`.

## When this overlay applies

`source.language == "rust"` in the shared context. Detected by
the presence of `Cargo.toml` at the source root.

## Library crates vs. application binaries

**Decide first.** Inspect `Cargo.toml`:

- `[lib]` present, **no** `[[bin]]` → **library crate**. Hand off
  to `debcargo` (see "Library crates" below). Bootstrap does not
  render a fresh `debian/`.
- `[[bin]]` present (with or without `[lib]`) → **application
  binary**. Use the `dh-cargo` path (see "Application binaries"
  below). Bootstrap renders the standard `debian/` from
  `rules.rust.tmpl`.
- Workspace (`[workspace]` only, no top-level `[lib]` or
  `[[bin]]`) → inspect each workspace member separately; in
  practice these are usually library collections destined for
  the Rust team's monorepo.

The two paths are not interchangeable:

- Library crates packaged via the dh-cargo path lose the
  automatic feature-flag handling and crate-version dependency
  graph that the Rust team relies on for the thousands of
  packaged crates in unstable.
- Application binaries packaged via debcargo end up with a
  source-package name like `rust-foo` rather than `foo`, which
  is unhelpful for end users.

## Library crates → debcargo

**Bootstrap bail-out.** When the source is a library crate,
bootstrap should not render a `debian/` directory at all.
Instead, direct the maintainer to debcargo:

```
debcargo new <crate-name> [<version>]
```

This produces a `debian/` shaped for the Rust-team monorepo.
The recommended workflow is to clone `debcargo-conf` and add
the crate as a new subdirectory under `src/<name>/`, with a
`debcargo.toml` override file.

### debcargo.toml essentials

Common overrides the maintainer will need to write:

```toml
overlay = "."
uploaders = ["Maintainer Name <user@debian.org>"]

[source]
section = "rust"
policy = "4.7.4"

[packages."librust-foo-dev"]
summary = "<short crate description>"
description = """
<long description>
"""
```

Feature flags are handled automatically per `Cargo.toml`
features; only override `[features]` in `debcargo.toml` when
the defaults are wrong (e.g. a feature pulls in an undesired
optional dep, or upstream's default feature set is unusable
on Debian).

### Versioning

debcargo follows Cargo's semver model. The Debian source name
encodes the major version: `rust-foo` for 1.x, `rust-foo-2`
for 2.x when an ABI-incompatible 2.x must coexist with 1.x.
This is debcargo's responsibility — don't fight it.

### Bootstrap with no existing crates packaged

When a crate's transitive dependencies aren't yet in Debian,
debcargo emits one source package per crate. Plan to upload
the dependency graph leaf-first; the Rust team's
`debcargo-conf` queues exist for this reason.

## Application binaries → dh-cargo

For Rust applications (a CLI tool or a daemon) the standard
debaid flow applies; `rules.rust.tmpl` renders the right
`debian/rules`.

### debian/control essentials

Build-Depends template:

```
Build-Depends:
 debhelper-compat (= 13),
 dh-cargo,
 cargo,
 rustc,
```

Add `librust-<dep>-dev (>= …)` lines for runtime crate
dependencies the application needs at build time. For most
applications, dh-cargo discovers these from `Cargo.toml`; only
list them when the maintainer wants explicit version
constraints.

Binary stanza:

```
Package: foo
Architecture: any
Depends:
 ${shlibs:Depends},
 ${misc:Depends},
Description: …
```

`Architecture: any` always — Rust produces compiled binaries.
Hardening is on by default via `DEB_BUILD_MAINT_OPTIONS =
hardening=+all`, set by `rules.rust.tmpl` via the
`has_compiled_binaries` flag.

### debian/rules

`rules.rust.tmpl` produces:

```makefile
#!/usr/bin/make -f
export DEB_BUILD_MAINT_OPTIONS = hardening=+all
%:
	dh $@ --buildsystem=cargo
```

Override `dh_auto_test` only when upstream's `cargo test`
requires network or has flaky tests; document why with an
inline comment.

### Cargo.lock

Application binaries should ship `Cargo.lock`, or have it
regenerated reproducibly at build time. Library crates
packaged via debcargo do not ship `Cargo.lock` — debcargo
strips it because each crate is consumed independently.

### debian/watch

Rust applications usually release on GitHub or GitLab, not on
crates.io directly. Use `Template: github` (see
`${CLAUDE_PLUGIN_ROOT}/docs/references/watch-v5.md`):

```
version=5
Template: github
URL: https://github.com/<owner>/<repo>
```

For crates.io-only upstreams (uncommon for applications), fall
back to raw v5 fields pointing at
`https://crates.io/crates/<name>` — there is no `crates.io`
template.

## autopkgtest

**No autodep8 generator exists for Rust.** Hand-roll
`debian/tests/` per the autopkgtest worker's normal flow:

- For an application binary, a `<bin> --version` /
  `<bin> --help` test is usually sufficient.
- For library-crate test packages produced by debcargo, the
  Rust team's CI runs `cargo test` on the built package; the
  maintainer rarely writes per-test scripts.

## Common refresh checks

The refresh skill applies these when `source.language ==
"rust"` and the package is an **application binary** —
debcargo-managed libraries are refreshed via the Rust team's
workflow, not via debaid:

- `dh-cargo`, `cargo`, `rustc` present in `Build-Depends`.
- `--buildsystem=cargo` used in `debian/rules` (not the legacy
  `--with cargo`).
- Hardening flags on (`DEB_BUILD_MAINT_OPTIONS = hardening=+all`).

If refresh detects the source is a library crate, surface that
finding to the maintainer and recommend the debcargo workflow
rather than trying to refresh a hand-written `debian/`.

## Bail-out conditions

In addition to the worker-level bail-outs:

- Source is a library crate — see § "Library crates" above.
  Bootstrap's "Rust library crate detected" bail-out in
  `${CLAUDE_PLUGIN_ROOT}/skills/bootstrap/SKILL.md` § "Bail-out
  conditions" applies.
- Workspace with mixed library and binary members — ask the
  maintainer which member to package and whether the rest
  should be packaged separately.
- Application binary depends on a library crate not yet in
  Debian; debcargo workflow for the missing crate is a
  prerequisite. Bail with the missing-crate list.
