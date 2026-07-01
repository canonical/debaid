# Review: `docs/house-style.md` changes

## Context

User made four updates to the prescriptive packaging house-style
file (`docs/house-style.md`) on top of the plugin restructure they
just shipped. The file is consulted by every debaid skill on
every run via `${CLAUDE_PLUGIN_ROOT}/docs/house-style.md`; changes
here propagate into generated `debian/` directories. Reviewer's
job is to flag correctness issues, internal inconsistencies, and
follow-up edits needed to keep templates / workers in sync with
the new house-style baseline.

## The four changes

### 1. debhelper-compat: 13 → 14

`docs/house-style.md:33-36`

Inverts the default. Was: 13 default, 14 opt-in. Now: 14 default,
13 only when the package "has at least one setuid binary that
depend on `Rules-Requires-Root` interactions."

**Findings:**

- **Verify externally**: is compat 14 the recommended baseline in
  Debian as of 2026-05? If yes, the inversion is fine. If 14 is
  still adoption-in-progress, 13 may still be the safer default.
  I cannot verify the current Debian recommendation from inside
  this session.
- **The rationale text is not quite right.** Compat 14's
  differences from 13 (e.g. new dh helpers, install destinations,
  stripping behavior) aren't specifically about setuid binaries
  or `Rules-Requires-Root` interactions — those are governed by
  the `Rules-Requires-Root:` field independently. The "fall back
  to 13 if setuid" caveat was carried over from the old text's
  framing of why 13 was the default; under the new framing it
  doesn't match a real compat-version-specific behavior.
  Recommend either dropping the caveat or replacing it with the
  actual compat-14-vs-13 incompatibility (whichever the user has
  in mind).
- **Grammar**: "binary that depend on" → "binaries that depend
  on" (or "a binary that depends on"). Small.

### 2. Standards-Version: 4.7.1 → 4.7.4

`docs/house-style.md:51`

**Findings:**

- **Verify externally**: is Policy 4.7.4 actually published? 4.7.x
  was the active series at the start-of-2026 cutoff; 4.7.4 by
  2026-05 is plausible but I can't confirm from this session.
  Wrong-numbered Policy versions trip lintian's
  `out-of-date-standards-version` check.
- House-style's "Next review due: 2026-08-13" header is the right
  pattern for catching this kind of drift; no action needed on
  the header.

### 3. debian/watch: v4 → v5 (and `pgpmode` → `Pgp-Mode`)

`docs/house-style.md:99-103`

This is the most substantive change.

**Findings:**

- **Verify externally**: as of my knowledge cutoff (Jan 2026),
  watch v5 was still in active discussion / draft, with
  `uscan` (devscripts) tooling support landing but not yet the
  recommended default for new packaging. By 2026-05 status may
  have changed; the user should verify against current devscripts
  documentation and Debian wiki guidance.
- **Field-name change is correct *for v5*.** v5 uses
  RFC822-style fields (`Pgp-Mode: auto`); v4 uses inline opts
  (`opts=pgpmode=auto`). If staying on v5, the case-change is
  right.
- **Risk if premature**: salsa-CI runners using older `uscan`
  versions, plus `qa.debian.org`'s watch-checking infrastructure,
  may not yet handle v5. A bootstrap worker emitting v5 against
  a package targeting older salsa-CI images could ship watch
  files that fail in CI.
- **Conservative alternative**: keep v4 as the prescribed
  default, add a forward-looking note that v5 will be the default
  once tooling-side support settles. That avoids generating
  files that may need rewriting in a few months.

### 4. Hardening: rely-on-defaults → explicit `hardening=+all`

`docs/house-style.md:159-162`

**Findings:**

- **Substantive content choice is defensible.** Many modern
  Debian packages do enable `hardening=+all` explicitly to cover
  the optional flags (`pie`, `bindnow`) on packages that don't
  get them by default. Pinning this in house-style is a
  reasonable opinion.
- **Tonal issue**: the new wording reads "by default seems
  reasonable" — hedging language ("seems reasonable") doesn't
  fit a prescriptive doc that other rules in the same file
  state as imperatives. Replace with:
  `Set "export DEB_BUILD_MAINT_OPTIONS = hardening=+all" in
  debian/rules.`
- **Logical consistency with rest of file**: the previous wording
  said "rely on defaults; do not override unless..." — the new
  rule itself is an override, so the second clause should be
  rewritten to match. Suggested phrasing:
  `Override the hardening set only with explicit justification
  (e.g. an upstream that breaks under FORTIFY).`
- **Whitespace**: extra space before `**DD-judgement.**`. Cosmetic.

## Knock-on inconsistencies (templates not updated)

The bootstrap worker ships templates that now contradict the
house style. Confirmed by grep over
`skills/bootstrap/templates/*.tmpl`:

| File | Line | Current value | Should be |
|---|---|---|---|
| `skills/bootstrap/templates/control.tmpl` | 6  | `debhelper-compat (= 13),` | `debhelper-compat (= 14),` |
| `skills/bootstrap/templates/control.tmpl` | 10 | `Standards-Version: 4.7.1` | `Standards-Version: 4.7.4` |
| `skills/bootstrap/templates/watch.tmpl`   | 1  | `version=4`                | `version=5` (if keeping v5) |
| `skills/bootstrap/templates/watch.tmpl`   | 3  | `opts=pgpmode=auto \`      | `Pgp-Mode: auto`            |
| `skills/bootstrap/templates/watch.tmpl`   | 6  | `opts=pgpmode=none \`      | `Pgp-Mode: none`            |
| `skills/bootstrap/templates/rules.tmpl`   | —  | minimal `dh $@` only       | add `export DEB_BUILD_MAINT_OPTIONS = hardening=+all` above the `%:` rule |

If watch v5 is rolled back to v4 (option above), templates stay
on v4 and the `Pgp-Mode` field change reverts.

The bootstrap worker's `SKILL.md` already cites house-style as the
source of truth, but `bootstrap` will copy template values
verbatim before substitution — drift between template and
house-style means generated `debian/` directories will fail their
own house-style audit on the very next `/debaid:lintian` run.

## Critical files to update (recommended order)

1. `docs/house-style.md` — fix the four wording issues (grammar,
   hedging, whitespace, compat-14 rationale). One-pass cleanup.
2. **External verification step** (user): confirm
   - debhelper-compat 14 status in Debian unstable
   - Standards-Version 4.7.4 exists
   - watch v5 tooling maturity (devscripts, salsa-CI runners)
3. `skills/bootstrap/templates/control.tmpl` — update compat
   default + Standards-Version.
4. `skills/bootstrap/templates/watch.tmpl` — depends on the v5
   verification outcome (either bump syntax or revert house-style
   to v4).
5. `skills/bootstrap/templates/rules.tmpl` — add the hardening
   export.

## Verification

After edits:

```
grep -E 'debhelper-compat|Standards-Version|version=[345]|Pgp-Mode|pgpmode|DEB_BUILD_MAINT|hardening' \
  skills/bootstrap/templates/*.tmpl docs/house-style.md
```

Every match in `templates/*.tmpl` should agree with the
corresponding match in `docs/house-style.md`. A second sanity
check: render the bootstrap templates against the
`hello-c-autotools/` fixture once it's promoted from stub to real
(`tests/run-fixtures.sh hello-c-autotools`) and confirm
`lintian -EvIL +pedantic` doesn't fire `out-of-date-standards-version`
or `package-uses-deprecated-debhelper-compat-version`.

## Out of scope

- License choice (still TBD in README).
- The remaining P1/P2/P3 items from the running review table.
- The bootstrap worker's `SKILL.md` body itself — it references
  house-style by path so it auto-tracks; no edit needed.

---

## Updates

This plan was written as a snapshot review of four house-style
changes. The actual work that landed went well beyond that scope —
the review surfaced template drift, which surfaced an Ubuntu-overlay
gap, which surfaced a need for new reference docs and a Patches
workflow declaration. Tracking the divergence below for future-me
who comes back to this file expecting a 1:1 mapping.

### Round 1 — the four flagged changes settled

- `8a5efe2` "house-style: refresh some oddities" — debhelper-compat
  **reverted to 13** ("battle-tested now") rather than promoted to
  14; hardening rewritten in imperative form
  (`export DEB_BUILD_MAINT_OPTIONS = hardening=+all`); watch v5
  retained with an added "ask the maintainer when in doubt"
  bullet. Template `rules.tmpl` got a `{{#has_compiled_binaries}}`
  guard so pure-data/Python packages don't carry a spurious export.
  Template `watch.tmpl` rewritten for RFC822-style v5 syntax with
  a `{{#has_template}}` conditional.
- `5b828bf` "docs: add references files" — `Standards-Version
  4.7.4`, plus much more than the plan called for: `Priority:`
  field handled per Policy 4.7.3 §5.6.6 (omit when default),
  `Git-Tag-Tagger`/`Git-Tag-Info` (Policy 4.7.3 §5.6.32-33),
  `non-free-firmware` rule (Policy §12.5 4.7.4), Build profiles
  section, full Ubuntu overlay section, five new reference docs
  (`build-profiles.md`, `git-tag-fields.md`, `sru.md`,
  `ubuntu-merges-syncs.md`, `ubuntu-versioning.md`).
- `d7a8259` "Update skills and scripts to use the latest version of
  policy" — schema additions (`source.ubuntu_delta`,
  `target.pocket`, `target.freeze_state`, seven Ubuntu-workflow
  tools in `tooling.*`), `tooling-probe.sh` extended, deny-list
  expanded (`dch -D`, `dch --distribution`), bootstrap
  `control.tmpl` got the conditional `{{#priority_nondefault}}`
  Priority block, new `tests/fixtures/ubuntu-merge/` stub.
- `b0bdf00` "skills: fix some mistakes in refresh skills" — refresh
  worker's `--rrr` flag flipped from "add/normalise" to
  "remove if redundant" (preparing for the R³ change that landed
  later); `--watch-v4` flag renamed to `--watch-v5`.

### Round 2 — follow-up review items I flagged and you accepted

- `e3e3559` "consistency fixes due to my house-style changes" —
  bundles the seven items I called out in the post-Round-1 review:
  stale `verify.sh` path in `shared-context.md` fixed to
  `${CLAUDE_PLUGIN_ROOT}/scripts/verify.sh`; R³ rule rewritten as
  "omit the field" with bootstrap template line dropped to match;
  refresh audit example target `4.7.1` → `4.7.4`; new
  `## Template render flags` section in `bootstrap/SKILL.md`
  documenting the new Mustache flags (`priority_nondefault`,
  `has_compiled_binaries`, `has_signing_key`, `has_template`) and
  the scalar/list variables; new `docs/references/watch-v5.md`
  with three idiomatic examples plus v4→v5 migration table;
  expanded quarterly-review checklist plus a "Drift hot-spots"
  table in `developer.md`.

Skipped at user's request: `ubuntu_delta` regex fix for the
`ubuntuNbuildM` shape — left for later.

### Round 3 — Patches workflow

- `0cb2333` "house-style: Clarify patching workflow and DEP-14
  expectations" — house-style Patches section rewritten (seven
  wording fixes: gbp-only declaration with a non-gbp bail
  clause, `patch-queue/<branch>` spelled out with the
  `patch-queue/debian/unstable` example, three-option choice when
  the patch-queue branch already exists, "author as commits"
  language replacing "apply patches", "Review" replacing
  "Control", three-bullet new-upstream sequence with explicit
  maintainer-conflict caveat, DD-judgement markers added
  throughout); `debian/latest` → `debian/unstable` across six
  files (kept in `scripts/detect-source.sh` regex which matches
  all three conventions for *detection*); `skills/lintian/SKILL.md`
  "fix upstream via patch" classification now points at the
  Patches section as the authoring authority.

Plus a defensive note added in a separate commit (working tree
state at session end) to `bootstrap/SKILL.md` step 7: if a
verify failure needs a patch, follow the house-style Patches
workflow — never write to `debian/patches/` directly. Pointer
to house-style only; no duplication of the actual rule.

### Sibling commits (not from this plan's scope but landed
adjacent)

- `58cc113`, `a6933a0` — `decopy` and `lrc` added to
  `tooling-probe.sh`. Not reviewed in this plan; flagged here
  only so future-me knows these were not silently mine.

### What got externally verified

The plan asked the user to verify three things against current
Debian state. From the commits that landed:

- debhelper-compat default — verified, decision was to **stay on
  13** (not bump to 14 as the original house-style edit
  suggested).
- Standards-Version 4.7.4 — verified, Policy 4.7.4 (March 2026)
  is real and cited in `house-style.md` header.
- watch v5 — verified, kept as the prescribed default with a
  "ask the maintainer when in doubt" escape hatch in
  `house-style.md` and worked examples in
  `docs/references/watch-v5.md`.

### What the verification grep looks like now

The grep at the bottom of the plan still works; current output:

```
$ grep -E 'debhelper-compat|Standards-Version|Version: [345]|version=[345]|Pgp-Mode|pgpmode' \
    skills/bootstrap/templates/*.tmpl docs/house-style.md
skills/bootstrap/templates/watch.tmpl:Version: 5
skills/bootstrap/templates/watch.tmpl:Pgp-Mode: auto
skills/bootstrap/templates/watch.tmpl:Pgp-Mode: none
docs/house-style.md:- `debhelper-compat (= 13)` ...
docs/house-style.md:- `Standards-Version: 4.7.4`. ...
skills/bootstrap/templates/control.tmpl: debhelper-compat (= 13),
skills/bootstrap/templates/control.tmpl:Standards-Version: 4.7.4
```

Template and house-style agree on compat 13, Standards-Version
4.7.4, watch v5, Pgp-Mode field syntax. No drift.
