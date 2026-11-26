#!/bin/bash
# Monitor for the PC job runner (rebuilt 11-22): prints a line when something needs a look. Run it in a background task or a Monitor:
#   notes/crew/tester-tools/t2/pcwatch.sh            # loops every 60 s, one line per alert (and per change), quiet while all is well
# Alerts: PC not answering, runner not running, /home free under 30 GB, GPU over 85 C, a running job older than its limit + 10 min, queue idle with pending jobs.
PC=ronaldg@192.168.1.13
last=""
while true; do
  out=$(ssh -o BatchMode=yes -o ConnectTimeout=8 $PC '
    q=~/looktest/queue
    echo "runner=$(pgrep -f "[r]unner.py" | head -1)"
    echo "free=$(df --output=avail -BG /home | tail -1 | tr -dc 0-9)"
    echo "gpu=$(nvidia-smi --query-gpu=temperature.gpu,utilization.gpu,memory.used --format=csv,noheader | tr -d " ")"
    echo "pending=$(ls $q/pending | wc -l) running=$(ls $q/running | wc -l)"
    now=$(date +%s)
    for f in $q/running/*.json; do [ -e "$f" ] || continue; lim=$(grep -o "\"limit\": [0-9]*" $f | grep -o "[0-9]*"); age=$((now - $(stat -c %Y $f))); [ -n "$lim" ] && [ $age -gt $((lim + 600)) ] && echo "stuck=$(basename $f .json) age=${age}s limit=${lim}s"
    done' 2>&1)
  if [ -z "$out" ] || ! echo "$out" | grep -q runner=; then msg="PC does not answer (asleep or off?)"
  else
    msg=""
    echo "$out" | grep -q '^runner=$' && msg="$msg runner NOT running;"
    free=$(echo "$out" | sed -n 's/^free=//p'); [ "${free:-99}" -lt 30 ] && msg="$msg /home only ${free} GB free;"
    t=$(echo "$out" | sed -n 's/^gpu=\([0-9]*\).*/\1/p'); [ "${t:-0}" -gt 85 ] && msg="$msg GPU ${t} C;"
    echo "$out" | grep '^stuck=' | while read s; do :; done
    st=$(echo "$out" | grep '^stuck=' | tr '\n' ' '); [ -n "$st" ] && msg="$msg $st"
  fi
  [ -n "$msg" ] && [ "$msg" != "$last" ] && echo "$(date +%H:%M) PCWATCH:$msg"
  last="$msg"
  sleep 60
done
