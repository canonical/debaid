#!/bin/bash
# verify.sh — build + lintian snapshot for worker iteration loops.
#
# Workers call this between attempts to check progress. The decision
# logic (continue / bail-out / declare success) lives in the worker
# LLM, which reads the JSON snapshot and the iteration-budget envelope
# from shared-context.md.
#
# Usage:
#   verify.sh [--builder=sbuild|dpkg-buildpackage] [--no-build] [PATH]
#
#   PATH      source tree (must contain debian/). Default: CWD.
#   --builder pick a builder explicitly. Default: sbuild if available,
#             else dpkg-buildpackage.
#   --no-build skip the build and run lintian against whatever
#              .changes/.dsc already exists alongside the source tree.
#
# Output: a single JSON object on stdout. Its shape is defined by
# `VerifySnapshot` in src/model/verify.rs; shared-context.md
# § "Verify snapshot semantics" explains what the fields mean.
#
# This script is stateless. Workers compute progress (e.g.
# "same_class_as_previous") by comparing two consecutive snapshots
# themselves — don't add caching here.
#
# Exit code: 0 on a successful snapshot (regardless of build/lint
# pass-or-fail — the worker decides what to do). Non-zero only on
# input errors (missing debian/, missing jq/lintian, bad args).
#
# Requires: jq. Strongly recommended: lintian, sbuild (or
# dpkg-buildpackage).

set -euo pipefail

if ! command -v jq >/dev/null 2>&1; then
  printf 'verify.sh: jq is required but not installed\n' >&2
  exit 2
fi

BUILDER=""
DO_BUILD=1
ROOT=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --builder=*) BUILDER="${1#*=}"; shift ;;
    --builder)   BUILDER="${2:?missing value for --builder}"; shift 2 ;;
    --no-build)  DO_BUILD=0; shift ;;
    -h|--help)
      sed -n '2,/^$/p' "$0" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    --)          shift; ROOT="${1:-}"; break ;;
    -*)
      printf 'verify.sh: unknown arg: %s\n' "$1" >&2
      exit 64
      ;;
    *)           ROOT="$1"; shift ;;
  esac
done

ROOT="${ROOT:-$PWD}"
ROOT="$(cd "$ROOT" && pwd)"

if [[ ! -d "$ROOT/debian" ]]; then
  printf 'verify.sh: %s has no debian/ directory\n' "$ROOT" >&2
  exit 2
fi

# --- pick builder -----------------------------------------------------------

if (( DO_BUILD )); then
  if [[ -z "$BUILDER" ]]; then
    if   command -v sbuild >/dev/null 2>&1;            then BUILDER="sbuild"
    elif command -v dpkg-buildpackage >/dev/null 2>&1; then BUILDER="dpkg-buildpackage"
    else                                                    BUILDER="none"
    fi
  fi
else
  BUILDER="none"
fi

# --- run build --------------------------------------------------------------

build_ran=false
build_ok=false
build_exit=0
build_log="$(mktemp -t verify-build.XXXXXX.log)"

if (( DO_BUILD )); then
  build_ran=true
  case "$BUILDER" in
    none)
      build_ran=false
      build_exit=127
      printf 'verify.sh: no builder available (need sbuild or dpkg-buildpackage)\n' \
        >>"$build_log"
      ;;
    sbuild)
      if ( cd "$ROOT" && sbuild --no-arch-all ) >"$build_log" 2>&1; then
        build_ok=true
      else
        build_exit=$?
      fi
      ;;
    dpkg-buildpackage)
      if ( cd "$ROOT" && dpkg-buildpackage -us -uc -b -nc ) >"$build_log" 2>&1; then
        build_ok=true
      else
        build_exit=$?
      fi
      ;;
    *)
      printf 'verify.sh: unknown builder: %s\n' "$BUILDER" >&2
      exit 64
      ;;
  esac
fi

# --- run lintian ------------------------------------------------------------

lintian_ran=false
lintian_scope="none"
lintian_log="$(mktemp -t verify-lintian.XXXXXX.log)"
errors_json='[]'
warnings_json='[]'
infos_json='[]'
pedantics_json='[]'
overrides_applied=0

if command -v lintian >/dev/null 2>&1; then
  # Prefer the latest .changes; fall back to --source on the latest
  # .dsc; fall back to running against the source tree.
  # Workers should weight "clean" results less when scope=source-tree
  # — binary-only checks don't fire without a built .deb.
  parent="$(cd "$ROOT/.." && pwd)"
  changes="$(ls -1t "$parent"/*.changes 2>/dev/null | head -n1 || true)"
  dsc="$(ls -1t "$parent"/*.dsc 2>/dev/null | head -n1 || true)"

  if [[ -n "$changes" ]]; then
    lintian -EvIL +pedantic --no-tag-display-limit "$changes" \
      >"$lintian_log" 2>&1 || true
    lintian_ran=true
    lintian_scope="changes"
  elif [[ -n "$dsc" ]]; then
    lintian -EvIL +pedantic --no-tag-display-limit --source "$dsc" \
      >"$lintian_log" 2>&1 || true
    lintian_ran=true
    lintian_scope="dsc"
  else
    ( cd "$ROOT" && lintian -EvIL +pedantic --no-tag-display-limit ) \
      >"$lintian_log" 2>&1 || true
    lintian_ran=true
    lintian_scope="source-tree"
  fi

  list_tags() {
    # Lintian severity lines: "E: pkg [scope]: tag context".
    # grep returns 1 on no match — wrap to keep pipefail happy.
    local sev="$1"
    { grep -E "^${sev}:" "$lintian_log" 2>/dev/null || true; } \
      | awk -F': ' '{print $3}' \
      | awk '{print $1}' \
      | jq -R -s 'split("\n") | map(select(length > 0))'
  }

  errors_json="$(list_tags E)"
  warnings_json="$(list_tags W)"
  infos_json="$(list_tags I)"
  pedantics_json="$(list_tags P)"
  overrides_applied="$(awk '/^N: Overridden:/ {c++} END {print c+0}' "$lintian_log" 2>/dev/null)"
  : "${overrides_applied:=0}"
fi

# --- diff size --------------------------------------------------------------

diff_size_lines='null'
if git -C "$ROOT" rev-parse --git-dir >/dev/null 2>&1; then
  n="$(git -C "$ROOT" diff --numstat -- debian/ 2>/dev/null \
       | awk '{s += $1 + $2} END {print s+0}')"
  diff_size_lines="$n"
fi

# --- emit -------------------------------------------------------------------

jq -n \
  --arg builder "$BUILDER" \
  --argjson build_ran "$build_ran" \
  --argjson build_ok "$build_ok" \
  --arg build_log "$build_log" \
  --argjson build_exit "$build_exit" \
  --argjson lintian_ran "$lintian_ran" \
  --arg lintian_scope "$lintian_scope" \
  --arg lintian_log "$lintian_log" \
  --argjson errors "$errors_json" \
  --argjson warnings "$warnings_json" \
  --argjson infos "$infos_json" \
  --argjson pedantics "$pedantics_json" \
  --argjson overrides_applied "$overrides_applied" \
  --argjson diff_size_lines "$diff_size_lines" \
  '{
    build: {
      tool: $builder,
      ran: $build_ran,
      ok: $build_ok,
      log_path: $build_log,
      exit_code: $build_exit
    },
    lintian: {
      ran: $lintian_ran,
      scope: $lintian_scope,
      log_path: $lintian_log,
      errors: $errors,
      warnings: $warnings,
      infos: $infos,
      pedantics: $pedantics,
      overrides_applied: $overrides_applied
    },
    diff_size_lines: $diff_size_lines
  }'
