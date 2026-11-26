#!/usr/bin/env bash
# Run on CachyOS before rebooting: copies the DLSS kit to the Windows "Storage" drive
# (mounting it asks your password once). Then on Windows: <Storage>:\chiacraft-dlss-kit.
set -euo pipefail
kit="$(cd "$(dirname "$0")" && pwd)"
dev=/dev/disk/by-label/Storage
mnt=$(findmnt -n -o TARGET "$dev" || true)
if [ -z "$mnt" ]; then
	udisksctl mount -b "$dev"
	mnt=$(findmnt -n -o TARGET "$dev")
fi
touch "$mnt/.chiacraft-write-test" 2> /dev/null && rm "$mnt/.chiacraft-write-test" || {
	echo "The Storage drive mounted read-only (Windows' fast startup leaves it so). Copy the kit with a USB stick instead: $kit" >&2
	exit 1
}
rsync -a --info=progress2 --exclude results "$kit/" "$mnt/chiacraft-dlss-kit/"
sync
echo "Copied. On Windows: open $(basename "$mnt") > chiacraft-dlss-kit and follow CHECKLIST.txt."
