#!/bin/bash
# proton_run.sh <out dir> <limit s> [weft args...]: the DLSS kit's weft.exe on the Linux PC under Proton-CachyOS
# (pacman proton-cachyos-native + umu-launcher; dxvk-nvapi and the driver's NGX shim with PROTON_ENABLE_NVAPI=1) on a
# private virtual KWin screen: DLSS SR/DLAA/RR through Streamline without booting Windows (11-06). Only inside a t2q hold.
# Paths for the game are Windows ones (Z:\home\...). WEFT_DLSS_JITTER etc. pass through.
out=$1; limit=$2; shift 2
export XDG_RUNTIME_DIR=/run/user/$(id -u)
sock=wayland-dlss
mkdir -p "$out"
pkill -f "socket $sock" 2>/dev/null; sleep 1; rm -f $XDG_RUNTIME_DIR/$sock $XDG_RUNTIME_DIR/$sock.lock
timeout $((limit + 40)) kwin_wayland --virtual --xwayland --width 1920 --height 1080 --socket $sock > "$out/kwin.log" 2>&1 &
kw=$!
for i in $(seq 1 30); do [ -S $XDG_RUNTIME_DIR/$sock ] && break; sleep 0.5; done
sleep 2
disp=$(grep -o -E "DISPLAY=:[0-9]+|display :[0-9]+" "$out/kwin.log" | head -1 | grep -o ":[0-9]*")
export WINEPREFIX=$HOME/win/pfx PROTONPATH=/usr/share/steam/compatibilitytools.d/proton-cachyos-native GAMEID=umu-chiacraft
export PROTON_ENABLE_NVAPI=1 PROTON_LOG=1 PROTON_LOG_DIR="$out" WAYLAND_DISPLAY=$sock
[ -n "$WINE_WAYLAND" ] && export PROTON_ENABLE_WAYLAND=1
[ -n "$disp" ] && export DISPLAY=$disp || unset DISPLAY
echo "DISPLAY=${DISPLAY:-none} wayland=${PROTON_ENABLE_WAYLAND:-0}" > "$out/env.txt"
cd $HOME/win/chiacraft-dlss-kit/chiacraft
timeout -s KILL $limit umu-run ./weft.exe "$@" > "$out/stdout.log" 2>&1
echo "exit $?" >> "$out/env.txt"
kill $kw 2>/dev/null; wait $kw 2>/dev/null
cp run/logs/latest.log "$out/game.log" 2>/dev/null
cp streamline*/sl.log "$out/" 2>/dev/null
