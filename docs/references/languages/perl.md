# Perl — debaid overlay

> **⚠️ DRAFT.** Drafted from public Debian Perl Group knowledge
> without input from an active pkg-perl maintainer. Remove this
> marker only after a pkg-perl maintainer has reviewed the
> file. See § "DRAFT marker — likely needs correction" at the
> end for specific bits to scrutinise.

Debian Perl packaging is handled by the Debian Perl Group
(pkg-perl). Most CPAN modules build with a plain `dh $@` —
Perl-specific debhelper handlers ship with `debhelper` itself,
and `dh` auto-detects `Makefile.PL` vs `Build.PL`. Read
alongside `${CLAUDE_PLUGIN_ROOT}/docs/house-style.md`; this
file documents what is *Perl-specific*.

Authoritative upstream sources:

- Debian Perl Group: https://salsa.debian.org/perl-team
- `man dh_perl`, `man dh-make-perl`.
- pkg-perl-maintainers@lists.alioth.debian.org

## When this overlay applies

`source.language == "perl"` in the shared context. Detected by
the presence of `Makefile.PL`, `Build.PL`, `META.json`, or
`META.yml` at the source root.

## Library modules vs. application binaries

Perl in Debian has two shapes:

- **Library module** (`libfoo-bar-perl`): Perl modules under
  `/usr/share/perl5/` (pure Perl) or
  `/usr/lib/<triplet>/perl5/<perlversion>/` (XS).
  `Architecture: all` for pure Perl, `any` for XS.
- **Application** (`<command>`): a Perl program with a
  shebang in `/usr/bin/`. Often packaged as a library plus
  a thin command-line front-end.

Most CPAN distributions are library modules; pure-application
Perl packages are uncommon.

## Initial packaging (comparison run only)

`dh-make-perl --cpan Foo::Bar` generates a starter `debian/`
from CPAN metadata. **Do not ship `dh-make-perl` output
directly.** Use it as a comparison run in `/tmp`, the same
way debaid treats `dh_make` and `dh-make-golang` output:

```
mkdir /tmp/dhmp && cd /tmp/dhmp
dh-make-perl --cpan Foo::Bar
```

Compare its choices against debaid's output, then write
the actual `debian/` per house-style.

## Package naming

- Library packages: `libfoo-bar-perl` for `Foo::Bar`.
  Lowercase, `::` replaced by `-`.
  Example: `Module::Build::Tiny` → `libmodule-build-tiny-perl`.
- Application packages: name after the produced command, not
  the module.
- Documentation packages: `libfoo-bar-perl-doc` only when
  upstream ships substantial non-pod documentation worth
  shipping separately.

## File layout

- Pure-Perl modules: `/usr/share/perl5/Foo/Bar.pm`.
- XS modules: `/usr/lib/<triplet>/perl5/<perlversion>/…`.
- Application scripts: `/usr/bin/<command>` with shebang
  `#!/usr/bin/perl`.
- Manpages: generated from POD by `dh_installman`; placed
  under `/usr/share/man/`.

## debian/control essentials

Build-Depends template for a pure-Perl module:

```
Build-Depends:
 debhelper-compat (= 13),
 perl,
```

Add test-deps actually used by upstream's test suite — common
ones:

```
 libtest-simple-perl,
 libtest-pod-perl,
 libtest-pod-coverage-perl,
```

For `Build.PL`-based distributions:

```
 libmodule-build-tiny-perl,
```

XS modules typically don't need a separate `*-xs-dev`
Build-Depends today — the `perl` package ships the headers.
Check `Makefile.PL` for its `XS` configuration to confirm.

Section: `perl` for library packages.

Binary stanza for a pure-Perl library:

```
Package: libfoo-bar-perl
Section: perl
Architecture: all
Depends:
 ${perl:Depends},
 ${misc:Depends},
Description: …
```

For XS modules: `Architecture: any`, add
`${shlibs:Depends}` to `Depends:`.

## debian/rules

Minimum viable `debian/rules`:

```makefile
#!/usr/bin/make -f
%:
	dh $@
```

`dh` auto-detects whether the upstream uses `Makefile.PL`
(perl_makemaker buildsystem) or `Build.PL` (perl_build
buildsystem). No `--buildsystem` or `--with` flag is needed.

**Hardening for XS modules.** XS modules produce compiled
`.so` files. Add the hardening export manually:

```makefile
export DEB_BUILD_MAINT_OPTIONS = hardening=+all
```

`rules.perl.tmpl` does **not** include this by default
because `source.language == "perl"` does not set
`has_compiled_binaries` in the shared context. The XS case is
a known gap; the maintainer adds the export by hand for XS
packages.

Common overrides — only when needed:

- `override_dh_auto_test:` — skip network tests or upstream's
  author tests:

  ```makefile
  override_dh_auto_test:
  	AUTHOR_TESTING=0 dh_auto_test
  ```

- `override_dh_perl:` — override perl-deps calculation
  (rare; `${perl:Depends}` is usually correct).

## debian/watch (v5 with metacpan template)

For CPAN-tracked modules, use the `metacpan` template — see
`${CLAUDE_PLUGIN_ROOT}/docs/references/watch-v5.md`:

```
version=5
Template: metacpan
Dist: Foo-Bar
```

`Dist:` is the CPAN distribution name (dashes, not
double-colons): `Foo::Bar` → `Foo-Bar`. Some upstreams
release on GitHub before pushing to CPAN — use
`Template: github` and tag-based release detection in that
case.

## autopkgtest

For most Perl modules the autodep8 shortcut is the right
choice:

```
Testsuite: autopkgtest-pkg-perl
```

This generates `debian/tests/control` automatically. It runs
`prove -v` against the upstream test suite using the
installed package — meaningful for distributions whose tests
exercise the module's public API. For XS modules the
generator also verifies the compiled `.so` loads.

Hand-roll `debian/tests/` instead when:

- The package's behaviour-of-interest is a CLI tool, not the
  underlying module.
- Upstream's test suite is unrunnable against the installed
  package (e.g. needs the build tree, fixtures from `xt/`
  author tests).
- The test needs services or network the generator does not
  enable.

## Patches

Standard DEP-3 headers per house-style. For CPAN-tracked
distributions, prefer forwarding upstream via the
distribution's modern issue tracker. CPAN RT is being phased
out — most maintainers now use GitHub or GitLab issues.
Mark `Forwarded:` with the actual URL when filed.

## Common refresh checks

The refresh skill applies these when `source.language ==
"perl"`:

- `debhelper-compat (= 13)` and `perl` present in
  `Build-Depends`.
- No legacy `${perl:Provides}` or hand-maintained
  `Provides: perl5` lines on binary stanzas.
- `${perl:Depends}` substvar present in binary stanzas;
  `${shlibs:Depends}` also present on XS modules.
- `Section: perl` on library binary packages.
- `debian/watch` is v5 with `Template: metacpan` for
  CPAN-tracked modules.
- `Testsuite: autopkgtest-pkg-perl` present on library
  packages without a hand-rolled `debian/tests/`.
- `Architecture: all` on packages with no XS code;
  `Architecture: any` only when XS is present.

## Bail-out conditions

In addition to the worker-level bail-outs:

- A direct Perl module dependency has no Debian package and
  is not trivially packageable — list the missing modules.
- Upstream uses a non-standard build system (not
  `Makefile.PL`, not `Build.PL`) — bail; dh's auto-detection
  will not work.
- XS module requires a C library not packaged for Debian —
  bail with the C-library list.
- Upstream relies on test fixtures from `xt/` or `t/extra/`
  that need network or external services.

## DRAFT marker — likely needs correction

This overlay was drafted from public Debian Perl Group
knowledge without input from an active pkg-perl maintainer.
The bits most likely to be wrong or outdated:

- Test-dep selection — the `libtest-*` package list above is
  illustrative; an actual pkg-perl maintainer should curate
  the canonical Build-Depends set.
- Whether `dh-sequence-perl` or `dh-perl-extra` is a useful
  Build-Depends today (or whether plain `perl` is enough).
- pkg-perl team workflow details (gbp + DEP-14 conventions,
  pkg-perl-tools usage, sponsorship process).
- XS Build-Depends — whether `perl` alone is enough or
  whether older `perl-xs-dev` / similar packages are still
  needed.
- Hardening for XS — should `has_compiled_binaries` in the
  shared-context schema be extended to cover Perl + XS?

Remove the DRAFT marker at the top of this file once a
pkg-perl maintainer has reviewed and corrected the content.
