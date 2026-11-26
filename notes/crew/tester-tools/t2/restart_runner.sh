#!/bin/bash
# Restart the PC job runner (run on the PC, or: ssh ronaldg@192.168.1.13 'bash -s' < restart_runner.sh).
# The runner itself refuses new builds below 30 GB free on /home and prunes old C_*/target dirs below 40 GB (see runner.py: disk_free_gb, PRUNE).
cd ~/looktest/queue || exit 1
pkill -f '[r]unner.py' 2>/dev/null; sleep 1
setsid nohup python3 runner.py > runner.out 2>&1 < /dev/null &
sleep 3
ps -eo pid,etimes,args | grep '[r]unner.py' | grep -v 'sh -c' | head -2
tail -3 runner.log
