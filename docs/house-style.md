# debaid — house style

**Version**: 2 · **Last reviewed**: 2026-05-13 · **Next review due**: 2026-08-13

Tracks Debian Policy 4.7.4 (March 2026).

This file is the source of truth for *prescriptive* packaging choices
that go beyond what Debian Policy mandates. Workers consult it on
every run via `${CLAUDE_PLUGIN_ROOT}/docs/house-style.md`; a
maintainer can override the active style by passing
`--house-style=<path>` to the orchestrator. Every rule below carries one of:

- **Policy** §x.y — Debian Policy Manual reference
- **devref** §x.y — Debian Developer's Reference reference
- **DEP-NN** — a DEP standard
- **DD-judgement** — opinion of the maintainer of this repo;
  callers may override with their own house style via
  `--house-style=<path>`.

> Workers MUST cite the rule when they apply it (e.g. in a diff
> hunk comment or bail-out summary). Maintainers will not accept
> opaque "because the LLM said so" changes.

---

## Source format

- `3.0 (quilt)` for non-native packages. **Policy §5.6.13.**
- `3.0 (native)` only for genuinely native packages (no upstream
  tarball, version has no Debian revision). **Policy §5.6.12.**
- Never `1.0`. **DD-judgement** — deprecated for new work.

## debhelper

- `debhelper-compat (= 13)` as Build-Depends (virtual package).
  **DD-judgement** — 13 is battle-tested now
- Use `dh-sequence-<name>` virtual Build-Depends to load add-ons,
  not `--with` in `debian/rules`. **DD-judgement** — declarative,
  surfaces dependencies to apt, lintian-checkable.
- `debian/rules` minimal form:
  ```make
  #!/usr/bin/make -f
  %:
  	dh $@
  ```
  Override targets only when necessary; never preemptively.
  **DD-judgement.**

## Control fields

- `Standards-Version: 4.7.4`. **Policy** — pin a known-good value;
  bump only as a deliberate refresh, never auto-bumped per run.
  Re-pin at every quarterly review.
- `Rules-Requires-Root`: **omit the field** for the default case.
  **Policy §4.9.2 (4.7.x).** `no` is the policy default, so
  specifying `Rules-Requires-Root: no` in `debian/control` is
  redundant; refresh workers MAY propose removing such a line.
  Set the field explicitly to `binary-targets` if (and only if)
  the package installs setuid/setgid files or specific ownership
  that fakeroot can't preserve. Workers MUST detect setuid/setgid
  build outputs and warn.
- `Priority:`: **omit the field** for the default case. **Policy
  §5.6.6 (4.7.3)** — specifying `Priority` in source control fields
  is no longer recommended unless the value needs to differ from the
  default. Omitting it inherits `optional`, which is what most
  packages want. Set the field explicitly only for `required`,
  `important`, or `standard` per **Policy §2.5**. Refresh workers
  MAY propose removing a redundant `Priority: optional` line.
- `Section`: pick from
  https://packages.debian.org/unstable/ — workers MUST NOT invent
  a section.
- `Vcs-Git`/`Vcs-Browser`: present and correct, pointing at the
  Salsa repo by default (Debian-first). Ubuntu overlays point at
  Launchpad-hosted git via the git-ubuntu workflow — see
  `docs/references/ubuntu-merges-syncs.md`. **DD-judgement.**
- `Maintainer`/`Uploaders`: workers NEVER mutate these. **DD-judgement.**
- `Git-Tag-Tagger:` / `Git-Tag-Info:` (Policy 4.7.3, §5.6.32 &
  §5.6.33): new optional source-control fields recording the
  tagger and tag annotation that produced the release tag. Workers
  MAY leave these to `gbp` / the release tooling; if a maintainer
  is using them, refresh MUST NOT strip them. See
  `docs/references/git-tag-fields.md`.

## Binary packages

- `Multi-Arch:` annotations are advisory-only from workers. The
  worker may propose `same` for libraries with arch-specific paths,
  `foreign` for arch-independent helper binaries, but MUST verify
  no file collisions across architectures before recommending.
  **Policy §5.6** (Multi-Arch field, documented in Policy 4.7.4).
  Setting `M-A: same` blindly breaks coinstallation.
- `Pre-Depends:` only when essential (e.g. `${misc:Pre-Depends}`
  for multiarch). Workers MUST justify any other Pre-Depends.
  **Policy §7.2.**

## debian/copyright

- DEP-5 machine-readable format always. **DEP-5.**
- For vendored dependencies or large source trees, default to
  asking the maintainer rather than guessing. **DD-judgement** —
  bad DEP-5 is worse than no DEP-5.
- `licensecheck -r .` is a starting point, not an oracle. Workers
  MUST verify each `Files:` stanza against actual file headers.
- For packages destined for `non-free` **or `non-free-firmware`**,
  `debian/copyright` MUST explain why the package is not part of
  Debian. **Policy §12.5 (4.7.4).** Workers detecting either
  archive area MUST surface this requirement.

## debian/changelog

- `dch --create` (new package) or `dch -i` (new entry) — never
  hand-edit the format. **DD-judgement.**
- Distribution `UNRELEASED` until the maintainer explicitly approves
  a release. Workers NEVER set `unstable`, `trixie-backports`,
  `noble`, etc. **DD-judgement** — release is a human action.
- Workers NEVER bump the version on a refresh run. Refresh ≠ release.

## debian/watch

- Version 5 syntax is the only supported target. Refresh upgrades
  any earlier version (v3, v4) directly to v5; there is no
  intermediate v3→v4 step. **DD-judgement.** See
  `${CLAUDE_PLUGIN_ROOT}/docs/references/watch-v5.md` for worked
  examples and the v4→v5 migration table.
- `Pgp-Mode: auto` (or `mangle`) ONLY when `debian/upstream/signing-key.asc`
  exists. Otherwise `Pgp-Mode: none`.  Workers MUST check for the key before
  enabling signature verification.
- Use the proper fields depending on the existence of a template for the
  source, ask the developer if needed rather than guessing or parsing manpages.
- Prefer `Template: git` for projects without tarball releases.
  **DD-judgement.**

## debian/source/format and options

- `3.0 (quilt)`: yes.
- `debian/source/options`: omit unless needed. Common justified
  exception: `extend-diff-ignore` for generated files the
  maintainer can't get upstream to add to `.gitignore`. **DD-judgement.**
- `debian/source/local-options`: never check in. **devref §6.7.4.**

## Patches

Debaid assumes a `git-buildpackage` / DEP-14 workflow throughout.
Workers operating on a package that is **not** packaged with `gbp`
(no `debian/gbp.conf`, no DEP-14 branch layout) MUST bail to the
maintainer with a clear message rather than try to translate the
quilt-only workflow on the fly.

- **All patches are authored via `git-buildpackage`'s patch
  queue.** **DD-judgement** — uniform workflow, integrates with
  DEP-14.
- **Use `gbp pq import` to create the `patch-queue/<current-branch>`
  branch** — e.g. `patch-queue/debian/unstable` when on
  `debian/unstable`. If that branch already exists, ask the
  maintainer to choose: (a) reuse it as-is, (b) drop it and
  reimport from `debian/patches/`, or (c) inspect for un-exported
  work before deciding. **DD-judgement.**
- **Author new patches as commits on `patch-queue/<branch>`.** One
  logical change per commit; the commit message becomes the
  patch's DEP-3 `Description:` and `Author:` headers. `gbp pq
  export` materialises each commit as one `.patch` file under
  `debian/patches/`. Never edit `debian/patches/*.patch` by hand.
  **DD-judgement.**
- **Review the diff produced by `gbp pq export` before committing
  the packaging branch** — confirm `debian/patches/series` and the
  generated `.patch` files match your intent.
  **DD-judgement.**
- **New upstream tarball import sequence**:
  - Ensure `patch-queue/<branch>` is current. On a clean packaging branch, run
    `gbp pq import` to create the `patch-queue/<branch>`, then `gbp pq switch`
    to come back to the packaging branch. If the import fails, hand the control
    to the maintainer.
  - Run `gbp import-orig --pristine-tar` to bring the new
    upstream in.
  - Run `gbp pq rebase` to re-anchor patches on the new upstream.
    Conflicts here are maintainer territory — the worker MUST
    surface them and stop.
  **DD-judgement.**
- All upstream modifications via `debian/patches/` with DEP-3
  headers (`Origin:`, `Forwarded:`, `Last-Update:`, `Author:`,
  `Description:`). **DEP-3.**
- `debian/patches/series` ordered, no blank lines, no comments
  (`#`-lines are tolerated but discouraged). **DD-judgement.**
- Workers NEVER edit upstream sources in-place to "fix" a build.
  Always a patch. **DD-judgement.**

## Salsa-CI

- `debian/salsa-ci.yml` present for new packages. **DD-judgement.**
- Pin the template version explicitly; do not `include:` a
  `master` ref. (See `docs/references/salsa-ci.md`.)

## Formatting

- `wrap-and-sort -ast` (or `-abst` if the maintainer prefers
  blank-line groups) applied to:
  `debian/control`, `debian/copyright`, `debian/*.install`,
  `debian/*.examples`, `debian/*.docs`.
  **DD-judgement** — diff-stable, review-friendly.

## VCS layout

- DEP-14: `debian/unstable` for development, `pristine-tar` branch
  for tarballs when used, `upstream/latest` for upstream imports.
  **DEP-14.**
- Workers MUST detect existing layout and not assume `master` is
  the packaging branch.
- Tags signed (`gbp buildpackage --git-tag-only --git-sign-tags`)
  when the maintainer has a key configured. **DD-judgement.**

## Autopkgtest

- `debian/tests/control` minimum `Restrictions:` to make the test
  pass. **DEP-8.**
- `Restrictions: needs-root, isolation-container` only when
  genuinely required and with a written justification. **DD-judgement.**
- Library packages: ABI smoke (load + a trivial symbol). Daemons:
  start + ping + stop, isolation-container scoped. CLI tools: invoke
  with `--version` at minimum, then the most common subcommand.

## Hardening

- Set `export DEB_BUILD_MAINT_OPTIONS = hardening=+all` in `debian/rules` by
  default; Override the hardening set only with explicit justification (e.g. an
  upstream that breaks under FORTIFY). **DD-judgement.**
- `blhc` clean on the build log. **DD-judgement.**

## Lintian

- Target: `lintian -EvIL +pedantic` clean OR overridden with
  justification.
- Overrides MUST have a `# reason` comment line directly above
  the override. **DD-judgement** — opaque overrides rot.
- Place overrides in `debian/source/lintian-overrides` for
  source-level tags, `debian/$pkg.lintian-overrides` for
  binary-level tags. **Policy/lintian convention.**

## Build profiles

- Policy 4.7.4 documents build profiles formally. Workers MAY
  propose adding `<!nocheck>`, `<!nodoc>`, `<stage1>` etc. when
  appropriate, but MUST cite a concrete reason (e.g. bootstrap
  cycle break) — never preemptively. **Policy** (build profiles).
- See `docs/references/build-profiles.md`.

## Ubuntu overlay

When `target.distro == ubuntu`, the following overlay applies on
top of the Debian-first rules above. See
`docs/references/ubuntu-versioning.md`,
`docs/references/ubuntu-merges-syncs.md`, and
`docs/references/sru.md` for the full background.

- **Versioning.** New Ubuntu changes on top of a Debian package
  append `ubuntu1`, `ubuntu2`, ... to the Debian revision (e.g.
  `2.0-2ubuntu1`). No-change rebuilds use `buildN`. The
  `willsync` suffix exists for the auto-sync case. Workers MUST
  NOT invent any other suffix.
- **Distribution.** Workers NEVER set `dch -D <release>`. The
  maintainer chooses `noble`, `noble-proposed`, etc. when they
  release.
- **Pocket.** Uploads targeting a released series go to
  `<series>-proposed` for SRU, never directly to `<series>`. The
  orchestrator MUST surface this when `target.pocket != dev`.
- **Maintainer field.** Ubuntu packages set
  `Maintainer: Ubuntu Developers <ubuntu-devel-discuss@lists.ubuntu.com>`
  with the prior Debian maintainer moved to `XSBC-Original-Maintainer:`.
  This is handled by `update-maintainer` from `ubuntu-dev-tools`;
  workers NEVER touch it by hand. **DD-judgement** + Ubuntu policy.
- **SRU regression-potential.** Any changelog entry targeting a
  stable Ubuntu release MUST include a regression-potential
  paragraph; the maintainer writes it, not the worker. See
  `docs/references/sru.md`.

---

## Notes for future-me

- Re-check Standards-Version, debhelper-compat default, and
  Salsa-CI template version at every quarterly review.
- Re-check language-overlay drift hot-spots: pybuild major
  version and `dh-sequence-python3` behaviour
  (`docs/references/languages/python.md`); dh-cargo /
  debcargo workflow changes
  (`docs/references/languages/rust.md`); dh-golang major
  version and `XS-Go-Import-Path:` conventions
  (`docs/references/languages/golang.md`); perl version
  transitions, pkg-perl conventions, and the DRAFT marker
  status (`docs/references/languages/perl.md`).
- Re-read `/usr/share/doc/debian-policy/upgrading-checklist.txt.gz`
  at every quarterly review and update the citations above.
- New DEPs accepted between reviews: add to this file before
  workers can rely on them.
- Items marked DD-judgement here may be overridden in a
  downstream house-style file passed via `--house-style=<path>`.
  Workers MUST then cite "house-style (custom)" rather than
  "DD-judgement".

