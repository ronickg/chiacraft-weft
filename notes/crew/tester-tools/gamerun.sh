#!/bin/zsh
# gamerun.sh <dir with target/release/weft> <script> [world] [timeout s]; EXTRA="--size 1280x720 ..."
# One game at a time. Exit status (11-07, after c_tr05_v wedged the GPU and the chain went on, because a TIMEOUT exited 0):
#   0   the game ended on its own
#   2,3,4,7  refused by guard.sh (lock / a game running, a GPU stall on this boot, paused or no binary, too many graphics clients)
#   5   GPU stall: killed and runs/GPU_STALLED written (no Mac runs this boot); also when a killed game would not die
#   6   a lock hang: killed, runs go on (the code, not the GPU)
#   8   TIMEOUT: still running after the timeout, killed (GPU_STALLED too if its log had gone quiet, or it would not die)
#   9   STUCK: the log was quiet for two stall windows although the main thread kept running frames: killed (GPU_STALLED if it would not die)
# A chain must stop on any non-zero status. A log quiet STALL s is sampled by stuck.sh: GPU kills and blocks Mac runs for this boot,
# LOCK kills (code hang), UNKNOWN kills and blocks; ALIVE (frames turning) gets ONE more window, and a second quiet window with
# frames still turning is STUCK (the script or a worker is stuck and nothing is logged). Shots and logs are archived after.
set -u
DIR=${1:?dir} SCRIPT=${2:?script} WORLD=${3:-tk$(date +%H%M%S)} TMO=${4:-600}; STALL=${STALL:-60}
source ${0:A:h}/guard.sh
echo "$WORLD in $DIR since $(date +%T)" > $Q/lock/who
[[ -x $DIR/target/release/weft ]] || { echo "no binary at $DIR/target/release/weft"; exit 4; }
[[ -e $DIR/run/mods/testmod.zip ]] && { mkdir -p $DIR/run/stale-mods; mv $DIR/run/mods/testmod.zip $DIR/run/stale-mods/; echo "moved a stale run/mods/testmod.zip aside"; }
LOG=$Q/$WORLD-$(date +%H%M%S).log; MARK=$Q/.mark-$$; touch $MARK
cd $DIR && rm -rf run/saves/$WORLD
./target/release/weft --world $WORLD --script ${SCRIPT:A} ${=EXTRA:-} > $LOG 2>&1 &
PID=$!; echo "pid $PID, log $LOG"
HERE=${0:A:h}
# Kills the game and waits for it to be gone, never for ever: SIGTERM first (unless $1 is KILL), then SIGKILL; returns 1 if it is still
# there after that (a process stuck in the GPU driver does not die: the machine is wedged).
killgame() {
  local i
  if [[ ${1:-TERM} != KILL ]]; then kill $PID 2>/dev/null; for i in {1..10}; do kill -0 $PID 2>/dev/null || return 0; sleep 1; done; fi
  kill -9 $PID 2>/dev/null; for i in {1..30}; do kill -0 $PID 2>/dev/null || return 0; sleep 1; done
  return 1
}
flag() { { echo $BOOT; echo "$(date +%T) $WORLD in $DIR: $1; last line:"; sed 's/\x1b\[[0-9;]*m//g' $LOG | tail -1; } > $FLAG; }
start=$SECONDS; last=0; lastchange=$SECONDS; strikes=0
while kill -0 $PID 2>/dev/null; do
  sleep 2; size=$(stat -f %z $LOG); if (( size != last )); then last=$size; lastchange=$SECONDS; strikes=0; fi
  if (( SECONDS - lastchange >= STALL )); then
    ST=${LOG%.log}.stack.txt; V=$($HERE/stuck.sh $PID $ST); echo "$(date +%T) quiet ${STALL}s: $V (stack $ST)"
    case $V in
      ALIVE*)
        strikes=$((strikes + 1))
        if (( strikes < 2 )); then echo "# tester $(date +%T): quiet ${STALL}s but $V; one more window" >> $LOG; last=$(stat -f %z $LOG); lastchange=$SECONDS; continue; fi
        echo "# tester $(date +%T): quiet two windows, frames turning: STUCK" >> $LOG
        if killgame TERM; then
          echo "STUCK ($V, quiet $((STALL * 2)) s): killed; nothing logged while frames kept turning. A chain must stop here"
          $HERE/archive.sh $DIR $MARK stuck-$WORLD $LOG; exit 9
        else
          flag "STUCK and would not die ($V)"
          echo "STUCK and the game would not die: GPU STALL flagged; no more Mac runs this boot, tell Planner"
          $HERE/archive.sh $DIR $MARK stalled-$WORLD $LOG; exit 5
        fi ;;
      LOCK*)
        if killgame KILL; then
          echo "HUNG ($V): killed; a code hang, runs go on (exit 6: a chain must still stop)"
          $HERE/archive.sh $DIR $MARK hung-$WORLD $LOG; exit 6
        else
          flag "LOCK hang, would not die ($V)"
          echo "HUNG and the game would not die: GPU STALL flagged"
          $HERE/archive.sh $DIR $MARK stalled-$WORLD $LOG; exit 5
        fi ;;
      *)
        killgame KILL; flag "$V"
        echo "GPU STALL ($V): killed; no more Mac runs this boot, tell Planner"
        $HERE/archive.sh $DIR $MARK stalled-$WORLD $LOG; exit 5 ;;
    esac
  fi
  if (( SECONDS - start >= TMO )); then
    quiet=$(( SECONDS - lastchange ))
    echo "TIMEOUT after ${TMO}s (log quiet for ${quiet}s)"
    if killgame TERM && (( quiet < ${QUIETMAX:-30} )); then
      echo "the game ended on SIGTERM and was still logging: a plain timeout (exit 8); a chain must stop"
      $HERE/archive.sh $DIR $MARK timeout-$WORLD $LOG; exit 8
    fi
    flag "TIMEOUT after ${TMO}s, log quiet ${quiet}s, $(kill -0 $PID 2>/dev/null && echo 'still running after SIGKILL' || echo 'killed')"
    echo "TIMEOUT with a quiet log or a game that would not die: GPU STALL flagged; no more Mac runs this boot, tell Planner"
    $HERE/archive.sh $DIR $MARK stalled-$WORLD $LOG; exit 5
  fi
done
wait $PID 2>/dev/null; rc=$?; echo "exit $rc after $((SECONDS - start))s"
$HERE/archive.sh $DIR $MARK $WORLD $LOG
sed 's/\x1b\[[0-9;]*m//g' $LOG | grep -E "testkit: (PASS|FAIL)|testkit report|ERROR|panicked" | tail -20
exit 0
