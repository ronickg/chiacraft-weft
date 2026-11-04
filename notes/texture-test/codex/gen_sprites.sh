#!/bin/sh
# Four block textures in one Codex image: ./gen_grid.sh NAME "top left" "top right" "bottom left" "bottom right"
set -e
cd "$(dirname "$0")"
name=$1
[ -f "$name.png" ] && { echo "$name.png exists"; exit 0; }
prompt=$(python3 -c "import sys;t=open('sprite_common.txt').read();print(t.replace('NAME',sys.argv[1]).replace('TL',sys.argv[2]).replace('TR',sys.argv[3]).replace('BL',sys.argv[4]).replace('BR',sys.argv[5]))" "$@")
/Applications/Codex.app/Contents/Resources/codex-cli/bin/codex exec --skip-git-repo-check --sandbox workspace-write --ephemeral -i ../palette.png -i approved_blocks.png -o "last_$name.txt" "$prompt" < /dev/null > "run_$name.log" 2>&1 || true
if [ -f "$name.png" ]; then echo "$name ok"; else echo "$name FAILED: $(grep -o '"message":"[^"]*"' "run_$name.log" | head -1)"; exit 1; fi
