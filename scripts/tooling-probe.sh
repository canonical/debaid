#!/bin/bash
# tooling-probe.sh — emit JSON describing packaging tooling availability.
#
# Usage: tooling-probe.sh
#
# Output: a single JSON object on stdout forming the "tooling"
# sub-object of the context document: an open map keyed by
# executable name; see `Tooling` in src/model/context.rs.
#
# Requires: jq (used for safe JSON construction).

set -euo pipefail

if ! command -v jq >/dev/null 2>&1; then
  printf 'tooling-probe.sh: jq is required but not installed\n' >&2
  exit 2
fi

probe() {
  local cmd="$1"
  local version_flag="${2:---version}"
  if command -v "$cmd" >/dev/null 2>&1; then
    local v
    v="$("$cmd" "$version_flag" 2>&1 | head -n1 \
         | sed -E 's/[^0-9.]*([0-9]+(\.[0-9]+){1,3}).*/\1/' || true)"
    if [[ -z "$v" || "$v" == *" "* ]]; then v="unknown"; fi
    jq -nc --arg v "$v" '{available: true, version: $v}'
  else
    jq -nc '{available: false, version: null}'
  fi
}

cat <<EOF
{
  "sbuild":             $(probe sbuild),
  "pbuilder":           $(probe pbuilder),
  "autopkgtest":        $(probe autopkgtest),
  "lintian":            $(probe lintian),
  "debputy":            $(probe debputy),
  "wrap-and-sort":      $(probe wrap-and-sort),
  "gbp":                $(probe gbp),
  "dh_make":            $(probe dh_make),
  "cme":                $(probe cme),
  "blhc":               $(probe blhc),
  "hardening-check":    $(probe hardening-check),
  "licensecheck":       $(probe licensecheck),
  "decopy":             $(probe decopy),
  "lrc":                $(probe lrc),
  "uscan":              $(probe uscan),
  "dch":                $(probe dch),
  "jq":                 $(probe jq),
  "git-ubuntu":         $(probe git-ubuntu),
  "requestsync":        $(probe requestsync),
  "pull-debian-source": $(probe pull-debian-source),
  "pull-lp-source":     $(probe pull-lp-source),
  "syncpackage":        $(probe syncpackage),
  "update-maintainer":  $(probe update-maintainer),
  "mk-sbuild":          $(probe mk-sbuild),
  "reportbug":          $(probe reportbug)
}
EOF
