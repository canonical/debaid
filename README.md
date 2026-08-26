# debaid

A self-contained CLI that helps a Debian / Ubuntu maintainer
bootstrap, modernise, lint-clean, and add autopkgtest coverage to a
package -- while keeping the maintainer in the loop for every
judgement call.

debaid is a **CLI tool**: a single binary you install and run
directly. It is not an editor plugin -- it drives
[opencode](https://opencode.ai) under the hood for the LLM
judgement calls, and otherwise stays out of your editor's way.

debaid is **prescriptive** (house style + policy + tooling
indicators) and **respectful** (dry-run by default on destructive
operations, no uploads, no version bumps without an explicit
request, no Maintainer-field changes).

## Approach

debaid is a single Rust binary that shells out to two kinds of
tool: the real Debian tools it drives (`lintian`, `sbuild`,
`dpkg-buildpackage`, `gbp`, ...) and `opencode`, which carries the
LLM judgement calls. The provider and model are opencode config,
not a build-time choice -- any OpenAI-compatible endpoint works.

The split:

- **Deterministic Rust** -- source/tooling detection, verify (build
  + lintian snapshot), template rendering, `debian/` edits via
  comment-preserving deb822 parsing, iteration budgets and the
  command deny-list.
- **LLM** -- lintian tag classification (fix / patch / override /
  won't-fix), prose (long descriptions, override reasons, bail-out
  summaries), package-shape inference for autopkgtest.

`debaid detect` and `debaid verify` are pure deterministic paths
and run with no API key.

## Configure

debaid needs `opencode` on `PATH`, plus a key for whichever
provider opencode is pointed at:

```
export OPENROUTER_API_KEY=sk-or-...
```

Model and endpoint are opencode's own configuration; debaid
settings live in `~/.config/debaid/config.toml`. See
[`.envrc.example`](./.envrc.example) for a working OpenRouter
setup, and [`opencode.json`](./opencode.json) for the command
deny-list debaid ships.

## Commands

```
debaid run                              # full pipeline
debaid run --only=refresh,lintian       # phase subset
debaid lintian                          # single phase
debaid detect                           # context JSON, no LLM
debaid verify                           # build + lintian, no LLM
```

Global flags: `--dry-run`, `--house-style=PATH`, `--reference=PATH|none`,
`--yes` (skip per-phase confirmation gates), `-v`.

## Status

Early. The CLI skeleton is in place; the phases are stubs and the
opencode-backed agent loop is not wired up yet.

Language overlays ship for Python (pybuild), Rust (dh-cargo; bails
out to debcargo for libraries), Go (dh-golang), and Perl (DRAFT,
pending pkg-perl review). Other languages fall back to a generic
`dh $@` skeleton.

## What it will not do

- Upload anything (`dput`, `debrelease`, `dgit push`, `git push`).
- Edit `debian/changelog` distribution away from `UNRELEASED`.
- Edit `Maintainer:` or `Uploaders:`.
- Edit upstream sources in place (always a DEP-3 quilt patch).
- Suppress lintian tags without a `# reason:` comment.
- Set `Multi-Arch: same` without verifying file paths.

These are enforced in code, not just in prompts.

## Docs

- [`shared-context.md`](./shared-context.md) -- the contract every
  worker obeys (field semantics, iteration budget, bail-out format,
  hard rules). The document shapes themselves live in `src/model/`.
- [`docs/house-style.md`](./docs/house-style.md) -- every
  prescriptive choice, with citations.
- [`docs/developer.md`](./docs/developer.md) -- adding a worker, a
  tool, or a language overlay.

## License

GPL-3.0-only. See [`LICENSE`](./LICENSE).
