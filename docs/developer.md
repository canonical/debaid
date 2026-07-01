# debaid — developer guide

How to extend `debaid` with a new worker, fixture, or reference
doc.

`debaid` is packaged as a Claude Code plugin. The plugin
manifest is `.claude-plugin/plugin.json`; everything else
(skills, scripts, docs, tests) sits at the plugin root. Skills
become slash commands under the `/debaid:` namespace —
`skills/lintian/SKILL.md` ⇒ `/debaid:lintian`, and so on. The
orchestrator lives at `skills/run/` and is invoked as
`/debaid:run`.

## Adding a new worker

A worker is a Claude Code skill — a directory under `skills/`
containing at least `SKILL.md`.

Steps:

1. **Pick a name.** Workers are named after their phase:
   `bootstrap`, `refresh`, `lintian`, `autopkgtest`. Pick a short
   verb or noun; the slash command will be `/debaid:<name>`.
2. **Create the skill directory.**
   ```
   mkdir -p skills/<name>
   ```
3. **Write `SKILL.md`** with frontmatter:
   ```yaml
   ---
   name: <name>
   description: One sentence the orchestrator (and skill discovery)
     will use to decide when to invoke this. Lead with the user
     intent, not the implementation. Mention "Debian" or "Debian
     source package" to anchor scope.
   ---
   ```
   The body MUST cover:
   - Preconditions (what must be true about the context, including
     the "if context.json is missing, build it via the probe
     scripts" recovery path).
   - Hard rules — both the suite-wide ones (inlined, see below)
     and any phase-specific bans.
   - Process (numbered steps from input to output).
   - Bail-out conditions.
4. **Inline the suite-wide hard rules.** Each worker's `## Hard
   rules` section opens with the suite-wide list verbatim, then
   adds phase-specific rules. The canonical list lives in
   `shared-context.md`. If you change one rule, update all five
   worker `SKILL.md` files and the spec file in the same commit.
5. **Reference shared assets by `${CLAUDE_PLUGIN_ROOT}`.** The
   plugin runtime substitutes that variable before the LLM sees
   the prompt. Always use it; never use `../<sibling-skill>/...`
   — that path will not resolve after the plugin is installed.

   Common references:
   - `${CLAUDE_PLUGIN_ROOT}/scripts/detect-source.sh`
   - `${CLAUDE_PLUGIN_ROOT}/scripts/tooling-probe.sh`
   - `${CLAUDE_PLUGIN_ROOT}/scripts/verify.sh`
   - `${CLAUDE_PLUGIN_ROOT}/shared-context.md`
   - `${CLAUDE_PLUGIN_ROOT}/docs/house-style.md`
   - `${CLAUDE_PLUGIN_ROOT}/docs/references/*.md`
6. **Cite the house style.** Every prescriptive choice in the
   worker output must trace back to
   `${CLAUDE_PLUGIN_ROOT}/docs/house-style.md` or
   `${CLAUDE_PLUGIN_ROOT}/docs/references/*.md`.
7. **Update the orchestrator.** Add the phase to the dispatch
   table in `skills/run/SKILL.md`.
8. **Add a fixture** under `tests/fixtures/` that exercises the
   worker end-to-end (or annotate an existing fixture to cover
   the new phase).
9. **Update the README phase list.**

## Adding a fixture

See `tests/fixtures/README.md`. Each fixture is a minimal
upstream tree plus an `expected/` golden output and a `test.sh`
driver. `tests/run-fixtures.sh` walks the fixtures directory and
runs each fixture's `test.sh`.

## Adding a reference doc

Reference docs live under `docs/references/`. Keep them:

- **Short.** One page each. Cheatsheets, not textbooks.
- **Opinionated.** State the preferred choice and why.
- **Cited.** Link to the canonical source (Debian Policy,
  devref, DEP, upstream manpage).
- **Dated.** Add `**Last reviewed**: YYYY-MM-DD` at the top.

## Adding a language overlay

Language overlays specialise the workers' output by detected
`source.language`. The four shipped overlays (Python, Rust, Go,
Perl) are the canonical examples; follow their shape when
adding a new one.

### When to add an overlay

Add an overlay when both of these hold:

- The target language has Debian-specific packaging conventions
  the workers would otherwise get wrong (a build helper like
  `dh-cargo` / `pybuild`, a watch template, an autodep8
  generator, a refresh-time cruft pattern).
- The language is established enough in Debian that multiple
  users of debaid are likely to package in it.

### Files an overlay touches

A complete language overlay is three concerns:

```
docs/references/languages/<lang>.md          # the overlay doc
skills/bootstrap/templates/rules.<lang>.tmpl # rules variant
skills/{bootstrap,refresh,autopkgtest}/SKILL.md   # anchor edits
```

The rules template can be near-empty for languages whose dh
setup is just `dh $@` (see `rules.perl.tmpl`). For languages
with build-system flags or required env vars, copy the shape
of `rules.python.tmpl`.

### Recipe

1. **Pick a language code.** This is the value `source.language`
   will hold. The current enum lives in
   `${CLAUDE_PLUGIN_ROOT}/shared-context.md` § "JSON schema
   (v1)". If your language is not in that enum, extend it
   first.

2. **Write the overlay doc.** Create
   `docs/references/languages/<lang>.md`. Follow the shape of
   the existing overlays — at minimum these sections:

   - `# <Language> — debaid overlay` + a short intro pointing
     back to `house-style.md`.
   - Authoritative upstream sources (Debian policy / team /
     manpages).
   - `## When this overlay applies` — what triggers
     `source.language == "<lang>"` detection.
   - `## Library packages vs. application binaries` if the
     language has both shapes.
   - `## Initial packaging (comparison run only)` if a
     `dh-make-*` tool exists.
   - `## Package naming` — explicit naming conventions.
   - `## File layout` — install paths.
   - `## debian/control essentials` — Build-Depends template
     and binary stanza examples.
   - `## debian/rules` — exact contents of the corresponding
     `rules.<lang>.tmpl` plus common overrides.
   - `## debian/watch (v5 with <template>)` — the right v5
     template if one exists.
   - `## autopkgtest` — autodep8 shortcut if a generator exists,
     otherwise an explicit "no autodep8" statement.
   - `## Common refresh checks` — the per-language items the
     refresh worker should surface.
   - `## Bail-out conditions` — language-specific bail-outs in
     addition to the worker-level ones.

3. **Align to debaid house style.** Three frequent traps:
   - **Watch v5 only.** Never recommend v3 or v4 watch syntax.
   - **No `dh_make`-style output shipped.** `dh-make-perl`,
     `dh-make-golang`, `py2dsc`, etc. are comparison runs in
     `/tmp`, not the source of truth.
   - **sbuild-first verification.** Refer to
     `${CLAUDE_PLUGIN_ROOT}/scripts/verify.sh`, not
     `dpkg-buildpackage -us -uc`, as the primary check.

4. **Add the rules template.** Create
   `skills/bootstrap/templates/rules.<lang>.tmpl`. Gate
   hardening with `{{#has_compiled_binaries}}…{{/has_compiled_binaries}}`
   when the language produces native object code. The
   dispatch table in `skills/bootstrap/SKILL.md` § "Language
   dispatch" must list this template.

5. **Wire bootstrap.** Edit `skills/bootstrap/SKILL.md`:
   - Confirm (or add) the row in § "Language dispatch".
   - Add a bullet under Process step 4 "Build-deps discovery"
     pointing at the new overlay's
     `## debian/control essentials` section.
   - If the template needs a scalar (e.g. `pybuild_name` for
     Python), add it to the Scalar values table.

6. **Wire refresh.** Edit `skills/refresh/SKILL.md`: replace
   the language's TBD stub in § "Language-aware audit" with a
   short summary of the checks, linking back to the overlay's
   `## Common refresh checks` section.

7. **Wire autopkgtest.** Edit `skills/autopkgtest/SKILL.md`:
   - If the language has an autodep8 generator, update the
     "Notes" cell in the autodep8 shortcut table.
   - If no generator exists, extend the explicit
     "No autodep8 generator exists for …" line that currently
     covers Rust and Go.

8. **No fixture work in the overlay commit.** Fixture
   promotion is handled in a separate pass — see
   `tests/fixtures/README.md`. Shipping overlay text and a
   template does not require touching `tests/`.

### DRAFT marker convention

If you draft an overlay without input from a maintainer who
actively packages in that language, mark it DRAFT:

- Add a `> **⚠️ DRAFT.** …` blockquote at the top of the file
  with a one-line description of the review needed.
- Add a `## DRAFT marker — likely needs correction` section at
  the end listing the bits most likely to be wrong or
  outdated.
- Carry the DRAFT caveat through to the refresh and autopkgtest
  worker references (e.g. "see `<lang>.md` (DRAFT pending …
  review)").

`docs/references/languages/perl.md` is the worked example.

### Worked examples

| Language | Overlay | Rules template | Notes |
|---|---|---|---|
| Python | `docs/references/languages/python.md` | `rules.python.tmpl` | `--buildsystem=pybuild` + `pybuild_name` scalar; autodep8 `autopkgtest-pkg-python`. |
| Rust | `docs/references/languages/rust.md` | `rules.rust.tmpl` | dh-cargo for application binaries; debcargo bail-out for library crates; no autodep8. |
| Go | `docs/references/languages/golang.md` | `rules.golang.tmpl` | `--buildsystem=golang` + `XS-Go-Import-Path:` source-stanza field; no autodep8. |
| Perl | `docs/references/languages/perl.md` | `rules.perl.tmpl` | Plain `dh $@`; autodep8 `autopkgtest-pkg-perl`; DRAFT pending pkg-perl review. |

## Quarterly review checklist

The house style and references go stale. Quarterly, check:

- [ ] `Standards-Version` default in `house-style.md` matches
      both `skills/bootstrap/templates/control.tmpl` and the
      current Debian Policy package version.
- [ ] `debhelper-compat` default in `house-style.md` matches
      `skills/bootstrap/templates/control.tmpl` and current
      debhelper.
- [ ] Salsa-CI pinned ref in `docs/references/salsa-ci.md` and
      `skills/bootstrap/templates/salsa-ci.yml.tmpl`.
- [ ] Each `docs/references/*.md` `Last reviewed` date < 6 months.
- [ ] Any new accepted DEPs.

**Drift hot-spots.** A handful of values are pinned in
`docs/house-style.md` AND duplicated in
`skills/bootstrap/templates/`. They must move together:

| Value                      | Files                                                                                |
|----------------------------|--------------------------------------------------------------------------------------|
| `debhelper-compat` version | `docs/house-style.md` (debhelper §) + `skills/bootstrap/templates/control.tmpl`      |
| `Standards-Version`        | `docs/house-style.md` (Control fields §) + `skills/bootstrap/templates/control.tmpl` |
| Salsa-CI pinned ref        | `docs/references/salsa-ci.md` + `skills/bootstrap/templates/salsa-ci.yml.tmpl`       |
| Watch syntax version       | `docs/house-style.md` (debian/watch §) + `skills/bootstrap/templates/watch.tmpl`     |

`debian/control` has no comment syntax, so no inline reminder
lives in the template — the discipline is here.

## Style for SKILL.md prompts

- **Address the LLM.** "Run X, then Y, then ask the maintainer."
- **Bullet rules, not paragraphs.** Easier for the model to
  attend to.
- **Hard rules in a dedicated section** titled "Hard rules". Use
  the same phrasing across workers so the prompts reinforce.
- **Examples for fiddly formats.** E.g. show what a good
  `lintian-overrides` comment looks like; the model will copy
  the shape.
- **Avoid hedging in instructions.** "MUST" / "MUST NOT" / "MAY"
  / "SHOULD" with their RFC-2119 meanings.

## Testing changes locally

```
# Load the plugin from this checkout for the current session:
claude --plugin-dir /path/to/debaid

# Then invoke a worker as a slash command:
/debaid:lintian
```

After editing a skill, run `/reload-plugins` to pick up changes
without restarting Claude Code.

The workshop's `claude-exec` action wires the equivalent
non-interactive form — see `workshop.yaml`.

## Plugin manifest

`.claude-plugin/plugin.json` carries the plugin name, description,
version, and author. The `name` field controls the slash-command
namespace (`/debaid:<skill>`). Bump `version` on every release
that ships to users — Claude Code uses it to decide whether to
re-fetch the plugin.
