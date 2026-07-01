---
name: autopkgtest
description: Add or improve debian/tests/ for as-installed Debian package testing per DEP-8. Detects package shape (library, daemon, CLI tool), proposes minimal Restrictions, runs tests with the lightest virt backend available. Asks the maintainer before enabling isolation-container or needs-root.
---

# debaid:autopkgtest

Add or improve `debian/tests/` for as-installed testing.

## Preconditions

- A context JSON exists at `./.debaid/context.json`. If missing,
  build it: run `${CLAUDE_PLUGIN_ROOT}/scripts/detect-source.sh`
  and `${CLAUDE_PLUGIN_ROOT}/scripts/tooling-probe.sh`, merge their
  outputs (see `${CLAUDE_PLUGIN_ROOT}/shared-context.md` for the
  full schema).
- `source.has_debian_dir == true`.
- The package builds (you don't need to verify; the orchestrator
  ensures this when chained).
- `tooling.autopkgtest.available == true` for the run phase; the
  authoring phase works without it but tests are not verified.

## Package shape detection

Inspect `debian/control` and the built artefacts to classify:

- **Library** — binaries named `lib*` providing `.so` or
  language-specific equivalents (`.rlib`, `.a` for static-only).
  Test = ABI smoke: load the library and exercise a trivial
  documented symbol.
- **CLI tool** — `Section: utils`, `devel`, `text`, etc. with
  one or more binaries in `/usr/bin/`. Test = `<bin> --version`,
  `<bin> --help`, plus the most common subcommand if obvious.
- **Daemon** — ships a `*.service` unit, listens on a port,
  `Section: net`, `admin`, etc. Test = start + healthcheck +
  stop. Restrictions needed; ASK before adding.
- **Library + tool** — provide both kinds of tests, in separate
  test files.
- **Pure data/docs** — `Architecture: all` with no executables.
  Test = installability + content sanity (file presence). Often
  a single `superficial` test.

## Restrictions defaults

Start with the *minimum* that makes the test pass:

| Need | Restriction |
|---|---|
| Test only reads installed files | (none) |
| Test writes to `$AUTOPKGTEST_TMP` | `allow-stderr` if stderr is expected |
| Test needs network | `needs-internet` (justify) |
| Test starts a service | `isolation-container` (justify, ASK) |
| Test must run as root | `needs-root` (justify, ASK) |
| Test depends on the build tree | `needs-build` |

**`isolation-container` and `needs-root` require maintainer
approval.** Do not enable them silently.

## Autodep8 shortcuts

For some upstream ecosystems, the `autodep8` framework generates
`debian/tests/control` automatically from a single `Testsuite:`
line in `debian/control`. When the language matches one of the
generators below, **prefer the autodep8 shortcut over hand-rolling
`debian/tests/control`** — fewer lines to maintain, and the
generator tracks ecosystem conventions you would otherwise have
to encode by hand.

| `source.language` | `Testsuite:` value | Notes |
|---|---|---|
| `python` | `autopkgtest-pkg-python` | Generates `python3 -c "import <name>"` per `python3-*` binary; the import name is derived from the package suffix. Set `X-Python3-Module:` when it diverges. See `${CLAUDE_PLUGIN_ROOT}/docs/references/languages/python.md` § "autopkgtest". |
| `perl` | `autopkgtest-pkg-perl` | Runs `prove -v` against upstream's test suite using the installed package; for XS modules also verifies the compiled `.so` loads. See `${CLAUDE_PLUGIN_ROOT}/docs/references/languages/perl.md` § "autopkgtest" (DRAFT). |
| `ruby` | `autopkgtest-pkg-ruby` | Ruby overlay deferred; the generator works without an overlay. |
| `nodejs` | `autopkgtest-pkg-nodejs` | No overlay planned in this pass; the generator works without one. |

**No autodep8 generator exists for Rust or Go.** Application
binaries in those languages still go through the full Process
below; do not propose a `Testsuite:` line for them.

Use the shortcut only when the upstream test suite is structured
in the way the generator expects (run on the installed package,
no unusual fixtures, no network). If the package has non-default
test requirements (custom env vars, non-standard test entry
points, mocked services), hand-roll instead and link to the
relevant `languages/<lang>.md` for ecosystem nuances.

## Process

1. **Load context.**
2. **Inspect package shape.** Read `debian/control`, the
   filesystem contents of the built `.deb` (if available), and
   the upstream test layout (if any). Report shape to maintainer.
3. **Propose tests.** If an autodep8 shortcut applies (see §
   "Autodep8 shortcuts" above), propose the `Testsuite:` line in
   `debian/control` first and stop here unless the maintainer
   rejects it. Otherwise, for each binary package, propose one or
   more test cases with their `Test-Command:` or script body.
   Print the proposal as a draft `debian/tests/control` plus any
   per-test scripts under `debian/tests/`.
4. **Confirmation gate.** If any restriction beyond the
   minimum-set is required, ASK the maintainer. Same for any
   `Depends:` lines that pull in heavy extras.
5. **Write phase.** Create `debian/tests/control` and the test
   scripts. Make scripts executable. Run `wrap-and-sort` on
   control if applicable.
6. **Verification phase.** If `tooling.autopkgtest.available ==
   false`, report that tests are authored but not verified and
   stop here. Otherwise run `autopkgtest -- null` first
   (cheapest). On success, suggest running under
   `autopkgtest-virt-qemu` or `autopkgtest-virt-lxc` if
   installed; ask before launching (those are slow).
7. **Report.** Files added, test results, restrictions chosen,
   anything deferred.

## Authoring guidelines

- One test per file under `debian/tests/`, named after what it
  tests (e.g. `cli-version`, `lib-smoke`, `daemon-startup`).
- Bash test scripts: `set -euo pipefail` at top, `cd
  "$AUTOPKGTEST_TMP"` early.
- Python test scripts: do not pollute the system Python; install
  the package under test into a virtualenv only if absolutely
  needed, otherwise rely on the installed Debian package.
- Avoid `Test-Command:` one-liners longer than ~80 chars; move
  to a script for readability.
- `Depends: @` pulls in the package's own binary; use it instead
  of listing the binary name explicitly when possible.
- `Depends: @builddeps@` is rarely the right answer — it's huge.

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

Phase-specific (autopkgtest):

- **Never enable `isolation-container` or `needs-root` without
  asking the maintainer.**
- **Never write tests that hit the public internet** unless
  `needs-internet` is set AND the maintainer approves.
- **Never download upstream test fixtures during the test** —
  if the test needs data, it goes in `debian/tests/` or comes
  from the package itself.

## Bail-out conditions

- Package shape is ambiguous (e.g. library that also ships a
  CLI but the CLI is private to the build).
- Test framework requires `isolation-container` and maintainer
  declines.
- `autopkgtest -- null` fails for reasons not attributable to
  the test (env issue, broken backend).

Use the bail-out summary format from
`${CLAUDE_PLUGIN_ROOT}/shared-context.md` § "Bail-out summary
format".
