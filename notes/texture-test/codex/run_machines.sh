#!/bin/sh
# Every grid in batches_machines.tsv, two at a time.
cd "$(dirname "$0")"
run() { IFS='	' read -r name tl tr bl br rest; ./gen_grid.sh "$name" "$tl" "$tr" "$bl" "$br"; }
i=0
while IFS= read -r line; do
  printf '%s\n' "$line" | run &
  i=$((i+1))
  [ $((i % 2)) -eq 0 ] && wait
done < batches_machines.tsv
wait
