#!/bin/zsh
# stuck.sh <pid> <out>: samples the game 3 s; prints what its main thread does:
#   GPU (a GPU wait: vkWaitForFences, MoltenVK fence/semaphore waits, Metal waits), LOCK (a lock or
#   condvar most of the time), ALIVE (running frames), UNKNOWN, GONE.
PID=$1 OUT=$2
sample $PID 3 -file $OUT >/dev/null 2>&1 || { echo GONE; exit; }
awk '/com\.apple\.main-thread/{f=1;print;next} f && /^    [0-9]+ Thread_/{exit} f' $OUT > $OUT.main
total=$(head -1 $OUT.main | awk '{print $1}')
n() { grep -E "$1" $OUT.main | awk '{gsub(/[^0-9 ]/," ",$0); print $1}' | sort -rn | head -1; }
gpu=$(n "vkWaitForFences|vkQueueWaitIdle|vkDeviceWaitIdle|MVKFence[^ ]*[Ww]ait|MVKSemaphore[^ ]*[Ww]ait|MVKTimelineSemaphore[^ ]*[Ww]ait|MVKFenceSitter|waitUntilCompleted|waitUntilScheduled|IOGPU.*Wait|AGX.*[Ww]ait"); gpu=${gpu:-0}
lock=$(n "__psynch_mutexwait|__psynch_cvwait|_pthread_cond_wait|pthread_mutex_lock"); lock=${lock:-0}
frame=$(n "app5frame|vkQueueSubmit"); frame=${frame:-0}
if (( total > 0 && gpu * 2 > total )); then echo "GPU ($gpu/$total samples in a GPU wait)"
elif (( total > 0 && lock * 2 > total )); then echo "LOCK ($lock/$total samples on a lock)"
elif (( frame > 0 )); then echo "ALIVE ($frame/$total samples in frames)"
else echo "UNKNOWN (main thread: $total samples)"; fi
