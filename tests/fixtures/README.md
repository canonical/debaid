# tests/fixtures — reference packages and test inputs

This directory serves two purposes simultaneously:

1. **End-to-end test inputs.** Each fixture is a minimal upstream
   project that exercises one or more workers.
2. **Reference corpus.** Workers consult `<fixture>/debian/` (for
   fixtures that have one) as idiom examples — see
   `shared-context.md` § "Reference-corpus contract" at the
   plugin root.

The fixture driver lives one level up at `tests/run-fixtures.sh`.
It walks each fixture, runs `<fixture>/test.sh` if executable, and
reports `[PASS]`/`[FAIL]`/`[STUB]` per fixture. Pass `--strict` to
fail when stubs remain.

Not to be confused with `scripts/verify.sh` (at the plugin root),
which is the build+lintian snapshot tool that worker skills call
between iteration attempts — see `shared-context.md`.

## Fixtures (planned)

| Fixture | Upstream shape | Workers exercised |
|---|---|---|
| `hello-c-autotools/` | C, autotools, single binary | bootstrap |
| `hello-go/` | Go, go.mod, single binary | bootstrap, autopkgtest |
| `hello-rust/` | Rust, cargo, library + bin | bootstrap |
| `hello-python/` | Python, pyproject, library | bootstrap, autopkgtest |
| `stale-debian/` | Pre-existing out-of-date debian/ | refresh, lintian |
| `ubuntu-merge/` | Debian + Ubuntu delta pair       | merge (future) |

## Status

**These fixtures are stubs (TODO).** First version of debaid
ships with the directory layout and READMEs only. Real upstream
trees and golden `debian/` outputs are a follow-up.

When promoting a fixture from stub to real:

1. Drop a minimal but realistic upstream source under
   `<fixture>/`.
2. Write the expected post-bootstrap `debian/` tree to
   `<fixture>/expected/debian/`.
3. Add a `<fixture>/test.sh` driver that:
   - Copies upstream to a scratch dir.
   - Runs the relevant worker via
     `${DEBAID_CLAUDE_CMD:-claude --bare --print} '/debaid:bootstrap ...'`.
   - Diffs the produced `debian/` against `expected/debian/`.
   - Reports lintian/sbuild status.

## What good fixtures look like

- Build in under 60s on a laptop.
- Cover one ecosystem cleanly; don't pile on edge cases per
  fixture.
- Use a license workers can recognise (MIT, Apache-2, GPL-2+).
- Have an obvious entry point (`main()`, `lib.rs`, etc.).
- Avoid network access in the build.
