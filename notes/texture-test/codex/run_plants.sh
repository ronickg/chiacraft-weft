#!/bin/sh
# Every grid in batches_plants.tsv, two at a time.
cd "$(dirname "$0")"
run() { IFS='	' read -r name tl tr bl br rest; ./gen_sprites.sh "$name" "$tl" "$tr" "$bl" "$br"; }
i=0
while IFS= read -r line; do
  printf '%s\n' "$line" | run &
  i=$((i+1))
  [ $((i % 2)) -eq 0 ] && wait
done < batches_plants.tsv
wait
