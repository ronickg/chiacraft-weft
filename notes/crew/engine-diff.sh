#!/usr/bin/env bash
# Our engine diff against Weft, file by file, checked against the ledger (notes/ENGINE_CHANGES.md).
#
#   notes/crew/engine-diff.sh               # against upstream/master's merge base
#   notes/crew/engine-diff.sh <ref>         # against another upstream ref
#   notes/crew/engine-diff.sh --missing     # only the checks, no per-file list
#
# Prints every file under crates/ and backends/ that differs from the merge base: how it
# changed, +/- lines, how many upstream commits since the base touched it (more than 0: expect
# a conflict at the next merge), and NO ROW when the ledger doesn't name it. A file counts as
# named when the ledger has it in backticks (`crates/engine/src/app.rs`), or its whole crate
# (`crates/wallet/`) for our own crates. Then it lists ledger paths that no longer differ
# (stale rows). Exits 1 when a file has no row, so a sync can't miss one.
# Read-only: it runs git only.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

only_checks=0
up=upstream/master
for a in "$@"; do
  case "$a" in
    --missing) only_checks=1 ;;
    -h|--help) sed -n '2,15p' "$0"; exit 0 ;;
    *) up=$a ;;
  esac
done

ledger=notes/ENGINE_CHANGES.md
base=$(git merge-base HEAD "$up")
echo "Merge base with $up: $(git rev-parse --short "$base") ($(git log -1 --format=%cs "$base"))"
echo "Ours: $(git diff --shortstat "$base" HEAD -- crates/ backends/)"
echo

# Every backticked path in the ledger, once.
named=$(grep -oE '`(crates|backends)/[^`]+`' "$ledger" | tr -d '`' | sort -u)

has_row() {
  local f=$1
  grep -qxF "$f" <<<"$named" && return 0
  # our own crates are named as a whole (`crates/wallet/`)
  local crate
  crate=$(cut -d/ -f1-2 <<<"$f")/
  grep -qxF "$crate" <<<"$named"
}

missing=()
changed=()
[ $only_checks = 1 ] || printf '%-3s %7s %7s %4s  %s\n' "" "+" "-" "up" "file"
while IFS=$'\t' read -r st f; do
  changed+=("$f")
  read -r a d _ < <(git diff --no-renames --numstat "$base" HEAD -- "$f")
  if has_row "$f"; then flag=""; else flag="  NO ROW"; missing+=("$f"); fi
  if [ $only_checks = 0 ]; then
    n=$(git rev-list --count "$base..$up" -- "$f")
    printf '%-3s %7s %7s %4s  %s%s\n' "$st" "$a" "$d" "$n" "$f" "$flag"
  fi
done < <(git diff --no-renames --name-status "$base" HEAD -- crates/ backends/)

echo
stale=()
while read -r p; do
  [ -z "$p" ] && continue
  case "$p" in */) continue ;; esac            # a whole crate
  # only real files: not a directory, a pattern (destroy_0-9.png) or a path that never existed
  [ "$(git cat-file -t "HEAD:$p" 2>/dev/null || git cat-file -t "$base:$p" 2>/dev/null)" = blob ] || continue
  printf '%s\n' "${changed[@]}" | grep -qxF "$p" || stale+=("$p")
done <<<"$named"

if [ ${#stale[@]} -gt 0 ]; then
  echo "Ledger paths that no longer differ from Weft (${#stale[@]}): drop or fix their rows"
  printf '  %s\n' "${stale[@]}"
  echo
fi
if [ ${#missing[@]} -gt 0 ]; then
  echo "Files with no ledger row (${#missing[@]}): add a row, or take upstream's at the merge"
  printf '  %s\n' "${missing[@]}"
  exit 1
fi
echo "Every changed engine file has a ledger row (${#changed[@]} files)."
