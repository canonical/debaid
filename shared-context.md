# Shared context contract

This file is the developer-facing spec that all debaid skills
implement. It documents the runtime context schema, the verify
script's output, the iteration-budget envelope, and the bail-out
format.

Worker `SKILL.md` files inline the parts they need to attend to at
every invocation (hard rules, the short reactive templates). They
reference this file by absolute path
(`${CLAUDE_PLUGIN_ROOT}/shared-context.md`) for the longer
schemas, which they `Read` only when consuming them.

If you change a hard rule or the bail-out format here, update all
five worker `SKILL.md` files to match — see `docs/developer.md`
for the contributor checklist.

## How context is passed

The orchestrator (or a directly-invoked worker) writes a single
JSON document to `${DEBAID_CONTEXT}` if that env var is set,
else to `./.debaid/context.json` relative to the source tree.

Workers MUST:

1. Check `${DEBAID_CONTEXT}` first.
2. Fall back to `./.debaid/context.json`.
3. If neither exists, run
   `${CLAUDE_PLUGIN_ROOT}/scripts/detect-source.sh` and
   `${CLAUDE_PLUGIN_ROOT}/scripts/tooling-probe.sh` themselves,
   then merge their outputs.

## JSON schema (v1)

```json
{
  "schema_version": 1,
  "generated_at": "ISO 8601 UTC timestamp",
  "source": {
    "path": "absolute path to source tree",
    "language": "go|rust|python|c|cpp|java|nodejs|perl|ruby|haskell|unknown",
    "build_system": "autotools|meson|cmake|cargo|go-mod|setuptools|pyproject|nodejs|make|unknown",
    "has_debian_dir": true,
    "has_quilt_patches": false,
    "debian_branch_layout": "monorepo|dep14|separate-branch|none|unknown",
    "upstream_vcs": "git|hg|svn|tarball|none|unknown",
    "ubuntu_delta": null
  },
  "tooling": {
    "sbuild":             {"available": true,  "version": "x.y"},
    "pbuilder":           {"available": false, "version": null},
    "autopkgtest":        {"available": true,  "version": "x.y"},
    "lintian":            {"available": true,  "version": "x.y"},
    "debputy":            {"available": false, "version": null},
    "wrap-and-sort":      {"available": true,  "version": "x.y"},
    "gbp":                {"available": true,  "version": "x.y"},
    "dh_make":            {"available": true,  "version": "x.y"},
    "cme":                {"available": false, "version": null},
    "git-ubuntu":         {"available": false, "version": null},
    "requestsync":        {"available": false, "version": null},
    "pull-debian-source": {"available": false, "version": null},
    "pull-lp-source":     {"available": false, "version": null},
    "syncpackage":        {"available": false, "version": null},
    "update-maintainer":  {"available": false, "version": null},
    "mk-sbuild":          {"available": false, "version": null}
  },
  "target": {
    "distro":       "debian|ubuntu",
    "release":      "unstable|trixie|forky|stonking|resolute|noble|...|unknown",
    "pocket":       "dev|proposed|updates|security|backports",
    "freeze_state": "none|debian-import-freeze|feature-freeze|final-freeze|unknown",
    "host_arch":    "amd64|arm64|..."
  },
  "user": {
    "debfullname": "value of DEBFULLNAME or git config user.name",
    "debemail":    "value of DEBEMAIL or git config user.email"
  },
  "budget": {
    "max_attempts_per_error_class": 3,
    "diff_threshold_lines": 200,
    "repeat_budget": 2
  },
  "reference_corpus": "absolute path to reference debian/ trees, or null",
  "house_style": "absolute path to house-style.md (orchestrator passes the active one)"
}
```

## Field semantics — Ubuntu-specific

- **`source.ubuntu_delta`** — `true` if `debian/changelog` shows
  any non-`buildN` Ubuntu revision (e.g. `2.0-2ubuntu1`); `false`
  if the package is in Ubuntu but carries no delta from Debian
  (eligible for sync); `null` when `target.distro != ubuntu`.
  See `docs/references/ubuntu-merges-syncs.md`.
- **`target.pocket`** — `dev` for the in-development series
  (Debian unstable, current Ubuntu devel). `proposed`/`updates`/
  `security`/`backports` apply to released Ubuntu series and
  pull in the SRU workflow (see `docs/references/sru.md`).
  Default: `dev`.
- **`target.freeze_state`** — Ubuntu release-cycle gate currently
  in effect, or `none`. The orchestrator uses this to add a
  freeze warning to the confirmation gate when running against a
  frozen dev release. The detect step MAY leave this `unknown`
  if it cannot determine it cheaply; workers MUST then treat
  cautious behaviour as default.

## Verify-script output schema (v1)

`${CLAUDE_PLUGIN_ROOT}/scripts/verify.sh` is the iteration-loop primitive
workers consult between fix attempts. It runs a build (sbuild or
dpkg-buildpackage) and lintian, then emits a single JSON snapshot.
The script is **stateless**: workers hold previous snapshots in
context and compute progress (e.g. "same tag fired last attempt")
themselves.

Exit code is 0 on a successful snapshot regardless of build/lint
pass-or-fail. Non-zero only on input errors (missing `debian/`,
missing `jq`, bad args).

```json
{
  "build": {
    "tool":      "sbuild|dpkg-buildpackage|none",
    "ran":       true,
    "ok":        true,
    "log_path":  "/tmp/verify-build.XXXXXX.log",
    "exit_code": 0
  },
  "lintian": {
    "ran":               true,
    "scope":             "changes|dsc|source-tree|none",
    "log_path":          "/tmp/verify-lintian.XXXXXX.log",
    "errors":            ["tag-name", "..."],
    "warnings":          ["tag-name", "..."],
    "infos":             ["tag-name", "..."],
    "pedantics":         ["tag-name", "..."],
    "overrides_applied": 0
  },
  "diff_size_lines": 0
}
```

Field notes for workers:

- **`build.ran == false`** means no builder executed (either
  `--no-build` was passed or no builder was available). `build.ok`
  is meaningful only when `ran == true`.
- **`lintian.scope`** records what artifact lintian inspected.
  `changes` is authoritative; `dsc` covers source-level tags only;
  **`source-tree` is a degraded scope** — binary-only checks
  (file-permissions, shipped files, etc.) do not fire. Workers
  MUST weight a clean result less when `scope == "source-tree"`.
- **Tag arrays** may contain duplicates when a tag fires on more
  than one file. The repetition is signal — workers MAY use it to
  prioritise fixes that resolve many instances at once.
- **`overrides_applied`** counts `N: Overridden:` lines in the
  lintian log — it tells the worker how many tags are already
  suppressed by existing overrides, so it doesn't double-override.
- **`diff_size_lines`** counts changed lines under `debian/` only
  (relative to the git index), so upstream churn doesn't pollute
  the budget check. `null` if the source tree is not a git repo.
- **Log paths** point at `/tmp` files that persist for the
  session. Workers MAY `Read` them for context (e.g. lintian
  `--info` text on a specific tag) but MUST NOT mutate them.

## Iteration-budget envelope

All workers that mutate the source tree MUST honour:

- **`max_attempts_per_error_class`** — at most N attempts to resolve
  any single class of error (lintian tag, sbuild stage failure,
  autopkgtest test name). After N attempts targeting the same error
  class, bail to the maintainer with a structured summary.
- **`repeat_budget`** — if the same exact error reappears after a
  fix attempt N=`repeat_budget` times in a row, bail immediately
  regardless of `max_attempts_per_error_class`.
- **`diff_threshold_lines`** — before producing a diff larger than
  this, the worker MUST surface a summary to the maintainer and ask
  for confirmation to proceed. Refresh worker is especially subject
  to this.

## Reference-corpus contract

- If `reference_corpus` is set, workers MAY consult `${corpus}/<language>/`
  for exemplar `debian/` trees. Consultation is read-only.
- The default corpus is `tests/fixtures/` of this repo.
- Override with `--reference=<path>` on the orchestrator or worker;
  disable with `--reference=none` (sets `reference_corpus: null`).
- Workers MUST NOT copy corpus files verbatim — corpus exemplars
  are reference points for idiom, not templates. Generated files
  go through house-style rendering.

## Bail-out summary format

When a worker bails to the maintainer, it MUST produce a message
with these sections:

```
## What I was trying to do
<one paragraph>

## What I tried
- attempt 1: <change> → <result>
- attempt 2: <change> → <result>
- attempt 3: <change> → <result>

## Current state
- build: <pass|fail with stage>
- lintian: <N E, M W, K I>
- diff size: <N lines>

## Where I'm stuck
<concrete error, no hedging>

## Proposed options
1. <option> — <consequence>
2. <option> — <consequence>
3. Stop and let me investigate manually

Which would you like?
```

## What workers MUST NOT do

- Invoke `dput`, `debrelease`, `dgit push`, or any upload command.
- Run `git push` to any remote.
- Edit `debian/changelog` distribution from `UNRELEASED` to anything else.
- Edit `Maintainer:` or `Uploaders:` fields.
- Edit upstream source files (use `debian/patches/` with DEP-3 headers).
- Run `rm -rf` or `git clean -fdx` on the workspace.
- Write a lintian override without a `# reason` comment.
- Set `Multi-Arch: same` without verifying file paths.
