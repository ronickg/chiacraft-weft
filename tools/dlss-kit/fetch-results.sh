#!/usr/bin/env bash
# Run on CachyOS after the Windows test: copies the kit's results back to ~/win/dlss-results.
set -euo pipefail
dev=/dev/disk/by-label/Storage
mnt=$(findmnt -n -o TARGET "$dev" || true)
if [ -z "$mnt" ]; then
	udisksctl mount -b "$dev"
	mnt=$(findmnt -n -o TARGET "$dev")
fi
mkdir -p ~/win/dlss-results
rsync -a "$mnt/chiacraft-dlss-kit/results/" ~/win/dlss-results/
find "$mnt/chiacraft-dlss-kit/chiacraft" -maxdepth 2 -name 'sl*.log' -exec cp {} ~/win/dlss-results/ \; 2> /dev/null || true
ls ~/win/dlss-results
echo "Results in ~/win/dlss-results: tell Planner \"DLSS results in\"."
