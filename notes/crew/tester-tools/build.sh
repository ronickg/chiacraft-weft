#!/bin/zsh
# build.sh name:commit [...]: git archive each commit to exp/<name> and build it, one at a time,
# niced at 4 jobs. A folder already there keeps target/ and run dirs; changed files get a fresh
# mtime (no rsync -t), so cargo rebuilds them even when the commit is older than the last build.
# exp/ lives in the archive (TESTER_EXP overrides), not in the repo.
T=${TESTER_EXP:-$HOME/Coding/chiacraft-archive/tester-tools}; mkdir -p $T/exp
for pair in "$@"; do
  n=${pair%%:*} c=${pair#*:}; D=$T/exp/$n; X=$T/exp/$n.src
  rm -rf $X && mkdir -p $X $D && (cd $HOME/Coding/chiacraft-weft && git archive $c | tar -x -C $X) || { echo "EXPORT FAILED $n $c"; continue; }
  rsync -rlpD --checksum --delete --exclude /target --exclude '/run*' --exclude /xfer --exclude '/did-*' $X/ $D/ && rm -rf $X
  echo $c > $D/.commit
  cd $D && echo "== $n $c $(date +%T)" && nice -n 15 cargo build --release -j 4 2>&1 | grep -E "^error|Finished"; nice -n 15 cargo build --release -j 4 -p testmod 2>&1 | grep -E "^error|Finished"
  echo "BUILT $n $c $(date +%T)"
done
echo CHAIN-DONE
