#!/bin/zsh
# archive.sh <dir> <marker> <label> [log...]: copies shots under <dir> newer than the marker (shot
# folders only, not caches) and the logs into Ronald's archive. Never deletes anything.
DIR=$1 MARK=$2 LABEL=$3; shift 3
A=$HOME/Coding/chiacraft-archive/images/tester/$(date +%F)/$LABEL-$(date +%H%M%S); mkdir -p $A
find $DIR -name '*.png' -newer $MARK \( -path '*screenshots*' -o -path '*shots*' \) -not -path '*/target*' -not -path '*/cache/*' -not -path '*/chiacraft-archive/images/*' 2>/dev/null | while read f; do
  rel=${f#$DIR/}; mkdir -p $A/${rel:h}; cp -p "$f" "$A/$rel"; done
for l in "$@"; do cp -p "$l" $A/ 2>/dev/null; done
echo "archived $(find $A -name '*.png' | wc -l | tr -d ' ') shots to $A"
