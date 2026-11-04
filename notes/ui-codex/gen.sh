#!/bin/sh
# One generated UI image with Codex (game-art skill): ./gen.sh NAME request.txt [more reference images]
# Always attached: the approved kit parts (the style key) and the two mockups of the HUD and screens.
set -e
cd "$(dirname "$0")"
name=$1; req=$2; shift 2
[ -f "$name.png" ] && { echo "$name.png exists"; exit 0; }
codex=/Applications/Codex.app/Contents/Resources/codex-cli/bin/codex
ref=../ui-reference
prompt=$(python3 -c "import sys;t=open('style.txt').read();print(t.replace('NAME',sys.argv[1]).replace('REQUEST',open(sys.argv[2]).read()))" "$name" "$req")
imgs="-i approved_parts.png -i $ref/screens-overview-mockup.jpg -i $ref/hud-in-game-mockup.jpg"
for extra in "$@"; do imgs="$imgs -i $extra"; done
$codex exec --skip-git-repo-check --sandbox workspace-write --ephemeral $imgs -o "last_$name.txt" "$prompt" < /dev/null > "run_$name.log" 2>&1 || true
if [ -f "$name.png" ]; then echo "$name ok"; else echo "$name FAILED: $(grep -o '"message":"[^"]*"' "run_$name.log" | head -1)"; exit 1; fi
