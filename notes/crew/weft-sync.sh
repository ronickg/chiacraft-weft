#!/usr/bin/env bash
# During a Weft merge: settle the conflicted engine files the ledger says to take from upstream.
#
#   git merge upstream/master            # stops with conflicts
#   notes/crew/weft-sync.sh              # dry run: what it would do with each conflicted file
#   notes/crew/weft-sync.sh --apply      # do it: take upstream's side and stage those files
#
# For every conflicted file under crates/ and backends/ it reads the file's **Merge** column in
# section 12 of notes/ENGINE_CHANGES.md:
#   - no line in the index, "take upstream's" or "drop"   -> upstream's side (git checkout --theirs,
#     or git rm when upstream deleted it), staged
#   - "keep in engine"                                     -> left conflicted, listed with its rows:
#     start from upstream's file and put those rows back
#   - "move out"                                           -> left conflicted, listed: the hook it
#     needs is in section 11; until that hook exists, resolve it as "keep in engine"
# Conflicts outside crates/ and backends/ (mods, docs, assets of ours) are only listed.
# It never commits, never resolves a file the ledger keeps, and touches nothing outside the merge.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

apply=0
[ "${1:-}" = "--apply" ] && apply=1
ledger=notes/ENGINE_CHANGES.md

git rev-parse -q --verify MERGE_HEAD >/dev/null || { echo "No merge in progress (run git merge upstream/master first)."; exit 1; }

# section 12 only: "| `path` | mark | +/- | upstream | Merge | Rows |"
index=$(awk '/^## 12\./{on=1; next} /^## /{on=0} on' "$ledger")

verdict_of() {
  awk -F'|' -v p="\`$1\`" '{ f=$2; gsub(/^ +| +$/, "", f); if (f == p) { v=$6; gsub(/^ +| +$/, "", v); print v; exit } }' <<<"$index"
}
rows_of() {
  awk -F'|' -v p="\`$1\`" '{ f=$2; gsub(/^ +| +$/, "", f); if (f == p) { r=$7; gsub(/^ +| +$/, "", r); print r; exit } }' <<<"$index"
}

taken=(); keep=(); move=(); other=()
while read -r f; do
  [ -z "$f" ] && continue
  case "$f" in
    crates/*|backends/*) ;;
    *) other+=("$f"); continue ;;
  esac
  v=$(verdict_of "$f")
  case "$v" in
    ""|"take upstream's"|"drop")
      taken+=("$f  (${v:-no row})")
      if [ $apply = 1 ]; then
        if git ls-files -u -- "$f" | awk '$3 == 3' | grep -q .; then
          git checkout --theirs -- "$f" && git add -- "$f"
        else
          git rm -q -- "$f"               # upstream deleted it
        fi
      fi ;;
    "keep in engine") keep+=("$f  <- $(rows_of "$f")") ;;
    "move out") move+=("$f  <- $(rows_of "$f")") ;;
    *) keep+=("$f  (unknown verdict '$v': resolve by hand)") ;;
  esac
done < <(git diff --name-only --diff-filter=U)

show() { local t=$1; shift; [ $# -gt 0 ] && { echo "$t ($#):"; printf '  %s\n' "$@"; echo; }; return 0; }
if [ $apply = 1 ]; then show "Took upstream's and staged" ${taken[@]+"${taken[@]}"}; else show "Would take upstream's (--apply to do it)" ${taken[@]+"${taken[@]}"}; fi
show "Keep in engine: start from upstream's file, put these rows back" ${keep[@]+"${keep[@]}"}
show "Move out: no hook yet, so resolve as keep in engine (rows)" ${move[@]+"${move[@]}"}
show "Outside the engine (mods, docs, art): by the skill's rules" ${other[@]+"${other[@]}"}
left=$(git diff --name-only --diff-filter=U | wc -l | tr -d ' ')
echo "$left files still conflicted. When done: notes/crew/engine-diff.sh must list no file without a row."
