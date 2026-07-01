# Go — debaid overlay

Debian Go packaging is centralised around the `dh-golang`
build system and the `dh-make-golang` initial-conversion tool.
Read alongside `${CLAUDE_PLUGIN_ROOT}/docs/house-style.md`;
this file documents what is *Go-specific*.

Authoritative upstream sources:

- Debian Go Packaging Team: https://salsa.debian.org/go-team
- `man dh-golang`, `man dh-make-golang`.

## When this overlay applies

`source.language == "go"` in the shared context. Detected by
the presence of `go.mod` at the source root (or, for very old
upstreams, an absence of `go.mod` combined with `.go` files
under `cmd/` or `pkg/`).

## Library packages vs. application binaries

Go in Debian has two shapes:

- **Library package** (`golang-<importpath>-dev`): ships
  `.go` files under `/usr/share/gocode/src/<importpath>/` for
  other Go packages to compile against. Architecture: `all`
  (the source is the artefact).
- **Application binary** (`<command>`): produces a compiled
  binary in `/usr/bin/`. Architecture: `any`.

Both shapes use the same `dh $@ --buildsystem=golang`
invocation. `rules.golang.tmpl` renders that minimal form;
overrides are added when needed.

## Initial packaging (comparison run only)

`dh-make-golang` can generate a starter `debian/` from an
import path. **Do not ship `dh-make-golang` output directly.**
Use it as a comparison run, the same way debaid treats
`dh_make` output:

```
mkdir /tmp/dhmg && cd /tmp/dhmg
dh-make-golang make github.com/<owner>/<repo>
```

Compare its choices against debaid's output, then write the
actual `debian/` per house-style.

## Package naming

- Library packages: `golang-<importpath>-dev` with `/`
  replaced by `-` and `.` replaced by `-`.
  Example: `github.com/spf13/cobra` →
  `golang-github-spf13-cobra-dev`.
- Application packages: name after the produced command, not
  the import path. Example: `golang.org/x/tools/cmd/gopls` →
  `gopls`, not `golang-golang-x-tools-cmd-gopls-dev`.
- Documentation packages: `golang-<importpath>-doc` when
  upstream ships godoc-extra content worth shipping separately.

## File layout

- Library `.go` sources:
  `/usr/share/gocode/src/<importpath>/`.
- Application binary: `/usr/bin/<command>`.
- Vendored dependencies: do **not** ship them. Strip
  `vendor/` before build (`DH_GOLANG_EXCLUDES`). The Debian
  build uses the system Go and Debian-packaged `golang-*-dev`
  deps.

## debian/control essentials

Build-Depends template:

```
Build-Depends:
 debhelper-compat (= 13),
 dh-sequence-golang,
 golang-any,
```

Add `golang-<importpath>-dev` lines for each direct Go module
dependency that has a Debian package. For dependencies without
a Debian package yet, list them in the bootstrap follow-up
block — Go packages can't be built against an incomplete
dependency graph.

Source stanza must carry the import path so dh-golang knows
where to install sources:

```
XS-Go-Import-Path: github.com/<owner>/<repo>
```

Binary stanza for a library:

```
Package: golang-github-owner-repo-dev
Architecture: all
Depends:
 ${misc:Depends},
 golang-<dep>-dev,
Description: …
```

Binary stanza for an application:

```
Package: foo
Architecture: any
Built-Using: ${misc:Built-Using}
Depends:
 ${shlibs:Depends},
 ${misc:Depends},
Description: …
```

`Built-Using:` records which `golang-*-dev` packages
contributed source at build time. Standard for Go application
binaries — Go statically links the dependency graph and the
field aids archive-wide tracing.

## debian/rules

`rules.golang.tmpl` produces:

```makefile
#!/usr/bin/make -f
export DEB_BUILD_MAINT_OPTIONS = hardening=+all
%:
	dh $@ --buildsystem=golang
```

Older packages used `dh $@ --with=golang` plus a plain
`Build-Depends: dh-golang`. With `dh-sequence-golang` in
Build-Depends the `--with=golang` is redundant; prefer the
modern form on new packages.

Common overrides — only add when needed:

- `export DH_GOLANG_INSTALL_EXTRA = README.md LICENSE docs/`
  — install non-`.go` files alongside the sources. Useful for
  testdata directories that the package's tests need at
  runtime.
- `export DH_GOLANG_INSTALL_ALL = 1` — install all files, not
  just `.go`. Use sparingly; usually
  `DH_GOLANG_INSTALL_EXTRA` is enough.
- `export DH_GOLANG_EXCLUDES = vendor/` — strip vendored
  deps. Always set when upstream ships a `vendor/` directory.
- `override_dh_auto_test:` — Go's test suite often needs
  network or fixtures; skip selectively or pass
  `-short` via `dh_auto_test -- -short`.

## Versioning

- Tagged upstream (`v1.2.3`): standard `1.2.3-1`. Strip the
  `v` prefix via uscan mangling.
- Versionless upstream (HEAD-only project): use
  `0.0~git<date>.<hash>-1`. Example
  `0.0~git20240315.abc1234-1`. Date is the commit timestamp;
  hash is short-form (typically 7 chars). Bump the second `0`
  for significant API changes.
- Pre-releases (`v1.2.3-rc1`): mangle to `1.2.3~rc1-1` via
  `Uversionmangle: s/-(rc|alpha|beta)/~$1/`.

## debian/watch (v5 with github template)

Most Go upstreams release on GitHub:

```
version=5
Template: github
URL: https://github.com/<owner>/<repo>
```

For GitLab-hosted upstreams use `Template: gitlab`. For
upstreams that don't tag releases (HEAD-only), `debian/watch`
is impractical — document the situation in
`debian/README.source` and rely on manual bumps, or use a
custom v5 `mode=git` pattern.

## autopkgtest

**No autodep8 generator exists for Go.** Hand-roll
`debian/tests/` per the autopkgtest worker's normal flow:

- For application binaries: `<bin> --version` /
  `<bin> --help` is usually sufficient.
- For library packages: a `go test ./...` invocation against
  the installed sources. Test scripts typically need
  `golang-any` and the package itself in `Depends:`.

## Common refresh checks

The refresh skill applies these when `source.language == "go"`:

- `dh-sequence-golang` present in `Build-Depends` (not the
  legacy `dh-golang` Build-Depends paired with
  `dh --with golang` in `debian/rules`).
- `golang-any` present in `Build-Depends`.
- `XS-Go-Import-Path:` present in the source stanza.
- `--buildsystem=golang` used in `debian/rules`.
- `Built-Using: ${misc:Built-Using}` present in application
  binary stanzas.
- `debian/watch` is v5 with `Template: github` (or `gitlab`)
  for tag-based releases.
- No `vendor/` directory in the source tarball, or stripped
  via `DH_GOLANG_EXCLUDES`.

## Bail-out conditions

In addition to the worker-level bail-outs:

- A direct Go module dependency has no Debian package — list
  the missing modules; the maintainer needs to package them
  first.
- Upstream uses Go submodules (multiple `go.mod` files in
  subdirectories) and the maintainer wants to package only
  one subtree — ask which import path to use as
  `XS-Go-Import-Path`.
- Upstream relies on cgo for a C library not in Debian —
  bail with the C-library list.

## Notes for Golang package maintainer (not to be used by the LLM)

- Check XS-Go-Import-Path field name (this is the dh-golang convention I'm
  aware of, but the field name has had variations).
- Check /usr/share/gocode/src/<importpath>/ install path (so far it's the
  standard)
- Built-Using: ${misc:Built-Using} recommendation (still standard for Go
  application binaries last I knew).
- dh-sequence-golang virtual provided by dh-golang (vs. a standalone package?).
