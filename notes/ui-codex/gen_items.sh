#!/bin/sh
# One sheet of item icons with Codex: ./gen_items.sh NAME "request" (the approved item sheets attached)
set -e
cd "$(dirname "$0")"
name=$1
[ -f "$name.png" ] && { echo "$name.png exists"; exit 0; }
prompt=$(python3 -c "import sys;t=open('items_style.txt').read();print(t.replace('NAME',sys.argv[1]).replace('REQUEST',sys.argv[2]))" "$1" "$2")
r=../ui-reference
/Applications/Codex.app/Contents/Resources/codex-cli/bin/codex exec --skip-git-repo-check --sandbox workspace-write --ephemeral -i $r/items/materials.png -i $r/items/tools_wood_stone_iron.png -i $r/pixel-fantasy-inventory-ui-reference.jpg -o "last_$name.txt" "$prompt" < /dev/null > "run_$name.log" 2>&1 || true
if [ -f "$name.png" ]; then echo "$name ok"; else echo "$name FAILED: $(grep -o '"message":"[^"]*"' "run_$name.log" | head -1)"; exit 1; fi
