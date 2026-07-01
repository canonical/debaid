# Plan: language overlays for Python, Rust, Go, Perl

## Context

debaid ships as a phase-based plugin: `run` orchestrates
`bootstrap` / `refresh` / `lintian` / `autopkgtest`, all
coordinated by a shared context JSON and a prescriptive
`docs/house-style.md`. The plugin is **language-agnostic by
design today**: `source.language` is detected but no worker uses
that signal to apply language-specific Debian conventions
(pybuild, debcargo, dh-golang, dh-make-perl). For a maintainer
that means workers hand back a generic `dh $@` skeleton and
leave every language nuance to the human.

Other debian packaging agent skills do exist online. We chose to do things
our way both for licensing matters and to adhere to the house style we set.
Most of these productions are the consequence of a back and forth with Claude
and reviews/edition from Ubuntu or Debian Developers.

The goal of this pass: ship Python / Rust / Go / Perl
overlays so that the existing phase workers specialise their
output by detected language, **without expanding the skill
count or duplicating knowledge**.

## Decisions locked in with the user

1. **Overlay shape** — `docs/references/languages/<lang>.md`.
   Mirrors the existing `house-style.md` / `references/`
   pattern. Workers cite specific sections; no per-language
   skill; no duplication.
2. **Scope** — Python, Rust, Go, **Perl** (drafted from public
   Debian Perl Group knowledge — no upstream content to lean
   on). Ruby deferred.
3. **Templates** — yes to per-language `rules.tmpl` variants
   (`rules.python.tmpl`, `rules.rust.tmpl`, `rules.golang.tmpl`,
   `rules.perl.tmpl`). Bootstrap selects by `source.language`.
4. **Workers** — bootstrap, refresh, autopkgtest. `lintian`
   deferred.
5. **License** — no licence chosen yet for debaid; commit 1
   ships a placeholder LICENSE. The clean-redesign approach
   means licence selection is independent of this plan.
6. **Fixtures** — `tests/fixtures/*` are stubs by design. A
   future pass will promote all of them together (upstream
   source + `expected/debian/` + `test.sh`). This plan does
   **not** add per-commit fixture assertions; that was a
   misread of the harness, corrected during execution.

## Commit-by-commit roadmap

Each commit is atomic and reviewable in isolation. Commits 1–4
introduce shared infrastructure; commits 5–8 plug in one
language each; commits 9–11 are polish.

### ✅ Commit 1 — `legal: add LICENSE placeholder`
**Done.** Added a 17-line placeholder LICENSE noting no formal
licence is yet chosen, asking reviewers not to redistribute,
and stating contributions will be relicensable. No NOTICE
(clean-room redesign means no upstream attribution required).

### ✅ Commit 2 — `bootstrap: introduce language-keyed template dispatch`
**Done.** `skills/bootstrap/SKILL.md` grew:
- New `### Language dispatch` subsection with the
  `source.language` → template-file mapping table. Falls back
  to `rules.tmpl` for unknown languages.
- New `language` scalar in the variables table.
- New bail-out condition for Rust library crates (Cargo
  manifest with `[lib]` and no `[[bin]]`).

### ✅ Commit 3 — `refresh: scaffold language-aware audit section`
**Done.** New `## Language-aware audit` section in
`skills/refresh/SKILL.md` with per-language TBD stubs. Notes
that language-aware checks do not introduce new flags; they
ride on `--watch-v5` / `--dh-sequence` / etc.

### ✅ Commit 4 — `autopkgtest: add autodep8 Testsuite shortcut section`
**Done.** New `## Autodep8 shortcuts` section in
`skills/autopkgtest/SKILL.md` listing python / perl / ruby /
nodejs generators, with the explicit note that **no autodep8
generator exists for Rust or Go**. Process step 3 amended to
consider the shortcut first.

### ✅ Commit 5 — `python: language overlay, rules template, worker wiring`
**Done.** Files:
- `docs/references/languages/python.md` — interpreter/shebang
  policy, package naming (import name vs PyPI dist name), file
  layout, `debian/control` build-dep template, `debian/rules`
  pybuild pattern, watch v5 with `Template: pypi` (using
  `Dist:` field, downloads via pypi.debian.net), autopkgtest
  autodep8 nuances, wheels policy, Sphinx, refresh checks,
  iteration budget, bail-outs.
- `skills/bootstrap/templates/rules.python.tmpl` — `PYBUILD_NAME`
  export + `dh $@ --buildsystem=pybuild` + hardening gate.
- Worker SKILL.md edits: bootstrap Process step 4 anchor;
  `pybuild_name` scalar; refresh Python stub filled in;
  autopkgtest Python stub filled in.

### ✅ Commit 6 — `rust: language overlay, rules template, worker wiring`
**Done.** Files:
- `docs/references/languages/rust.md` — library-crate-vs-
  application-binary decision tree up front; debcargo path
  (bootstrap bail-out, debcargo.toml essentials, versioning,
  dependency-graph leaf-first ordering); dh-cargo path
  (build-deps, `rules`, `Cargo.lock`, watch v5 + `Template:
  github`); no autodep8; refresh checks; bail-outs.
- `skills/bootstrap/templates/rules.rust.tmpl` — minimal
  `dh $@ --buildsystem=cargo` with hardening flag.
- Worker SKILL.md edits: bootstrap Process step 4 anchor +
  library-crate bail-out gains a § link; refresh Rust stub
  filled in. (Autopkgtest unchanged — the "no autodep8 for
  Rust" line landed in commit 4.)

### ✅ Commit 7 — `golang: language overlay, rules template, worker wiring`
**Done.** Files:
- `docs/references/languages/golang.md` — library-vs-application
  shape decision; `dh-make-golang` as comparison-run-only;
  package naming (`golang-<importpath>-dev` for libs, command
  name for binaries); install paths under
  `/usr/share/gocode/src/`; `dh-sequence-golang` + `golang-any`
  Build-Depends; `XS-Go-Import-Path:` source-stanza field;
  `Built-Using: ${misc:Built-Using}` for application binaries;
  `DH_GOLANG_INSTALL_EXTRA` / `DH_GOLANG_EXCLUDES` overrides;
  `0.0~git<date>.<hash>-1` versioning for HEAD-only upstreams;
  watch v5 + `Template: github`; no autodep8; refresh checks;
  bail-outs. Has a "Notes for Golang package maintainer"
  section flagging open accuracy questions for review.
- `skills/bootstrap/templates/rules.golang.tmpl` — minimal
  `dh $@ --buildsystem=golang` (modern dh-sequence form, no
  `--with=golang`; plan deviation, documented in commit
  message context).
- Worker SKILL.md edits: bootstrap Process step 4 anchor;
  refresh Go stub filled in. (Autopkgtest unchanged — the "no
  autodep8 for Go" line landed in commit 4.)

### ✅ Commit 8 — `perl: drafted overlay, rules template, worker wiring`
**Done.** Files:
- `docs/references/languages/perl.md` — drafted from public
  Debian Perl Group knowledge with a prominent DRAFT
  blockquote at the top and a "DRAFT marker — likely needs
  correction" section at the bottom listing specific
  scrutiny points (test-dep selection, dh-sequence-perl
  existence, pkg-perl workflow, XS Build-Depends, hardening
  for XS). Covers library-vs-application split, naming
  (`libfoo-bar-perl`), file layout, plain `dh $@` rules with
  XS-hardening note, watch v5 + `Template: metacpan`,
  autodep8 `Testsuite: autopkgtest-pkg-perl`, refresh checks,
  bail-outs.
- `skills/bootstrap/templates/rules.perl.tmpl` — 4-line
  `dh $@` (no hardening gate; overlay explains XS modules
  need it added by hand).
- Worker SKILL.md edits: bootstrap Process step 4 anchor
  with explicit "DRAFT pending pkg-perl review" note;
  refresh + autopkgtest Perl stubs filled in (DRAFT caveat
  carried through).

### ✅ Commit 9 — `developer: document language-overlay recipe`
**Done.** New `## Adding a language overlay` section in
`docs/developer.md` inserted between "Adding a reference doc"
and "Quarterly review checklist". Covers: when to add an
overlay; the three concerns (doc + template + worker
anchors); 8-step recipe; the three house-style traps
(watch v5 only, no `dh_make`-style output shipped,
sbuild-first verification); DRAFT marker convention with
`perl.md` as the worked example; worked-examples table
listing all four shipped overlays.

### ✅ Commit 10 — `house-style: add language-overlay drift to quarterly checklist`
**Done.** New bullet in `docs/house-style.md` § "Notes for
future-me", listing per-language drift hot-spots: pybuild
major version + `dh-sequence-python3` behaviour (Python);
dh-cargo / debcargo workflow (Rust); dh-golang major version
+ `XS-Go-Import-Path:` conventions (Go); perl transitions +
pkg-perl conventions + DRAFT-marker status (Perl).

### ✅ Commit 11 — `version: bump to 0.2.0, surface language overlays in README`
**Done.** Files:
- `.claude-plugin/plugin.json` — version `0.1.0` → `0.2.0`.
- `README.md` — Status section reworded (mentions language
  overlays shipped in 0.2.0; references
  `plan/04-language-integration.md`); new
  `## Language coverage` section between Status and Layout
  with one bullet per language; Layout tree updated with
  `references/languages/` subdir.

## Status: complete

All eleven commits landed. The language-overlay pass is
shipped in `debaid` 0.2.0.

## Resume notes

- The original plan file at `~/.claude/plans/this-repository-contains-a-purring-clarke.md`
  has the longer-form version (audits of both repos, alternative
  designs considered, exhaustive verification section). This
  file is the operational artefact.
- Two factual corrections came up during commit 5 that future
  language overlays should respect:
  - Watch v5 `Template: pypi` uses `Dist:`, not `Source:`.
  - PyPI watch templates download via `pypi.debian.net` —
    there is no template for direct `pypi.org` downloads.
- The user prefers neutral-toned overlays (Rust overlay tones
  down Rust-team-process flavour; Perl overlay should do the
  same on pkg-perl conventions — mention, do not prescribe).
- Templates intentionally stay minimal — see
  `rules.python.tmpl` (10 lines) and `rules.rust.tmpl` (8
  lines). The overlay docs carry the depth; the templates are
  just enough to differentiate the dispatch.
- No upstream-attribution headers in any of the language
  overlays (clean-room redesign decision).

## Out of scope (deferred follow-ups)

- Promoting `tests/fixtures/*` from stubs (upstream tree +
  `expected/debian/` + `test.sh`) — a separate pass covering
  all fixtures together.
- Ruby overlay — deferred; the generator works via
  autopkgtest-pkg-ruby without an overlay, and Ruby is lower-
  priority than the four shipped here.
- `lintian` worker language overlays — deferred; most lintian
  tags are language-neutral, so the delta is smaller than for
  bootstrap/refresh.
- Node.js, Java, Haskell, OCaml overlays — architecture is now
  in place; each is a copy-paste of the developer.md recipe
  (commit 9) once it lands.
- Hardening audit (`blhc`), M-A coinstall verification,
  salsa-ci troubleshooting helper, Ubuntu-specific sub-skills.
