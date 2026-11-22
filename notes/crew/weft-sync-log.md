# Weft sync log

Newest first. Each line: date, upstream hash, dev merge, conflicts, what we dropped in favour of Weft.

- **2026-11-22, hand-port (not a full sync)**, upstream `cab2d5f72`, branch `claude/upstream-fixes-1122-e85f21`. Small fixes and speed-ups from `notes/crew/upstream-2026-11-22.md`, one commit each, rows in `notes/ENGINE_CHANGES.md` section 10. Dropped in favour of Weft (rows left the ledger): our Vulkan upload fixes 9b919255 (drop uploads queued for a destroyed buffer or texture) and 0b06e383 (order upload copies that write the same bytes). Upstream's 1b9b2774 and 1049ba95 have been the code in our tree since the 11-05 sync; only the rows were left.
- **2026-11-05, Weft v26.11.2**, upstream `2db18c23`, sync branch `sync/weft-26.11.2`, merge `e9c8405c`.
