#!/bin/sh
# run.sh NAME request.txt : one codex image with our style refs (game-art UI flow)
cd "$(dirname "$0")"; name=$1; req=$2
codex=/Applications/Codex.app/Contents/Resources/codex-cli/bin/codex
prompt=$(python3 -c "import sys;t=open('../../../../notes/ui-codex/style.txt').read();print(t.replace('NAME',sys.argv[1]).replace('REQUEST',open(sys.argv[2]).read()))" "$name" "$req")
$codex exec --skip-git-repo-check --sandbox workspace-write --ephemeral -i ${REF:-our_effects.png} -i approved_parts.png -i hud-in-game-mockup.jpg -o "last_$name.txt" "$prompt" < /dev/null > "run_$name.log" 2>&1
[ -f "$name.png" ] && echo "$name ok" || echo "$name FAILED"
