#!/usr/bin/env bash
# Runs a chiacraft APK on the Android emulator (or a phone on adb) and takes pictures, for checks:
#
#   tools/release/android-run.sh <apk> <out dir> [seconds] [launch options...]
#
# Boots the AVD $AVD (default chiacraft: Pixel 6 screen, Android 15, x86_64, host GPU, made with
# avdmanager) without a window if no device is attached, installs the APK, writes the launch
# options to the app's args.txt (none: the title screen), starts it, takes a screenshot every
# $SHOT_EVERY seconds (default 10) for [seconds] (default 60), then saves logcat, the game's log,
# its memory (dumpsys meminfo) and the frame lines into <out dir>. TAPS="x,y@sec ..." taps the
# screen (device pixels) at those times; SWIPES="x1,y1,x2,y2,ms@sec ..." drags a finger (held
# for ms: a stick push or a look); KEYS="KEYCODE_BACK@sec ..." sends keys.
# STOP=1 shuts the emulator down at the end.
#
# On the test PC the emulator uses the GPU: take a queue hold first
# (`t2q.py hold 15 "android emulator"`), and give it a virtual screen: kwin_wayland --virtual
# --xwayland with a child that writes its $DISPLAY (~/chiacraft-android/screen.sh), then
# DISPLAY=<that> android-run.sh ...
set -euo pipefail

apk="${1:?usage: android-run.sh <apk> <out dir> [seconds] [launch options...]}"
out="${2:?out dir}"
secs="${3:-60}"
shift 3 2> /dev/null || shift $#
sdk="${ANDROID_HOME:-$HOME/Android/Sdk}"
adb="$sdk/platform-tools/adb"
pkg=net.chiacraft.game
mkdir -p "$out"

if ! "$adb" get-state > /dev/null 2>&1; then
	echo "booting AVD ${AVD:-chiacraft}"
	# The host GPU needs a display: with $DISPLAY (a virtual one on the test PC, see below) the
	# emulator's window goes there; without, the headless emulator has no EGL for the host GPU.
	window=(-no-window)
	[ -z "${DISPLAY:-}" ] || window=()
	nohup "$sdk/emulator/emulator" -avd "${AVD:-chiacraft}" "${window[@]}" -gpu "${GPU:-host}" -no-audio -no-boot-anim \
		-no-snapshot -feature Vulkan > "$out/emulator.log" 2>&1 &
	"$adb" wait-for-device
	for _ in $(seq 1 120); do
		[ "$("$adb" shell getprop sys.boot_completed 2> /dev/null | tr -d '\r')" = 1 ] && break
		sleep 2
	done
	"$adb" shell settings put system screen_off_timeout 1800000 || true
fi
"$adb" shell input keyevent KEYCODE_WAKEUP || true
"$adb" shell wm dismiss-keyguard 2> /dev/null || true

"$adb" install -r "$apk" > "$out/install.log" 2>&1 || { cat "$out/install.log"; exit 1; }
"$adb" shell am force-stop "$pkg"
# Launch options: the external app folder is writable by adb without root.
ext="/sdcard/Android/data/$pkg/files"
"$adb" shell mkdir -p "$ext"
printf '%s\n' "$*" > "$out/args.txt"
"$adb" push "$out/args.txt" "$ext/args.txt" > /dev/null
"$adb" logcat -c
started=$(date +%s)
"$adb" shell am start -W -n "$pkg/android.app.NativeActivity" > "$out/start.log" 2>&1

# "what@second" lists (taps, swipes, keys); plain loops, so macOS's bash 3 runs it too.
at() { for e in $1; do [ "${e##*@}" = "$2" ] && echo "${e%@*}"; done; true; }
every="${SHOT_EVERY:-10}"
for s in $(seq 1 "$secs"); do
	sleep 1
	for xy in $(at "${TAPS:-}" "$s"); do "$adb" shell input tap "${xy%,*}" "${xy#*,}"; done
	for k in $(at "${KEYS:-}" "$s"); do "$adb" shell input keyevent "$k"; done
	for w in $(at "${SWIPES:-}" "$s"); do "$adb" shell input swipe $(echo "$w" | tr , ' ') & done
	if [ $((s % every)) = 0 ]; then
		"$adb" exec-out screencap -p > "$out/shot_$(printf %03d "$s").png"
	fi
	"$adb" shell pidof "$pkg" > /dev/null || { echo "game ended at $s s"; break; }
done
echo "ran $(($(date +%s) - started)) s"

"$adb" exec-out screencap -p > "$out/shot_last.png" || true
"$adb" shell dumpsys meminfo "$pkg" > "$out/meminfo.txt" 2>&1 || true
"$adb" logcat -d > "$out/logcat.txt" 2>&1 || true
"$adb" shell run-as "$pkg" cat files/run/logs/latest.log > "$out/latest.log" 2>/dev/null || true
grep -E "GPU:|swapchain|ERROR|panic|fps|frame|surface" "$out/latest.log" | cut -c1-200 | tail -40 || true
grep -E "TOTAL PSS|TOTAL RSS|Graphics|Native Heap" "$out/meminfo.txt" || true
[ -z "${STOP:-}" ] || "$adb" emu kill || true
ls "$out"
