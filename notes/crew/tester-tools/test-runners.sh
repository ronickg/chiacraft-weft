#!/bin/zsh
# test-runners.sh: checks gamerun.sh / gamemulti.sh exit statuses and the GPU_STALLED flag with a FAKE game (a shell script
# standing in for target/release/weft, a scratch copy of the tools, stub stuck.sh and archive.sh). Launches no real game, uses
# no GPU, takes about 2 minutes. Prints one line per case: ok / WRONG.
# usage: test-runners.sh   (scratch under $TMPDIR or /tmp; leftover fake games are killed by pid, never by pattern)
set -u
SRC=${0:A:h}; D=$(mktemp -d /tmp/runner-test.XXXXXX); mkdir -p $D/tools $D/game/target/release $D/game/run
cp $SRC/gamerun.sh $SRC/gamemulti.sh $SRC/guard.sh $D/tools/
cat > $D/tools/stuck.sh <<'EOF'
#!/bin/zsh
echo ${FAKE_VERDICT:-ALIVE (9/10 samples in frames)}
EOF
cat > $D/tools/archive.sh <<'EOF'
#!/bin/zsh
echo "archive stub: $3"
EOF
cat > $D/game/target/release/weft <<'EOF'
#!/bin/zsh
# fake game: FAKE_MODE = logging (a line a second, for ever) | quiet (one line, then silence) | quiet_noterm (same, ignores SIGTERM) | ends (3 s)
echo $$ >> $FAKE_PIDS
case ${FAKE_MODE:-logging} in
  logging) while true; do echo "frame $SECONDS"; sleep 1; done ;;
  quiet) echo "loaded"; while true; do sleep 1; done ;;
  quiet_noterm) trap '' TERM; echo "loaded"; while true; do sleep 1; done ;;
  ends) echo "testkit: PASS"; sleep 3 ;;
esac
EOF
chmod +x $D/tools/*.sh $D/game/target/release/weft
export TESTER_RUNS=$D/runs FAKE_PIDS=$D/pids; : > $FAKE_PIDS; touch $D/script.txt
bad=0
# case <name> <want exit> <want flag yes|no> <tool> <env words...> -- <args...>
case_() {
  local name=$1 want=$2 wflag=$3 tool=$4; shift 4; local -a envs; while [[ $1 != -- ]]; do envs+=($1); shift; done; shift
  rm -rf $TESTER_RUNS; mkdir -p $TESTER_RUNS
  env "${envs[@]}" $D/tools/$tool "$@" > $D/$name.out 2>&1 &
  local rp=$! i; for i in {1..150}; do kill -0 $rp 2>/dev/null || break; sleep 1; done
  if kill -0 $rp 2>/dev/null; then kill -9 $rp; echo "$name: WRONG (runner still running after 150 s)"; bad=1; return; fi
  wait $rp; local got=$? gflag=no; [[ -e $TESTER_RUNS/GPU_STALLED ]] && gflag=yes
  for p in $(cat $FAKE_PIDS); do kill -9 $p 2>/dev/null; done; : > $FAKE_PIDS
  if [[ $got == $want && $gflag == $wflag ]]; then echo "$name: ok (exit $got, flag $gflag)"; else echo "$name: WRONG (exit $got want $want, flag $gflag want $wflag)"; bad=1; fi
}
G="$D/game $D/script.txt"
case_ normal_end        0 no  gamerun.sh FAKE_MODE=ends -- ${=G} w1 60
case_ timeout_logging   8 no  gamerun.sh FAKE_MODE=logging STALL=60 -- ${=G} w2 6
case_ stuck_alive       9 no  gamerun.sh FAKE_MODE=quiet FAKE_VERDICT=ALIVE STALL=3 -- ${=G} w3 90
case_ gpu_verdict       5 yes gamerun.sh FAKE_MODE=quiet FAKE_VERDICT=GPU STALL=3 -- ${=G} w4 90
case_ lock_verdict      6 no  gamerun.sh FAKE_MODE=quiet FAKE_VERDICT=LOCK STALL=3 -- ${=G} w5 90
case_ timeout_quiet     5 yes gamerun.sh FAKE_MODE=quiet STALL=100 QUIETMAX=3 -- ${=G} w6 8
case_ stuck_noterm      9 no  gamerun.sh FAKE_MODE=quiet_noterm FAKE_VERDICT=ALIVE STALL=3 -- ${=G} w7 90
# a flag for this boot makes the next run refuse (exit 3): write it, then run
rm -rf $TESTER_RUNS; mkdir -p $TESTER_RUNS; { sysctl -n kern.boottime | sed 's/.*sec = \([0-9]*\).*/\1/'; echo test; } > $TESTER_RUNS/GPU_STALLED
FAKE_MODE=ends $D/tools/gamerun.sh $D/game $D/script.txt w9 30 > $D/refused.out 2>&1; got=$?
if [[ $got == 3 ]]; then echo "refused_with_flag: ok (exit 3)"; else echo "refused_with_flag: WRONG (exit $got want 3)"; bad=1; fi
E="$D/game target/release/weft"
case_ multi_timeout     8 no  gamemulti.sh FAKE_MODE=logging STALL=60 -- ${=E} 6 m1 -- a -- b
case_ multi_stuck       9 no  gamemulti.sh FAKE_MODE=quiet FAKE_VERDICT=ALIVE STALL=3 -- ${=E} 90 m2 -- a -- b
case_ multi_gpu         5 yes gamemulti.sh FAKE_MODE=quiet FAKE_VERDICT=GPU STALL=3 -- ${=E} 90 m3 -- a -- b
for p in $(cat $FAKE_PIDS); do kill -9 $p 2>/dev/null; done
echo "outputs in $D"; exit $bad
