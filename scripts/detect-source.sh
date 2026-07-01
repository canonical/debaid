#!/bin/bash
# detect-source.sh — emit JSON describing a source tree for debaid workers.
#
# Usage: detect-source.sh [PATH]
#   PATH defaults to the current working directory.
#
# Output: a single JSON object on stdout matching shared-context.md's
# "source" sub-object.
#
# Requires: jq (used for safe JSON construction).

set -euo pipefail

if ! command -v jq >/dev/null 2>&1; then
  printf 'detect-source.sh: jq is required but not installed\n' >&2
  exit 2
fi

ROOT="${1:-$PWD}"
ROOT="$(cd "$ROOT" && pwd)"

have() { [[ -e "$ROOT/$1" ]]; }
have_glob() { compgen -G "$ROOT/$1" >/dev/null 2>&1; }

# has_files — recursive (depth-limited) name check via find.
# Predictable regardless of nullglob/globstar/failglob.
has_files() {
  find "$ROOT" -maxdepth 3 -type f -name "$1" -print -quit 2>/dev/null \
    | grep -q .
}

# --- language + build system -------------------------------------------------

language="unknown"
build_system="unknown"

if have go.mod; then
  language="go"; build_system="go-mod"
elif have Cargo.toml; then
  language="rust"; build_system="cargo"
elif have pyproject.toml; then
  language="python"; build_system="pyproject"
elif have setup.py || have setup.cfg; then
  language="python"; build_system="setuptools"
elif have meson.build; then
  build_system="meson"
elif have CMakeLists.txt; then
  build_system="cmake"
elif have configure.ac || have configure.in; then
  build_system="autotools"
elif have configure; then
  build_system="autotools"
elif have package.json; then
  language="nodejs"; build_system="nodejs"
elif have Makefile.PL || have Build.PL; then
  language="perl"; build_system="make"
elif have Gemfile; then
  language="ruby"; build_system="make"
elif have_glob "*.cabal" || have stack.yaml; then
  language="haskell"; build_system="make"
elif have Makefile || have GNUmakefile; then
  build_system="make"
fi

# Refine language from build-system hints by scanning for source files.
if [[ "$language" == "unknown" ]]; then
  case "$build_system" in
    autotools|cmake|meson|make)
      if   has_files '*.c';   then language="c"
      elif has_files '*.cpp'; then language="cpp"
      elif has_files '*.cc';  then language="cpp"
      elif has_files '*.cxx'; then language="cpp"
      fi
      ;;
  esac
fi

# --- debian/ state -----------------------------------------------------------

has_debian_dir=false
has_quilt_patches=false
if [[ -d "$ROOT/debian" ]]; then
  has_debian_dir=true
  if [[ -s "$ROOT/debian/patches/series" ]]; then
    has_quilt_patches=true
  fi
fi

# --- Ubuntu delta ------------------------------------------------------------
#
# null  : not an Ubuntu package (caller will set to null when distro=debian).
# true  : changelog top entry shows an Ubuntu revision (e.g. -2ubuntu1).
# false : changelog top entry is bare Debian or a buildN no-change rebuild.
#
# The orchestrator overwrites this with null when target.distro=debian. We
# always probe so the standalone script output is self-contained.
ubuntu_delta=null
if [[ -r "$ROOT/debian/changelog" ]]; then
  top_version="$(awk 'NR==1 {
    if (match($0, /\(([^)]+)\)/, m)) print m[1]
    exit
  }' "$ROOT/debian/changelog" 2>/dev/null || true)"
  if [[ -n "$top_version" ]]; then
    # ubuntu suffix that is *not* a no-change rebuild (buildN)
    if [[ "$top_version" == *ubuntu* && "$top_version" != *build* ]]; then
      ubuntu_delta=true
    elif [[ "$top_version" == *willsync* ]]; then
      ubuntu_delta=true
    else
      ubuntu_delta=false
    fi
  fi
fi

# --- branch layout + upstream VCS -------------------------------------------

debian_branch_layout="unknown"
upstream_vcs="unknown"
if [[ -d "$ROOT/.git" ]] || git -C "$ROOT" rev-parse --git-dir >/dev/null 2>&1; then
  upstream_vcs="git"
  branches="$(git -C "$ROOT" for-each-ref --format='%(refname:short)' \
              refs/heads refs/remotes 2>/dev/null || true)"
  if grep -qE '(^|/)debian/latest$|(^|/)debian/sid$|(^|/)debian/unstable$' <<<"$branches"; then
    debian_branch_layout="dep14"
  elif grep -qE '(^|/)debian$|(^|/)debian-packaging$' <<<"$branches"; then
    debian_branch_layout="separate-branch"
  elif [[ "$has_debian_dir" == true ]]; then
    debian_branch_layout="monorepo"
  else
    debian_branch_layout="none"
  fi
elif [[ -d "$ROOT/.hg" ]]; then
  upstream_vcs="hg"
elif [[ -d "$ROOT/.svn" ]]; then
  upstream_vcs="svn"
elif find "$ROOT" -maxdepth 1 -type f \
       \( -name '*.tar.gz' -o -name '*.tar.xz' \
       -o -name '*.tar.bz2' -o -name '*.tar.zst' \) \
       -print -quit 2>/dev/null | grep -q .; then
  upstream_vcs="tarball"
else
  upstream_vcs="none"
fi

# --- emit --------------------------------------------------------------------

jq -n \
  --arg path "$ROOT" \
  --arg language "$language" \
  --arg build_system "$build_system" \
  --argjson has_debian_dir "$has_debian_dir" \
  --argjson has_quilt_patches "$has_quilt_patches" \
  --arg debian_branch_layout "$debian_branch_layout" \
  --arg upstream_vcs "$upstream_vcs" \
  --argjson ubuntu_delta "$ubuntu_delta" \
  '{path: $path,
    language: $language,
    build_system: $build_system,
    has_debian_dir: $has_debian_dir,
    has_quilt_patches: $has_quilt_patches,
    debian_branch_layout: $debian_branch_layout,
    upstream_vcs: $upstream_vcs,
    ubuntu_delta: $ubuntu_delta}'
