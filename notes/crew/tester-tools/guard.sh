# Sourced by gamerun.sh / gamemulti.sh: refuse when a run can't or mustn't start. Sets Q, FLAG, BOOT.
# Q (logs, the lock, the PAUSED and GPU_STALLED flags) lives in the archive, not in the repo; TESTER_RUNS overrides it.
Q=${TESTER_RUNS:-$HOME/Coding/chiacraft-archive/tester-tools/runs}; mkdir -p $Q; FLAG=$Q/GPU_STALLED
if [[ -e $Q/PAUSED ]]; then echo "REFUSED: Mac runs paused: $(cat $Q/PAUSED)"; exit 4; fi
BOOT=$(sysctl -n kern.boottime | sed 's/.*sec = \([0-9]*\).*/\1/')
if [[ -e $FLAG && "$(head -1 $FLAG)" == "$BOOT" ]]; then echo "REFUSED: a GPU stall on this boot:"; tail -n +2 $FLAG; exit 3; fi
SURF=$(ioclasscount IOSurfaceRootUserClient 2>/dev/null | awk '{print $NF}')
if (( ${SURF:-0} >= 150 )); then echo "REFUSED: $SURF IOSurface connections in use (Planner's limit 150; the ceiling is ~1024): check argent's tool-server"; exit 7; fi
mkdir $Q/lock 2>/dev/null || { echo "REFUSED: lock held ($(cat $Q/lock/who 2>/dev/null))"; exit 2; }
trap 'rm -rf $Q/lock' EXIT
pgrep -x weft >/dev/null && { echo "REFUSED: a weft game is already running"; pgrep -lx weft; exit 2; }
