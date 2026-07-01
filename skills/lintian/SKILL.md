---
name: lintian
description: Resolve lintian -EvIL +pedantic output on a Debian source package by classifying each tag and producing the right fix — packaging change, DEP-3 quilt patch, or justified override with a comment. Bails to the maintainer after 3 attempts on the same tag. Never blanket-suppresses.
---

# debaid:lintian

Drive a package toward `lintian -EvIL +pedantic` clean, or
justified-overrides-only.

## Preconditions

- A context JSON exists at `./.debaid/context.json`. If missing,
  build it: run `${CLAUDE_PLUGIN_ROOT}/scripts/detect-source.sh`
  and `${CLAUDE_PLUGIN_ROOT}/scripts/tooling-probe.sh`, merge their
  outputs (see `${CLAUDE_PLUGIN_ROOT}/shared-context.md` for the
  full schema).
- `source.has_debian_dir == true`.
- `tooling.lintian.available == true`. If not, ask the maintainer
  to install lintian.

## Tag classification

For every tag lintian emits, classify it into exactly one bucket:

| Bucket | Use when | How to fix |
|---|---|---|
| **fix in packaging** | Tag is fixable by a change in `debian/` | Smallest patch that resolves the tag without side effects. |
| **fix upstream via patch** | Tag points to an upstream source issue | Author a DEP-3 patch via the `gbp pq` workflow: commit on `patch-queue/<branch>`, then `gbp pq export` materialises the `.patch` file under `debian/patches/` with `series` updated. Include a `Forwarded:` header (URL or `no` + reason). See `${CLAUDE_PLUGIN_ROOT}/docs/house-style.md` § "Patches". |
| **justified override** | Tag is a false positive, or the right answer is to suppress for this package | `lintian-overrides` file with a `# reason:` line directly above the override. |
| **won't fix** | Tag is real but maintainer accepts the cost | Report in summary; do NOT silently override. |

Default: prefer **fix in packaging** > **fix upstream** > **override**
> **won't fix**. Use override only when fix isn't possible or has
worse trade-offs.

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

Phase-specific (lintian):

- **Every override needs a `# reason:` comment immediately above
  it.** No exceptions. If you cannot write a clear reason, the
  tag is not eligible for an override — it's "won't fix" until
  the maintainer decides.
- **Never override a tag class wholesale.** No `*` wildcards in
  override files unless the maintainer explicitly approves.
- **Place overrides correctly.** Source-level tags go to
  `debian/source/lintian-overrides`. Binary-level tags go to
  `debian/$pkg.lintian-overrides`. Don't mix.
- **Never edit upstream source files directly to fix a tag.**
  Always a quilt patch.
- **Never bump the package version.** A lintian-fix run is not a
  release. The maintainer will `dch -i` if they want to record
  the change.

## Process

1. **Load context.**
2. **Run verify.** Call `${CLAUDE_PLUGIN_ROOT}/scripts/verify.sh`.
   This produces a JSON snapshot of build state and lintian tags
   bucketed by severity. If `build.ok == false` and lintian
   couldn't run, bail to the maintainer — the build failure is
   not your problem.
3. **Classify each tag.** Print the classification table to the
   maintainer.
4. **Resolve, tag by tag.** Process in order of severity (E → W → I
   → P). For each:
   - Apply the chosen fix.
   - Re-run verify (use `--no-build` if you have not changed
     anything that affects the build artefacts).
   - If the tag persists, count an attempt.
   - After `budget.max_attempts_per_error_class` attempts on the
     same tag, bail out for that tag with a structured summary
     and move on to the next.
5. **Final pass.** Run a full verify (no `--no-build`). Confirm
   only justified overrides remain. Report.

For the verify-script output schema, see
`${CLAUDE_PLUGIN_ROOT}/shared-context.md` § "Verify-script output
schema (v1)".

## Override file format

```
# reason: <one-line justification, ideally citing policy/devref/DEP>
<package-or-source>: <tag> [<extra context>]
```

Example:
```
# reason: upstream ships a pre-built README.html for offline use;
# regenerating it would require a Sphinx Build-Depends just to
# delete the file, which is worse than the tag.
mypkg: privacy-breach-generic [usr/share/doc/mypkg/README.html]
```

## Bail-out conditions

- Same tag persists after `max_attempts_per_error_class` fix
  attempts.
- Tag classification is ambiguous (could be packaging fix OR
  upstream patch and the right answer needs domain knowledge).
- Fix introduces a NEW lintian error.
- Build fails as a precondition for running lintian.

Use the bail-out summary format from
`${CLAUDE_PLUGIN_ROOT}/shared-context.md` § "Bail-out summary
format". Include:
- The unresolved tag and lintian's `--info` text for it (read it
  from `verify.sh`'s `lintian.log_path`).
- Each fix attempt and why it didn't work.
- Concrete proposed options for the maintainer.
