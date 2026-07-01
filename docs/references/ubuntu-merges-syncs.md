# Ubuntu merges and syncs

**Last reviewed**: 2026-05-13

Ubuntu is downstream of Debian. New Debian package versions
arrive in Ubuntu via two processes:

- **Sync** — Ubuntu has no delta; the new Debian version is
  copied verbatim. Automatic during the dev cycle until DIF
  (Debian Import Freeze).
- **Merge** — Ubuntu has a delta to preserve; the Ubuntu
  changes must be re-applied on top of the new Debian version.

The canonical Ubuntu docs:
`how-ubuntu-is-made/processes/merges-and-syncs.rst`.

## Decision matrix

| Ubuntu delta? | New Debian available? | Action            |
|---------------|-----------------------|-------------------|
| no            | yes                   | sync (auto)       |
| no (build-only)| yes                  | sync (auto)       |
| yes           | yes                   | merge             |
| yes, but upstreamed | yes             | re-sync (manual)  |

Manual sync request: `requestsync` (from `ubuntu-dev-tools`) or a
Launchpad ticket. Manual merge: `git ubuntu merge` workflow.

## Useful tooling

| Tool                  | Purpose                                       |
|-----------------------|-----------------------------------------------|
| `git-ubuntu`          | Launchpad-git workflow front-end              |
| `requestsync`         | File a sync request bug                       |
| `pull-debian-source`  | Fetch a Debian source package                 |
| `pull-lp-source`      | Fetch an Ubuntu source package from Launchpad |
| `syncpackage`         | Perform a sync (requires upload rights)       |
| `update-maintainer`   | Move `Maintainer:` to `XSBC-Original-Maintainer:` |

`tooling-probe.sh` reports each of these as available/missing.

## Merges-o-Matic

The MoM service at https://merges.ubuntu.com/ pre-computes merges
and publishes per-component reports (`main`, `universe`,
`restricted`, `multiverse`). When a merge is on the report, the
heavy lifting is already done — the maintainer reviews and uploads.

## What workers do (when a /debaid:merge worker exists)

- Detect `source.ubuntu_delta`.
- Read the changelog and identify the Ubuntu delta commits.
- Propose a target version (`<new-debian>ubuntu1`) per
  `ubuntu-versioning.md`.
- Run `update-maintainer` if absent.
- **Never** drop a delta entry without the maintainer's
  explicit instruction. Empty merges go through re-sync, not
  merge.

## Freeze gates

Once DIF lands, sync automation stops. After DIF and before final
release, sync/merge needs a freeze exception. After final release,
the package is under SRU rules — see `docs/references/sru.md`.

## See also

- Ubuntu docs: `how-ubuntu-is-made/processes/merges-and-syncs.rst`.
- `release-team/freezes.md` for the cycle gates.
