#!/bin/bash
# verify.sh — drive debaid fixtures end-to-end.
#
# Walks tests/fixtures/* and, for each fixture:
#   - if   fixture/test.sh    exists, executes it.
#   - else                            prints [STUB] and skips.
#
# A fixture's test.sh is the contract: how it invokes a worker and
# how it compares the output against fixture/expected/ is up to the
# fixture author. This runner just dispatches and aggregates.
#
# Exit codes:
#   0  all non-stub fixtures passed
#   1  at least one fixture failed
#   2  --strict and at least one fixture is a stub
#
# Environment:
#   DEBAID_CLAUDE_CMD   command used by fixtures to invoke claude.
#                         Default: "claude --bare --print".
#                         Exported for use by fixture test scripts.

set -euo pipefail

STRICT=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --strict) STRICT=1; shift ;;
    -h|--help)
      sed -n '2,/^$/p' "$0" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    *)
      printf 'verify.sh: unknown argument: %s\n' "$1" >&2
      exit 64
      ;;
  esac
done

HERE="$(cd "$(dirname "$0")" && pwd)"
FIXTURES="$HERE/fixtures"

if [[ ! -d "$FIXTURES" ]]; then
  printf 'verify.sh: %s does not exist\n' "$FIXTURES" >&2
  exit 2
fi

export DEBAID_CLAUDE_CMD="${DEBAID_CLAUDE_CMD:-claude --bare --print}"

pass=0
fail=0
stub=0
fail_names=()

for fixture in "$FIXTURES"/*/; do
  [[ -d "$fixture" ]] || continue
  name="$(basename "$fixture")"
  test_script="$fixture/test.sh"

  if [[ -x "$test_script" ]]; then
    if ( cd "$fixture" && "./test.sh" ); then
      printf '[PASS] %s\n' "$name"
      pass=$((pass + 1))
    else
      printf '[FAIL] %s\n' "$name"
      fail=$((fail + 1))
      fail_names+=("$name")
    fi
  else
    printf '[STUB] %s: no executable test.sh, skipping\n' "$name"
    stub=$((stub + 1))
  fi
done

printf '\n'
printf 'verify.sh: %d passed, %d failed, %d stubs\n' "$pass" "$fail" "$stub"

if (( fail > 0 )); then
  printf 'failures:\n'
  for n in "${fail_names[@]}"; do printf '  - %s\n' "$n"; done
  exit 1
fi

if (( STRICT == 1 && stub > 0 )); then
  printf 'verify.sh: --strict and %d stub(s) remain\n' "$stub" >&2
  exit 2
fi

exit 0
