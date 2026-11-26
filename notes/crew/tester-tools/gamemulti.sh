#!/bin/zsh
# gamemulti.sh <dir> <binary rel. to dir> <timeout> <label> -- args1 -- args2 [-- args3]
# Several copies at once (2 s apart), with gamerun.sh's guards, sampling and exit status, per copy:
#   0 all ended on their own; 2,3,4,7 refused by guard.sh; 5 GPU stall (all killed, runs/GPU_STALLED written; also a timeout with a
#   quiet log or a copy that would not die); 6 a lock hang (all killed); 8 plain TIMEOUT (all ended on SIGTERM, logs not quiet);
#   9 STUCK (a copy's log quiet two stall windows while frames kept turning; all killed). A chain must stop on any non-zero status.
set -u
DIR=$1 BIN=$2 TMO=$3 LABEL=$4; shift 4; [[ ${1:-} == -- ]] && shift; STALL=${STALL:-90}
source ${0:A:h}/guard.sh
HERE=${0:A:h}
echo "$LABEL in $DIR since $(date +%T)" > $Q/lock/who
cd $DIR || exit 4; [[ $BIN == /* ]] || BIN=./$BIN; [[ -x $BIN ]] || { echo "no binary $DIR/$BIN"; exit 4; }
[[ -e run/mods/testmod.zip ]] && { mkdir -p run/stale-mods; mv run/mods/testmod.zip run/stale-mods/; }
MARK=$Q/.mark-$$; touch $MARK; T=$(date +%H%M%S); typeset -a PIDS LOGS; n=0; cur=()
launch() { n=$((n+1)); L=$Q/$LABEL-$T-$n.log; $BIN "${cur[@]}" > $L 2>&1 & PIDS+=($!); LOGS+=($L); echo "copy $n pid $! log $L"; sleep 2; }
for a in "$@"; do if [[ $a == -- ]]; then launch; cur=(); else cur+=("$a"); fi; done; [[ ${#cur[@]} -gt 0 ]] && launch
start=$SECONDS; typeset -A last chg strikes; for i in {1..$n}; do last[$i]=0; chg[$i]=$SECONDS; strikes[$i]=0; done
alive() { for p in $PIDS; do kill -0 $p 2>/dev/null && return 0; done; return 1; }
anyalive() { alive; }
# Kills every copy and waits for them to be gone, never for ever: SIGTERM first (unless $1 is KILL), then SIGKILL; returns 1 if one is still there.
killall_copies() {
  local i
  if [[ ${1:-TERM} != KILL ]]; then kill $PIDS 2>/dev/null; for i in {1..10}; do anyalive || return 0; sleep 1; done; fi
  kill -9 $PIDS 2>/dev/null; for i in {1..30}; do anyalive || return 0; sleep 1; done
  return 1
}
flag() { { echo $BOOT; echo "$(date +%T) $LABEL: $1; last line:"; sed 's/\x1b\[[0-9;]*m//g' ${LOGS[1]} | tail -1; } > $FLAG; }
while alive; do
  sleep 2
  for i in {1..$n}; do
    P=$PIDS[$i] L=$LOGS[$i]; kill -0 $P 2>/dev/null || { chg[$i]=$SECONDS; continue; }
    s=$(stat -f %z $L); if (( s != last[$i] )); then last[$i]=$s; chg[$i]=$SECONDS; strikes[$i]=0; fi
    if (( SECONDS - chg[$i] >= STALL )); then
      ST=$Q/$LABEL-$T-$i.stack.txt; V=$($HERE/stuck.sh $P $ST); echo "$(date +%T) copy $i quiet ${STALL}s: $V"
      case $V in
        ALIVE*)
          strikes[$i]=$(( strikes[$i] + 1 ))
          if (( strikes[$i] < 2 )); then echo "# tester $(date +%T): quiet ${STALL}s but $V; one more window" >> $L; last[$i]=$(stat -f %z $L); chg[$i]=$SECONDS; continue; fi
          echo "# tester $(date +%T): quiet two windows, frames turning: STUCK" >> $L
          if killall_copies TERM; then
            echo "STUCK copy $i ($V): killed all; nothing logged while frames kept turning. A chain must stop here"
            $HERE/archive.sh $DIR $MARK stuck-$LABEL $LOGS; exit 9
          else
            flag "copy $i STUCK and would not die ($V)"; echo "STUCK and a copy would not die: GPU STALL flagged"
            $HERE/archive.sh $DIR $MARK stalled-$LABEL $LOGS; exit 5
          fi ;;
        LOCK*)
          if killall_copies KILL; then
            echo "HUNG copy $i ($V): killed all; a code hang (exit 6: a chain must still stop)"
            $HERE/archive.sh $DIR $MARK hung-$LABEL $LOGS; exit 6
          else
            flag "copy $i LOCK hang, would not die ($V)"; echo "HUNG and a copy would not die: GPU STALL flagged"
            $HERE/archive.sh $DIR $MARK stalled-$LABEL $LOGS; exit 5
          fi ;;
        *)
          killall_copies KILL; flag "copy $i: $V"
          echo "GPU STALL copy $i ($V): killed all; no more Mac runs this boot, tell Planner"
          $HERE/archive.sh $DIR $MARK stalled-$LABEL $LOGS; exit 5 ;;
      esac
    fi
  done
  if (( SECONDS - start >= TMO )); then
    quiet=0; for i in {1..$n}; do kill -0 $PIDS[$i] 2>/dev/null && (( SECONDS - chg[$i] > quiet )) && quiet=$(( SECONDS - chg[$i] )); done
    echo "TIMEOUT ${TMO}s (the quietest log for ${quiet}s)"
    if killall_copies TERM && (( quiet < ${QUIETMAX:-30} )); then
      echo "all copies ended on SIGTERM and were still logging: a plain timeout (exit 8); a chain must stop"
      $HERE/archive.sh $DIR $MARK timeout-$LABEL $LOGS; exit 8
    fi
    flag "TIMEOUT ${TMO}s, a log quiet ${quiet}s, $(anyalive && echo 'a copy still running after SIGKILL' || echo killed)"
    echo "TIMEOUT with a quiet log or a copy that would not die: GPU STALL flagged; no more Mac runs this boot, tell Planner"
    $HERE/archive.sh $DIR $MARK stalled-$LABEL $LOGS; exit 5
  fi
done
for i in {1..$n}; do wait $PIDS[$i]; echo "copy $i exit $?"; done; echo "took $((SECONDS - start))s"
$HERE/archive.sh $DIR $MARK $LABEL $LOGS
exit 0
