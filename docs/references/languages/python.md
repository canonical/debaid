# Python — debaid overlay

Debian Python packaging conventions and the debaid defaults
for them. Read alongside `${CLAUDE_PLUGIN_ROOT}/docs/house-style.md`;
this file only documents what is *Python-specific*.

Authoritative upstream sources:

- Debian Python Policy:
  https://www.debian.org/doc/packaging-manuals/python-policy/
- `man pybuild`, `man dh_python3`.
- debian-python@lists.debian.org, `#debian-python` on OFTC.

## When this overlay applies

`source.language == "python"` in the shared context. Detected by
the presence of `pyproject.toml`, `setup.py`, or `setup.cfg` at
the source root.

## Interpreter and shebangs

- Python 3 only. Python 2 is removed from Debian since bookworm.
- `#!/usr/bin/python3` — recommended shebang for scripts shipped in the
  package.
- `#!/usr/bin/python3.Y` — only when the script genuinely
  requires a specific minor version.
- **Never** `#!/usr/bin/env python` as it messes with Debian dependency
  resolution.
- **Never** `#!/usr/bin/python` — removed in bookworm.

## Package naming

- Library packages: `python3-<name>` where `<name>` is the
  *import* name (the thing you write in `import …`), **not** the
  distribution name on PyPI.
  Example: `pyyaml` on PyPI → `import yaml` → Debian package
  `python3-yaml`.
- Lowercase. Replace underscores with hyphens.
  Example: `distro_info` → `python3-distro-info`.
- Sub-packages: `python3-foo.bar` for `import foo.bar`. Rare;
  only split when the upstream layout justifies it.
- Documentation packages: `python-<name>-doc`. Some `python3-<name>-doc`
  package might exist, but for new packages, no `3` for doc packages.
  `Architecture: all`, `Section: doc`. 
- Application packages (a CLI tool that happens to be written in
  Python): name after the command, not the module — e.g.
  `youtube-dl`, not `python3-youtube-dl`.

## File layout

- Public modules: `/usr/lib/python3/dist-packages/<module>/`.
- Private modules (applications that are not meant to be
  imported by other Python code): `/usr/share/<package>/` for
  pure Python, `/usr/lib/<package>/` for code with C extensions.
- Never ship to `/usr/local/…`; packages should never deploy anything there.
- Never ship `.pyc` or `.pyo` files. `dh_python3` handles
  byte-compilation via `postinst` and `prerm` maintainer scripts.

## debian/control essentials

Build-Depends template for a pure-Python source:

```
Build-Depends:
 debhelper-compat (= 13),
 dh-sequence-python3,
 python3-all,
 pybuild-plugin-pyproject | python3-setuptools,
 python3-build,
 python3-installer,
```

Add `python3-wheel` only when the build genuinely needs the
wheel tool. For C extensions, replace `python3-all` with
`python3-all-dev`.

If the package requires a Python minor version beyond the
current Debian default, add `X-Python3-Version: >= 3.Y` to the
source stanza. Do not add it when the requirement matches the
current default — every package would otherwise drift over time.

Binary stanza for a library:

```
Package: python3-foo
Architecture: all
Depends:
 ${python3:Depends},
 ${misc:Depends},
Description: …
```

`Architecture: any` only when the package contains C extensions.
Otherwise `all`.

## debian/rules

Minimum viable `debian/rules` for pybuild:

```makefile
#!/usr/bin/make -f
export PYBUILD_NAME = foo
%:
	dh $@ --buildsystem=pybuild
```

`PYBUILD_NAME` is the upstream module name (the thing you
import), not the source-package name.

Common overrides — only add when needed; do not preemptively
clutter `debian/rules`:

- `export PYBUILD_DISABLE = test` — disable tests entirely (last
  resort; document why with an inline comment).
- `export PYBUILD_TEST_ARGS = -k "not test_network"` — narrow
  which tests pybuild runs (pytest selector syntax).
- `export PYBUILD_BEFORE_TEST = …` / `PYBUILD_AFTER_INSTALL = …`
  — hooks. Use sparingly; the recipe should be reproducible
  from reading `debian/rules`.

pybuild blocks outbound network during the build. Build failures with
`ConnectionError` or similar usually mean a test is reaching for the internet —
fix the test, not the proxy.

## debian/watch (v5 with pypi/github/… template)

Note: if more than one source provider (github, pypi, gitlab) do exist, the
template used should be asked to the maintainer if no watch file is shipped.

For PyPI-hosted upstreams, use the `pypi` template — see
`${CLAUDE_PLUGIN_ROOT}/docs/references/watch-v5.md`:

```
version=5
Template: pypi
Dist: foo
```

(Note: pypi template downloads from pypi.debian.net, there is no template for
direct pypi.org download.)

Pre-release mangling: add `Uversionmangle: s/(rc|a|b|c)/~$1/`
if upstream tags pre-releases as `1.0rc1` rather than `1.0~rc1`.

For Python projects released on GitHub rather than PyPI, rely on `Template:
github` and the generic v5 fields.

## autopkgtest

For most Python libraries the autodep8 shortcut is the right
choice:

```
Testsuite: autopkgtest-pkg-python
```

This generates `debian/tests/control` automatically. It runs
`python3 -c "import <name>"` per `python3-*` binary package; the
`<name>` is derived from the package suffix (`python3-foo` →
`import foo`). When the import name diverges from the package
suffix (e.g. `python3-PIL` exports `import PIL` while the
package is conventionally lowercased), set `X-Python3-Module:`
in the binary stanza.

Hand-roll `debian/tests/` instead of using autodep8 when:

- The test must exercise CLI entry points, not just imports.
- The test needs fixture data not part of the upstream test
  suite.
- The package has non-standard `entry_points` not installed
  under `/usr/bin/`.

In those cases, a single `pytest -v` via `Test-Command:` plus
`Depends: @, python3-pytest, …` is usually enough.

## Wheels and pip

- Do not ship wheels. The Debian source-format is `3.0 (quilt)`;
  upstream wheels are not the preferred form for modification.
- Narrow exceptions exist for `pip`, `virtualenv`, `pyvenv`
  themselves, which bootstrap the ecosystem.
- Never invoke `pip install` from `debian/rules` or from
  autopkgtest scripts. The package under test is the installed
  Debian package, not whatever `pip` would fetch.

## Sphinx documentation

When upstream ships Sphinx docs and the maintainer wants a
`-doc` package:

```makefile
override_dh_auto_build:
	dh_auto_build
	PYTHONPATH=. python3 -m sphinx -b html docs/ \
	    debian/python-foo-doc/usr/share/doc/python-foo-doc/html
```

Build-Depends: add `python3-sphinx` (and any theme packages
upstream requires). The doc binary is `python-foo-doc` (no `3`),
`Architecture: all`, `Section: doc`.

## Common refresh checks

The refresh skill applies these when `source.language == "python"`:

- `dh-sequence-python3` present in `Build-Depends` (not the
  legacy `python3` Build-Depends paired with `dh --with python3`).
- Legacy `X-Python-Version` removed; `X-Python3-Version` kept
  only when there is an active minimum-version requirement.
- `debian/watch` is v5 with `Template: pypi` for PyPI-hosted
  upstreams.
- `Testsuite: autopkgtest-pkg-python` present when the package
  is a library and no hand-rolled `debian/tests/` exists.
- No `python` / `python-dev` / `python-minimal` Build-Depends
  (Python 2 residue from before bookworm).
- `${python3:Depends}` substvar present in every binary stanza.

## Iteration budget and pybuild

pybuild runs the upstream test suite across each supported
Python version. When a test is flaky, the cost compounds. Use
the iteration-budget envelope (see
`${CLAUDE_PLUGIN_ROOT}/shared-context.md` § "Iteration-budget
envelope") to cap retries; if a test is flaky, narrow it via
`PYBUILD_TEST_ARGS` rather than retrying the whole build.

## Bail-out conditions

In addition to the worker-level bail-outs:

- Upstream ships a `setup.py` with non-trivial logic that
  pybuild cannot drive (custom commands, monkey-patches to
  setuptools, network in `setup.py` itself).
- Upstream has C extensions with `setup.py build_ext`
  intricacies that pybuild misclassifies. The maintainer may
  need to override `dh_auto_build`.
- Upstream depends on a Python module that has no Debian
  package and is not trivially packageable. Bail with the
  module list.
