#!/usr/bin/env python3
"""Test only: writes Distant Horizons' block-resolution far data out of its sqlite for our importer
(crates/mcimport, `dh`), so the game needs no sqlite of its own.

  dh_export.py <DistantHorizons.sqlite> <out dh0.bin>

Reads the FullData rows of detail level 0 (64x64 columns of one block each, format 2) and writes
`CCDH0001`, then per row: x, z, min y, the data blob's length and bytes, the mapping blob's length
and bytes (little-endian u32 lengths; the blobs stay zstd as DH wrote them). The world and its
export are Ronald's own and local only: never commit them.
"""
import sqlite3, struct, sys

if len(sys.argv) != 3:
    sys.exit(__doc__)
db = sqlite3.connect(f"file:{sys.argv[1]}?mode=ro", uri=True)
fmt = {r[0] for r in db.execute("select distinct DataFormatVersion from FullData where DetailLevel=0")}
modes = {r[0] for r in db.execute("select distinct CompressionMode from FullData where DetailLevel=0")}
if fmt != {2} or modes != {4}:
    sys.exit(f"expected format 2 with zstd (mode 4), found formats {fmt}, modes {modes}")
n = 0
with open(sys.argv[2], "wb") as out:
    out.write(b"CCDH0001")
    for x, z, miny, data, mapping in db.execute(
            "select PosX, PosZ, MinY, Data, Mapping from FullData where DetailLevel=0 and Data is not null and Mapping is not null"):
        out.write(struct.pack("<iiiI", x, z, miny, len(data)) + data + struct.pack("<I", len(mapping)) + mapping)
        n += 1
print(f"wrote {n} sections to {sys.argv[2]}")
