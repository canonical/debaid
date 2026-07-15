# debaid

A self-contained CLI that helps a Debian / Ubuntu maintainer
bootstrap, modernise, lint-clean, and add autopkgtest coverage to a
package -- while keeping the maintainer in the loop for every
judgement call.

debaid is **standalone**: a single binary you install and run
directly. It is not a plugin and needs no AI CLI or plugin
machinery -- it runs on its own and sits happily alongside your
editor or agent of choice (e.g. opencode).

debaid is **prescriptive** (house style + policy + tooling
indicators) and **respectful** (dry-run by default on destructive
operations, no uploads, no version bumps without an explicit
request, no Maintainer-field changes).

## Approach

debaid is a single, standalone Rust binary -- no dependency on any
other AI CLI. Its only external needs are the real Debian tools it
drives (`lintian`, `sbuild`, `dpkg-buildpackage`, `gbp`, ...) and an
OpenRouter-compatible endpoint for the LLM judgement calls -- the
provider and model are config, not a build-time choice.

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

```
export OPENROUTER_API_KEY=sk-or-...
```

Model and base URL default to OpenRouter and are overridable via
`~/.config/debaid/config.toml`, so any OpenAI-compatible endpoint
works.

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
agent loop is not wired up yet.

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
  worker obeys (context schema, verify output, iteration budget,
  bail-out format).
- [`docs/house-style.md`](./docs/house-style.md) -- every
  prescriptive choice, with citations.
- [`docs/developer.md`](./docs/developer.md) -- adding a worker or a
  language overlay.

## License

GPL-3.0-only. See [`LICENSE`](./LICENSE).
