---
name: bootstrap
description: Create a fresh debian/ directory for an unpackaged upstream source tree. Use when source.has_debian_dir is false. Output is builds-once, lintian-respectable, UNRELEASED packaging ready for maintainer review. Debian-first, Ubuntu overlay.
---

# debaid:bootstrap

Create a complete `debian/` directory from scratch for an upstream
source tree that has none.

## Preconditions

- A context JSON exists at `./.debaid/context.json`. If missing,
  build it: run `${CLAUDE_PLUGIN_ROOT}/scripts/detect-source.sh`
  and `${CLAUDE_PLUGIN_ROOT}/scripts/tooling-probe.sh`, merge their
  outputs (see `${CLAUDE_PLUGIN_ROOT}/shared-context.md` for the
  full schema).
- `source.has_debian_dir == false`. If `true`, refuse and suggest
  `/debaid:refresh` instead.
- A maintainer identity is available via `user.debfullname` and
  `user.debemail`. If either is missing, ask the maintainer before
  proceeding.

## What you produce

A minimal but complete `debian/` directory with:

- `debian/control` — house-style fields, accurate `Source:`,
  `Section:`, `Build-Depends:` derived from the build system, one
  or more binary stanzas. **Omit the `Priority:` field unless the
  package is `required`/`important`/`standard`** — Policy 4.7.3
  deprecated specifying the default (`optional`). `Standards-Version`
  is set from `${CLAUDE_PLUGIN_ROOT}/docs/house-style.md` (currently
  4.7.4). Long `Description:` is left as `<INSERT LONG DESCRIPTION HERE>`
  for the maintainer to fill in — NEVER invent prose marketing copy.
- `debian/changelog` — created with `dch --create
  --package=<name> --newversion=<upstream-version>-1
  "Initial packaging."` then immediately set distribution to
  `UNRELEASED`.
- `debian/copyright` — DEP-5 with one `Files: *` stanza for the
  upstream license (verified via `licensecheck -r .` AND a manual
  spot-check of file headers), plus one `Files: debian/*` stanza
  for the packaging.
- `debian/rules` — minimal `dh $@` form. Add `dh-sequence-*`
  Build-Depends instead of `--with`.
- `debian/source/format` — `3.0 (quilt)` (or `3.0 (native)` only
  if `source.upstream_vcs == none` AND version has no `-` revision).
- `debian/watch` — version 5. `pgpmode=auto` only if
  `debian/upstream/signing-key.asc` exists; otherwise `pgpmode=none`.
  For git-only upstreams, use `mode=git`.
- `debian/salsa-ci.yml` — include the pinned template version from
  `${CLAUDE_PLUGIN_ROOT}/docs/references/salsa-ci.md`.
- `debian/gbp.conf` — DEP-14 layout (`debian-branch = debian/unstable`,
  `upstream-branch = upstream/latest`, `pristine-tar = True` if the
  package will use it).

Templates live in `${CLAUDE_PLUGIN_ROOT}/skills/bootstrap/templates/`.
Render each through the house-style rules; do not copy verbatim.

## Template render flags

The templates use Mustache-style `{{var}}` and section
`{{#flag}}…{{/flag}}` / inverted `{{^flag}}…{{/flag}}` markers.
You substitute by string-matching — there is no separate render
engine. Compute each flag once at the start of the render step.

### Language dispatch

`rules.tmpl` is the generic `dh $@` fallback. When `source.language`
matches a language with a dedicated template, the renderer picks the
language-specific variant instead:

| `source.language` | Template selected |
|---|---|
| `python` | `rules.python.tmpl` |
| `rust` (application binary) | `rules.rust.tmpl` |
| `go` | `rules.golang.tmpl` |
| `perl` | `rules.perl.tmpl` |
| anything else | `rules.tmpl` |

The language-specific templates live alongside `rules.tmpl` in
`${CLAUDE_PLUGIN_ROOT}/skills/bootstrap/templates/` and inherit the
flags below. They land in later commits as each language overlay is
written; until a given template exists the dispatch falls back to
`rules.tmpl` for that language. The matching reference doc, when it
exists, lives at `${CLAUDE_PLUGIN_ROOT}/docs/references/languages/<lang>.md`
and the build-deps discovery step (process step 4) links into it.

Rust library crates do not go through this dispatch at all — see
"Bail-out conditions" below.

### Boolean flags

| Flag | True when | Notes |
|---|---|---|
| `priority_nondefault` | Target `Priority:` is one of `required`, `important`, `standard`. | Otherwise the field is omitted (Policy 4.7.3 §5.6.6 — `optional` is the default and should not be stated). Default behaviour for a fresh bootstrap is `false`. |
| `has_compiled_binaries` | `source.language` ∈ `{c, cpp, rust, go, haskell, ada, fortran}` — anything that produces native object code. | Gates the `DEB_BUILD_MAINT_OPTIONS = hardening=+all` export in `rules.tmpl`. Pure-Python/Perl/data packages get nothing. |
| `has_signing_key` | `debian/upstream/signing-key.asc` exists in the source tree. | Gates `Pgp-Mode: auto` vs `Pgp-Mode: none` in `watch.tmpl`. Check the path explicitly; do not infer from upstream metadata. |
| `has_template` | The upstream release pattern matches a known watch v5 `Template:` (`github`, `gitlab`, `pypi`, `sourceforge`, `cpan`, `git`). | When true, emit `Template: <name>` plus the template's required fields. When false, fall back to raw v5 fields. See `${CLAUDE_PLUGIN_ROOT}/docs/references/watch-v5.md`. |

### Scalar values

Pulled from context unless noted:

| Variable | Source |
|---|---|
| `source`, `section`, `priority`, `binary`, `architecture`, `short_description` | Sanity-check step (ask the maintainer or derive from upstream metadata; never invent). |
| `debfullname`, `debemail` | `user.debfullname`, `user.debemail` from `${DEBAID_CONTEXT}` / `./.debaid/context.json`. |
| `homepage`, `vcs_browser`, `vcs_git` | Sanity-check / derived from upstream metadata. |
| `upstream_name`, `upstream_contact`, `upstream_copyright_years`, `upstream_copyright_holder`, `upstream_license_short` | License-discovery step (`licensecheck -r .` plus a manual read of file headers). `upstream_license_short` is the SPDX-style short identifier (e.g. `MIT`, `Apache-2.0`, `GPL-2+`). |
| `packaging_year` | `date +%Y` at render time. |
| `pristine_tar` | `True` or `False` — `False` for a fresh bootstrap unless the maintainer asks for pristine-tar. |
| `template_name`, `template_specific_flags`, `watch_source_fields` | See watch v5 reference for the field set. |
| `language` | `source.language` from `${DEBAID_CONTEXT}` / `./.debaid/context.json`. Drives the template dispatch above. |
| `pybuild_name` | Used by `rules.python.tmpl`. The upstream *import* name (what you write in `import …`), not the source-package name. See `${CLAUDE_PLUGIN_ROOT}/docs/references/languages/python.md` § "Package naming". |

### List flags

| Flag | Shape |
|---|---|
| `build_deps` | List of objects `{ "name": "...", "version": "..." }`. Iterate with `{{#build_deps}} {{name}}{{#version}} ({{version}}){{/version}},\n{{/build_deps}}`. Computed from the build-deps discovery step. Always include `debhelper-compat (= 13)` in the static template body, not in this list. |

When in doubt about a flag's value, **ask the maintainer**
rather than guess — bootstrap is the most consequential phase
and a wrong default ripples into every later run.

## What you do NOT produce

- A finished long-form `Description:`. Leave the placeholder.
- A populated `debian/$pkg.install` unless the build system makes
  the install layout unambiguous (e.g. a single binary going to
  `/usr/bin/`). Otherwise, defer to the maintainer.
- A populated `debian/tests/` — that's `/debaid:autopkgtest`.
- `debian/upstream/metadata` — propose it as a follow-up; do not
  bootstrap it (DEP-12 fields need maintainer verification).
- `debian/upstream/signing-key.asc` — you cannot verify a key;
  the maintainer must place this.
- Lintian overrides at bootstrap time. If a tag fires, fix it,
  don't suppress.

## Process

1. **Load context.**
2. **Sanity-check inputs.** Confirm with maintainer: package name,
   upstream version, section (suggest one, ask to confirm),
   priority (default `optional`).
3. **License discovery.** Run `licensecheck -r .` and read the
   top-level `LICENSE`/`COPYING`/`README` files. For ambiguous
   results, ASK rather than guess.
4. **Build-deps discovery.** Read the build-system config
   (`Cargo.toml`, `go.mod`, `pyproject.toml`, `configure.ac`,
   `CMakeLists.txt`, etc.) and translate to Debian package names
   when known. For unknown mappings, list them in a follow-up
   block for the maintainer. When `source.language` matches a
   language overlay, consult the overlay for the canonical
   Build-Depends template:

   - **Python** (`pyproject.toml` / `setup.py` / `setup.cfg`):
     `${CLAUDE_PLUGIN_ROOT}/docs/references/languages/python.md`
     § "debian/control essentials".
   - **Rust application binaries** (`Cargo.toml` with `[[bin]]`):
     `${CLAUDE_PLUGIN_ROOT}/docs/references/languages/rust.md`
     § "Application binaries → dh-cargo". For Rust **library
     crates** (`[lib]`, no `[[bin]]`), see the "Rust library
     crate" entry in § "Bail-out conditions" below.
   - **Go** (`go.mod`):
     `${CLAUDE_PLUGIN_ROOT}/docs/references/languages/golang.md`
     § "debian/control essentials". Library vs application
     shape is decided per § "Library packages vs. application
     binaries".
   - **Perl** (`Makefile.PL` / `Build.PL` / `META.json` /
     `META.yml`):
     `${CLAUDE_PLUGIN_ROOT}/docs/references/languages/perl.md`
     § "debian/control essentials". Note that file is marked
     DRAFT pending pkg-perl maintainer review.
5. **Generate `debian/`** from templates + computed values.
6. **`wrap-and-sort -ast`** on the result.
7. **First verify.** Call `${CLAUDE_PLUGIN_ROOT}/scripts/verify.sh`
   to run sbuild (or fall back to dpkg-buildpackage) + lintian.
   Enter the iteration-budget loop (see
   `${CLAUDE_PLUGIN_ROOT}/shared-context.md` § "Iteration-budget
   envelope"). If neither builder is available, report and stop —
   the maintainer must set up a build environment before bootstrap
   verification can complete.
   If the verify failure is an upstream source issue and a patch is
   the right fix, follow the patches workflow in
   `${CLAUDE_PLUGIN_ROOT}/docs/house-style.md` § "Patches" — author
   via `gbp pq`, never write to `debian/patches/` directly. Patch
   authoring at bootstrap time is rare; prefer bailing to the
   maintainer over guessing a patch.
8. **Report.** Print:
   - Files created.
   - First build result.
   - First lintian result.
   - Any unresolved questions the maintainer must answer
     (Description, ambiguous license, unknown build-dep mappings).

## Hard rules

Suite-wide (apply to every debaid skill):

- Never invoke `dput`, `debrelease`, `dgit push`, or any upload
  command.
- Never `git push` to any remote.
- Never edit `debian/changelog` distribution from `UNRELEASED` to
  anything else.
- Never edit `Maintainer:` or `Uploaders:` fields.
- Never edit upstream source files directly (use `debian/patches/`
  with DEP-3 headers).
- Never run `rm -rf` or `git clean -fdx` on the workspace.
- Never write a lintian override without a `# reason` comment.
- Never set `Multi-Arch: same` without verifying file paths.

Phase-specific (bootstrap):

- **Do NOT ship `dh_make` output.** You may invoke `dh_make
  --native=no --copyright=<spdx>` IN A TEMPORARY DIRECTORY to
  compare its choices against yours, but the generated files go
  to `/tmp`, not to `debian/`.
- **Do NOT invent a `Maintainer:` for the package.** Use
  `user.debfullname <user.debemail>` from context. If those are
  missing, ask.
- **Do NOT enable `pgpmode=auto`** without an existing signing
  key file.

## Bail-out conditions

- Cannot determine the upstream license unambiguously.
- Cannot map a build dependency to a Debian package.
- Build system is unsupported (no `dh-sequence-*` for it).
- First build fails AND you cannot identify a recovery path
  within budget.
- Source is a Rust **library crate** (Cargo manifest has `[lib]`
  and no `[[bin]]`). Bootstrap should hand off to `debcargo`,
  which manages its own packaging layout, rather than render a
  fresh `debian/` from templates. See
  `${CLAUDE_PLUGIN_ROOT}/docs/references/languages/rust.md`
  § "Library crates → debcargo" for the recommended workflow.

Use the bail-out summary format from
`${CLAUDE_PLUGIN_ROOT}/shared-context.md` § "Bail-out summary
format".
