#!/bin/bash
# Push branches to the public GitHub repo with others' reference images left out of every commit.
#
# Ronald, 2026-11-22: the repo went public, with third-party screenshots (Bliss, Complementary,
# BSL, Photon, Voxy) removed from history first. Local branches keep them; only what goes to
# GitHub is filtered. git filter-repo is deterministic, so the same local commits always give the
# same public hashes, and later pushes fast-forward.
#
# Usage: notes/crew/push-public.sh <local-ref>:<github-branch> ...
#   e.g. notes/crew/push-public.sh dev:dev
# Add --force only for the first rewrite (Ronald's yes, 11-22).
set -euo pipefail
REPO=/Users/ronald/Coding/chiacraft-weft
FORCE=""
SPECS=()
for a in "$@"; do
  if [ "$a" = "--force" ]; then FORCE="--force"; else SPECS+=("$a"); fi
done
[ ${#SPECS[@]} -gt 0 ] || { echo "usage: $0 [--force] <local-ref>:<branch> ..."; exit 1; }

WORK=$(mktemp -d "${TMPDIR:-/tmp}/push-public.XXXXXX")
trap 'rm -rf "$WORK"' EXIT
git init -q --bare "$WORK/pub.git"
for s in "${SPECS[@]}"; do
  src=${s%%:*}; dst=${s##*:}
  sha=$(git -C "$REPO" rev-parse "$src^{commit}")
  git -C "$REPO" push -q "$WORK/pub.git" "$sha:refs/heads/$dst"
done

# Others' pictures: kept locally and in ~/Coding/chiacraft-archive, never on GitHub.
git -C "$WORK/pub.git" filter-repo --force --quiet --invert-paths \
  --path reports/water-references/ \
  --path-glob 'reports/*/refs/*'

for s in "${SPECS[@]}"; do
  dst=${s##*:}
  git -C "$WORK/pub.git" push $FORCE https://github.com/ronickg/chiacraft-weft.git "refs/heads/$dst:refs/heads/$dst"
done
