#!/bin/zsh
# sample_windows.sh <run dir> <out prefix> [label...]
# macOS's CPU sampler (/usr/bin/sample: call stacks only, nothing touches the GPU) over a game's
# REC windows. Start it right after the game: it waits for the newest log in <run dir>/logs, and
# for each label, on "REC <label> start" samples the newest `weft` process for 28 s into
# <out prefix>-<label>.sample.txt. Labels default to "stand walk". Then:
#   python3 tools/profile/sample_top.py <out prefix>-stand.sample.txt
set -u
RD=${1:?run dir}; OUT=${2:?out prefix}; shift 2
LABELS=(${@:-stand walk})
for i in {1..120}; do LOG=$(ls -t $RD/logs/*.log 2>/dev/null | head -1); [[ -n $LOG ]] && break; sleep 1; done
[[ -z ${LOG:-} ]] && { echo "no log in $RD/logs"; exit 1; }
echo "log $LOG"
for L in $LABELS; do
  until grep -q "REC $L start" $LOG 2>/dev/null; do sleep 0.5; pgrep -x weft >/dev/null || { echo "game gone before $L"; exit 1; }; done
  PID=$(pgrep -nx weft)
  echo "sampling $L (pid $PID)"
  /usr/bin/sample $PID 28 1 -mayDie -file $OUT-$L.sample.txt >/dev/null 2>&1
done
echo done
