# Plan: `debaid` — Debian packaging skills for LLMs

## Context

`debaid` will provide Claude Code skills that automate common Debian
packaging maintenance tasks: bootstrapping a new package, refreshing an
existing `debian/` directory to current practice, adding autopkgtest
coverage, and resolving lintian issues. Repo is greenfield — only a
`workshop.yaml` (Canonical workshop runner pinning `claude-code` on
Ubuntu 24.04) and an empty `skills/` exist today.

The intended user is a Debian/Ubuntu package maintainer who wants an
LLM to handle the mechanical parts of packaging while keeping the
maintainer in the loop for judgement calls. Output must be
Debian-archive-quality: not a rough draft, but `lintian`-clean,
`sbuild`-clean, autopkgtest-passing packaging that a DD would sign and
upload.

Design decisions locked in with the user:
- **Architecture**: single orchestrator skill + multiple worker skills,
  selected via `--only` / `--skip` flags.
- **Distro scope**: Debian-first, Ubuntu specifics as an overlay.
- **Authority for "good packaging"**: Debian Policy + DEP-* + devref,
  plus a DD-judgement house-style file, plus tooling output (lintian,
  debputy, blhc, hardening-check) as an *indicator* — not blindly
  obeyed. Reference-corpus is **hybrid**: a small set of vetted
  exemplar packages ships under `tests/fixtures/` and is consulted
  by default; a maintainer can override the path or disable.
- **Knowledge base**: short curated docs under `docs/references/`
  cover Debian-ecosystem practices (DEP summaries, build-tool tradeoffs,
  VCS workflows, release process). Workers cite them in bail-out
  summaries; maintainers read them directly.
- **Verification**: mandatory build+lint loop, but bail out to the
  human after a bounded number of iterations or when the diff exceeds
  a size threshold.

## Architecture

```
skills/
├── debaid/                  # Orchestrator skill (user entry point)
│   ├── SKILL.md               # accepts --only/--skip, dispatches workers
│   ├── house-style.md         # DD-judgement preferences (versioned, dated)
│   ├── shared-context.md      # how workers load common context
│   └── scripts/
│       ├── detect-source.sh   # build system, language, existing d/ state
│       └── tooling-probe.sh   # sbuild? debputy? autopkgtest? versions?
├── debaid-bootstrap/        # New package from scratch
├── debaid-refresh/          # Modernise existing debian/
├── debaid-lintian/          # Fix tags + write justified overrides
└── debaid-autopkgtest/      # Add or improve d/tests/
```

Each worker skill is **independently invocable** (so a maintainer can
call `debaid-lintian` directly without the orchestrator) but also
**designed to be chained** by the orchestrator, which pre-populates a
shared context file that workers consult to avoid redundant probing.

## Step-by-step plan

### Step 1 — Shared infrastructure

Build before any worker exists. Defines what every worker can assume.

- `skills/debaid/house-style.md`: the prescriptive DD-judgement file.
  Dated, versioned, reviewed quarterly. Contents (initial draft —
  needs your sign-off):
  - `debhelper-compat (= 13)` default; 14 only when explicitly opted in
  - `Standards-Version`: latest known good (pin a value, don't auto-bump)
  - `Rules-Requires-Root: no` always
  - Source format `3.0 (quilt)` for native-with-upstream; `3.0 (native)`
    only for genuinely native packages
  - Prefer `dh-sequence-*` virtual Build-Depends over `--with` in
    `debian/rules`
  - `debian/rules` minimal: `#!/usr/bin/make -f` + `%: ; dh $@`, only
    override targets when necessary, never preemptively
  - `wrap-and-sort -ast` on `debian/control`, `debian/copyright`,
    `debian/*.install`
  - `debian/watch` v4, `pgpmode=auto` only with
    `debian/upstream/signing-key.asc` present
  - `debian/copyright` in machine-readable DEP-5 format always
  - `debian/changelog`: `UNRELEASED` distribution until human
    explicitly approves a release
  - Salsa-CI enabled (`debian/salsa-ci.yml`) for new packages
  - DEP-14 branch naming for Vcs-Git

- `skills/debaid/scripts/detect-source.sh`: reports as JSON
  `{language, build_system, has_debian_dir, has_quilt_patches,
   debian_branch_layout, upstream_vcs}`.

- `skills/debaid/scripts/tooling-probe.sh`: reports versions and
  availability of `sbuild`, `pbuilder`, `autopkgtest`, `lintian`,
  `debputy`, `wrap-and-sort`, `gbp`, `git-buildpackage`,
  `dh_make`, `cme`.

- `skills/debaid/shared-context.md`: documents the JSON contract
  workers consume; defines the iteration-budget envelope (default: 3
  retries per error class, 200-line diff threshold before requiring
  developer confirmation); documents the reference-corpus contract
  (default path `tests/fixtures/`, override via `--reference=<path>`,
  disable via `--reference=none`).

- `docs/references/` — short, opinionated knowledge docs the workers
  and maintainers both consult. Each is a single page, links out to
  the canonical source. Initial set:
  - `build-tools.md` — sbuild vs pbuilder vs gbp buildpackage vs
    dpkg-buildpackage; when each is appropriate.
  - `deps.md` — DEP-3 (patch headers), DEP-5 (copyright format),
    DEP-8 (autopkgtest), DEP-12 (upstream metadata), DEP-14
    (Git layout). One paragraph + link each.
  - `vcs-workflows.md` — DEP-14 branch naming, `gbp import-orig`,
    `dgit`, pristine-tar, signed tags.
  - `tooling.md` — `debputy`, `wrap-and-sort`, `cme`, `devscripts`
    highlights (`uscan`, `licensecheck`, `dch`, `dget`), `blhc`,
    `hardening-check`.
  - `release-process.md` — UNRELEASED → unstable, NMU etiquette,
    RC bug handling, freeze policies.
  - `salsa-ci.md` — what the standard pipeline checks, common
    failures and their meaning.

### Step 2 — `debaid-bootstrap`

Create `debian/` for an unpackaged upstream source tree.

Inputs: source tree path, package name, upstream version (inferred or
asked), section/priority hints.

Output: complete, builds-once `debian/` dir, distribution=UNRELEASED,
ready for the human to review and fill in `Description` long-text.

Path:
1. Run `detect-source` and `tooling-probe`.
2. Generate `control`, `copyright`, `rules`, `changelog`, `watch`,
   `source/format`, `source/lintian-overrides` (empty), `salsa-ci.yml`
   per house style.
3. Use `dh_make --native=no --copyright=...` only as a scaffold to
   compare against, **never** ship its output unchanged — it is
   noisy and out of date.
4. Run build+lint loop (Step 6).

### Step 3 — `debaid-refresh`

Modernise an existing `debian/` to match the house style.

**Most dangerous worker** — mutates work the maintainer made
deliberate choices about. Hard rules:
- Default to **dry-run** (produce a diff, do not write).
- Never refactor `override_dh_*` targets without showing the maintainer.
- Never change `Maintainer:` / `Uploaders:`.
- Never bump `debian/changelog` (refresh ≠ release).
- Each modernisation is a separate, labelled hunk in the diff with a
  one-line justification linked to policy/devref/DD-style.

Scope of refresh (checklist, opt-in per item via flags):
- compat bump
- Standards-Version bump
- R³ / `Rules-Requires-Root`
- `dh-sequence-*` migration
- `wrap-and-sort` pass
- watch v4 upgrade
- DEP-5 copyright normalisation
- M-A annotations audit (advisory only — never auto-set `M-A: same`
  without verifying file collisions)
- Salsa-CI introduction

### Step 4 — `debaid-lintian`

Resolve `lintian -EvIL +pedantic` output.

Workflow:
1. Run lintian on the existing build or on source.
2. Classify each tag: *fix in packaging*, *fix upstream via patch*,
   *justified override*, *won't fix*.
3. For fixes: produce the smallest patch that addresses the tag without
   side effects.
4. For overrides: write to `debian/source/lintian-overrides` or
   `debian/$pkg.lintian-overrides`, **always with a comment** giving
   the reason. Never blanket-suppress.
5. For upstream issues: write a DEP-3 quilt patch under
   `debian/patches/`, add to `series`.

Iteration bail-out: if the same tag persists after 3 fix attempts, stop
and ask the maintainer with a structured summary of what was tried.

### Step 5 — `debaid-autopkgtest`

Add or improve `debian/tests/`.

- Detect language / framework; propose a `Test-Command:` or a
  per-test script under `debian/tests/`.
- Default `Restrictions:` to the minimum that passes (avoid
  `needs-root`, `isolation-container` unless required and justified).
- For library packages, propose ABI smoke tests; for daemons, propose
  service-start tests using `Restrictions: isolation-container,
  needs-root` only with explicit maintainer approval.
- Verify with `autopkgtest -- null` first (cheapest), then
  `autopkgtest-virt-qemu` or `autopkgtest-virt-lxc` if available.

### Step 6 — Verification loop

Shared by all workers that produce a build.

```
attempt = 0
while attempt < BUDGET:
  build = sbuild --no-arch-all? on dsc
  lint  = lintian -EvIL +pedantic
  if build.ok and lint.clean_or_only_justified_overrides:
    return success
  if same_error_class_as_previous(build, lint):
    repeated += 1
  if repeated >= REPEAT_BUDGET (default 2):
    bail_to_human(structured_summary)
  if diff_size > DIFF_THRESHOLD (default 200 lines):
    confirm_with_human()
  attempt += 1
bail_to_human(structured_summary)
```

Structured bail-out summary must include: what was tried, what
failed, current lintian / sbuild output, the proposed next step, and
a question for the maintainer with concrete options.

### Step 7 — Orchestrator `debaid`

Accepts `--only=<phases>` / `--skip=<phases>`; phases are the worker
names (`bootstrap`, `refresh`, `lintian`, `autopkgtest`). Examples:

- `/debaid` — full pipeline (detect → refresh-or-bootstrap →
  lintian → autopkgtest), each phase gated by a confirmation.
- `/debaid --only=lintian` — just lintian.
- `/debaid --only=refresh,lintian --skip=autopkgtest` —
  modernisation pass without test work.

Loads shared context once, passes it to each worker as a path argument,
serialises phase outputs into a single conversation transcript so the
maintainer sees the whole story.

### Step 8 (bonus) — Natural-language layer

Thin wrapper that translates intent ("just fix lintian", "modernise
this but skip the build tests") into the appropriate
`--only`/`--skip` invocation of Step 7. Implemented as orchestrator
prompt logic, not a separate skill. Ship after Steps 1–7 are stable.

## Key points to remember

- **House style is policy, not vibes.** Every prescriptive choice in
  `house-style.md` cites the rule (Policy section, devref chapter,
  DEP number) or says "DD-judgement" explicitly. Maintainers will
  push back on opaque preferences.
- **Never upload.** No worker invokes `dput`, `debrelease`, `dgit
  push`, or `git push` to anywhere. `distribution=UNRELEASED` until
  the human edits the changelog.
- **Never touch upstream sources directly.** All upstream
  modifications go through `debian/patches/` with DEP-3 headers.
- **DEP-14 awareness.** Detect `debian/latest`, `debian/sid` branch
  layouts; don't assume `master` is the packaging branch.
- **Bail-out is a feature, not a failure.** A structured "I tried X, Y,
  Z and got stuck on $TAG — here are three options" message is the
  desired terminal state when iteration stalls.
- **Iteration budget is per-error-class**, not per-attempt. Three
  attempts on three different problems is fine; three attempts on the
  same lintian tag is the bail-out trigger.

## Caveats to avoid

- **Don't trust `dh_make` output.** Use only as a structural reference;
  generate `debian/` from house-style templates instead.
- **`Multi-Arch: same` is a footgun.** Setting it without verifying
  file paths differ across architectures breaks coinstallation.
  Refresh worker proposes, never sets, M-A annotations.
- **`pgpmode=auto` requires a signing key.** Don't enable in watch
  files without `debian/upstream/signing-key.asc`.
- **Lintian overrides without comments are worse than the original
  tag.** Always justify; the override syntax with `# comment` lines
  matters.
- **Refresh on someone else's package is socially fraught.** Default
  to dry-run + per-hunk justification; never make a refresh PR
  unsolicited.
- **Salsa-CI yaml format drifts.** Pin to a tested template version in
  `house-style.md`; check Salsa docs quarterly.
- **`Rules-Requires-Root: no` interacts with maintainer scripts.** If
  the package installs setuid/setgid files, the worker must detect
  and warn (or set `R³: binary-targets`).
- **`debian/copyright` DEP-5 is easy to write, hard to write
  correctly.** For vendored deps / large source trees, default to
  asking the maintainer rather than guessing.
- **Workshop runs with `--dangerously-skip-permissions`.** All
  workers must be defensive about destructive ops on the workspace
  (no `rm -rf`, no `git clean -fdx` without confirmation, no
  `dpkg-source -x` over an existing tree).
- **Standards-Version, compat, debhelper add-on names drift.** The
  house-style file must be dated and reviewed; assume entries go
  stale within ~6 months.

## Repo layout (target)

```
debaid/
├── workshop.yaml             # already exists
├── README.md                 # ship-time
├── skills/
│   ├── debaid/
│   ├── debaid-bootstrap/
│   ├── debaid-refresh/
│   ├── debaid-lintian/
│   └── debaid-autopkgtest/
├── tests/
│   └── fixtures/             # small reference upstream trees to exercise workers
│       ├── hello-c-autotools/
│       ├── hello-go/
│       ├── hello-rust/
│       └── hello-python/
└── docs/
    ├── house-style.md        # canonical copy (symlinked into skill)
    ├── developer.md          # how to add a new worker
    └── references/           # short knowledge docs (build-tools, DEPs, vcs, ...)
        ├── build-tools.md
        ├── deps.md
        ├── vcs-workflows.md
        ├── tooling.md
        ├── release-process.md
        └── salsa-ci.md
```

## Verification (how we test debaid itself)

End-to-end fixtures under `tests/fixtures/`:
1. `hello-c-autotools/` — exercise bootstrap from a clean autotools
   project. Success = `sbuild`-clean, `lintian`-clean.
2. `hello-go/` — exercise bootstrap with `dh-sequence-golang`.
3. An intentionally-broken `debian/` dir under `tests/fixtures/
   stale-debian/` — exercise refresh and verify the diff matches
   golden output.
4. A package with known lintian tags — exercise the lintian worker
   and verify the right tags get justified overrides vs fixes.
5. Drive each test with `claude --bare --print` from the workshop
   action and diff against expected output (or score by
   lintian/sbuild exit status).

Bail-out behaviour testable by feeding a deliberately unsolvable
package (e.g. broken upstream build) and asserting the worker exits
with a structured question rather than looping forever.

---

## Patches to v1 (post-yolo review)

Issues found by the maintainer after the first version landed. Each
patch is small and self-contained.

### P1 — Ship `tests/verify.sh`

The v1 Verification section described how fixtures should be driven
but no runner was written. Add `tests/verify.sh` so the fixture
framework is usable now and self-extending later.

**File**: `tests/verify.sh` (new)

**Behaviour**:
- Walks immediate subdirectories of `tests/fixtures/`.
- For each fixture `F`:
  - If `F/expected/` exists AND `F/test.sh` exists → execute
    `F/test.sh`; capture exit + stdout/stderr; print `[PASS]` /
    `[FAIL]` line.
  - Otherwise → print `[STUB] F: no expected/ + test.sh yet,
    skipping`.
- Aggregate: exit non-zero only if any non-stub fixture fails.
- Pass `--strict` to also fail when any stub remains (gate for
  release).
- Honour `$DEBAID_CLAUDE_CMD` (default: `claude --bare --print`)
  so the workshop and a plain shell can both drive it.
- POSIX-ish bash, `set -euo pipefail`, no external deps beyond
  `diff`, `find`, and the `claude` CLI.

**Tests/fixtures README update**: cross-link `verify.sh` in
`tests/fixtures/README.md` so contributors know the entry point.

### P2 — Replace home-path in README

**File**: `README.md`, lines 17–19.

Replace:
```
First version. Skills are scaffolded; fixtures are stubs. See
`/home/peb/.claude/plans/you-are-a-specialized-refactored-pascal.md`
for the plan this implements.
```

With:
```
First version. Skills are scaffolded; fixtures are stubs. See
`plan/draft.md` for the plan this implements.
```

Rationale: absolute home path leaks the author's filesystem and is
meaningless to anyone else. `plan/draft.md` is the in-repo
artifact (Ultraplan's refined version) and survives clone.

### P3 — Safe JSON emission in both helper scripts

`$ROOT` (detect-source.sh) and `$v` (tooling-probe.sh) are
interpolated raw into JSON. Paths with quotes/backslashes/newlines,
or odd `--version` output, will produce invalid JSON.

**Files**:
- `skills/debaid/scripts/detect-source.sh`
- `skills/debaid/scripts/tooling-probe.sh`

**Approach**: build JSON with `jq -n --arg ... --argjson ...`.
`jq` is a hard dep — acceptable because every Debian packaging
host has it (it's in `devscripts` Recommends and used by salsa-ci).

**detect-source.sh emit section** becomes:
```bash
jq -n \
  --arg path "$ROOT" \
  --arg language "$language" \
  --arg build_system "$build_system" \
  --argjson has_debian_dir "$has_debian_dir" \
  --argjson has_quilt_patches "$has_quilt_patches" \
  --arg debian_branch_layout "$debian_branch_layout" \
  --arg upstream_vcs "$upstream_vcs" \
  '{path: $path,
    language: $language,
    build_system: $build_system,
    has_debian_dir: $has_debian_dir,
    has_quilt_patches: $has_quilt_patches,
    debian_branch_layout: $debian_branch_layout,
    upstream_vcs: $upstream_vcs}'
```

**tooling-probe.sh `probe()`** becomes:
```bash
probe() {
  local cmd="$1"
  local version_flag="${2:---version}"
  if command -v "$cmd" >/dev/null 2>&1; then
    local v
    v="$("$cmd" "$version_flag" 2>&1 | head -n1 \
         | sed -E 's/[^0-9.]*([0-9]+(\.[0-9]+){1,3}).*/\1/' || true)"
    if [[ -z "$v" || "$v" == *" "* ]]; then v="unknown"; fi
    jq -nc --arg v "$v" '{available: true, version: $v}'
  else
    jq -nc '{available: false, version: null}'
  fi
}
```

The outer `cat <<EOF` in tooling-probe.sh stays but the inner
calls now emit valid JSON objects unconditionally.

Add a `command -v jq` precheck at the top of both scripts that
errors clearly if jq is missing.

### P4 — Drop `**` patterns; use `find`

`**/*.c` requires `shopt -s globstar`, which the script doesn't
enable. Currently `**` collapses to `*` silently and the recursive
intent is lost.

**File**: `skills/debaid/scripts/detect-source.sh`, lines 52–61.

Replace the `have_glob "**/*.c"` (etc.) calls with a `find`-based
helper:

```bash
has_files() {
  # $1 = name pattern, search root-relative, depth-limited for speed
  find "$ROOT" -maxdepth 3 -type f -name "$1" -print -quit \
    2>/dev/null | grep -q .
}
```

Then the language-refinement block becomes:
```bash
if [[ "$language" == "unknown" ]]; then
  case "$build_system" in
    autotools|cmake|meson|make)
      if   has_files '*.c';          then language="c"
      elif has_files '*.cpp';        then language="cpp"
      elif has_files '*.cc';         then language="cpp"
      fi
      ;;
  esac
fi
```

Rationale: `find` is predictable, doesn't interact with `nullglob`
/ `globstar` / `failglob`, and `-maxdepth 3` keeps it fast on big
trees.

Keep `have_glob` for non-recursive single-directory checks (e.g.
`*.cabal`) — those work correctly today.

### P5 — Robust tarball detection

`[[ -f "$ROOT"/*.tar.gz ]]` fails with `set -e` when:
- Multiple tarballs match (`[[ -f a b c ]]` is a syntax error).
- The glob expands to nothing AND `failglob`/`nullglob` is on.

**File**: `skills/debaid/scripts/detect-source.sh`, line 94.

Replace:
```bash
elif [[ -f "$ROOT"/*.tar.gz ]] 2>/dev/null || [[ -f "$ROOT"/*.tar.xz ]] 2>/dev/null; then
  upstream_vcs="tarball"
```

With:
```bash
elif find "$ROOT" -maxdepth 1 -type f \
       \( -name '*.tar.gz' -o -name '*.tar.xz' -o -name '*.tar.bz2' -o -name '*.tar.zst' \) \
       -print -quit 2>/dev/null | grep -q .; then
  upstream_vcs="tarball"
```

Also covers `.tar.bz2` and `.tar.zst` (modern upstream releases).

---

## Verification (patches)

After applying all five patches:

1. `bash -n skills/debaid/scripts/detect-source.sh` — syntax-check.
2. `./skills/debaid/scripts/detect-source.sh` from this repo
   root — should still emit valid JSON; pipe through `jq .` to
   verify.
3. `./skills/debaid/scripts/detect-source.sh /tmp/path\ with\ "quotes"`
   (after creating such a path) — JSON should remain valid.
4. Drop a stray `foo.tar.gz` AND `bar.tar.gz` next to a non-git
   tree; rerun detect-source — should report `upstream_vcs:
   tarball` without erroring.
5. `./tests/verify.sh` — should run; every fixture currently
   prints `[STUB]`; exit code 0. With `--strict`, exit code 1.
6. `grep -n /home/peb README.md` — no matches.

## Files touched by patches

- `tests/verify.sh` (new, executable) — see P6 correction
- `tests/fixtures/README.md` (link to verify.sh)
- `README.md` (P2 line swap)
- `skills/debaid/scripts/detect-source.sh` (P3 + P4 + P5)
- `skills/debaid/scripts/tooling-probe.sh` (P3)

### P6 — Misplaced verify.sh (correction)

P1 shipped a fixture-test runner under `tests/verify.sh`. That was
wrong: the plan's Step 6 verify.sh is the **build+lintian snapshot
tool that workers call between iteration attempts**, not a
fixture-test driver. The two scripts have nothing in common
beyond their name.

**Corrections:**

- Move `tests/verify.sh` → `tests/run-fixtures.sh` (its real
  purpose). Update the `tests/fixtures/README.md` link.
- Add the actual `skills/debaid/scripts/verify.sh`. Shape:
  - Inputs: `[--builder=sbuild|dpkg-buildpackage]
            [--no-build] [PATH]`.
  - Runs the configured build (auto-pick sbuild → dpkg-buildpackage)
    and lintian.
  - Parses lintian output by severity into JSON arrays of tag
    names; preserves log paths for the worker to grep.
  - Includes diff-size-in-lines (vs. HEAD on `debian/`) for the
    worker to compare against `budget.diff_threshold_lines`.
  - Exit 0 on snapshot success regardless of build/lint pass;
    non-zero only on input errors. The worker LLM owns the
    decision logic.
  - `jq` dep (consistent with P3).
- The iteration loop and bail-out semantics from
  `shared-context.md` stay where they are — they are worker-level
  policy, not script behaviour.

**Files touched (P6):**

- `skills/debaid/scripts/verify.sh` (new, executable)
- `tests/verify.sh` → `tests/run-fixtures.sh` (rename)
- `tests/fixtures/README.md` (rename + add disambiguation note)

