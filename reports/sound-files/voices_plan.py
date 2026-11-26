"""Writes voices.tsv for sfx.py: base mobs' and the animals' voice families, from Freesound CC0
previews kept in ~/Coding/chiacraft-archive/references/sound/freesound/ (`<id>_<author>.mp3`).
Usage: python3 -I voices_plan.py > voices.tsv"""
import glob, os
SRC = os.path.expanduser("~/Coding/chiacraft-archive/references/sound/freesound")
LEN = {"ambient": (2.5, -3), "hurt": (0.8, -2), "death": (1.6, -2)}
BASE = "mods/base/assets/sounds/entity"
ANIMALS = "mods/animals/assets/sounds/voice"
# folder: {event: [(freesound id, mode)]}; files are <event>1.., in this order
PLAN = {
    f"{BASE}/zombie": {"ambient": [(58626, "hit:0"), (58626, "hit:1"), (463721, "copy")], "hurt": [(370036, "copy"), (58626, "hit:2")], "death": [(244320, "copy")]},
    f"{BASE}/skeleton": {"ambient": [(202102, "hit:0"), (202102, "hit:1")], "hurt": [(473526, "hit:0"), (473526, "hit:1")], "death": [(202102, "copy")]},
    f"{BASE}/spider": {"ambient": [(343927, "copy"), (343928, "copy")], "hurt": [(146963, "copy"), (343927, "copy")], "death": [(160350, "hit:0")]},
    f"{BASE}/creeper": {"hurt": [(343928, "copy"), (146963, "copy")], "death": [(160350, "hit:1")]},
    f"{BASE}/pig": {"ambient": [(442906, "copy"), (442907, "copy"), (442905, "copy")], "hurt": [(233151, "hit:0"), (233151, "hit:1")], "death": [(233151, "copy")]},
    f"{BASE}/cow": {"ambient": [(59245, "copy"), (275154, "copy"), (513565, "hit:0")], "hurt": [(177253, "hit:0"), (177253, "hit:1")], "death": [(365134, "hit:0")]},
    f"{BASE}/sheep": {"ambient": [(171149, "copy"), (171151, "copy"), (362283, "hit:0")], "hurt": [(362283, "hit:1"), (362283, "hit:2")], "death": [(182509, "hit:0")]},
    f"{BASE}/chicken": {"ambient": [(456803, "hit:0"), (456803, "hit:1"), (456803, "hit:2")], "hurt": [(316920, "copy"), (316921, "hit:0")], "death": [(316921, "hit:1")]},
    f"{ANIMALS}/canine": {"ambient": [(398430, "copy"), (256533, "hit:0"), (9031, "hit:0")], "hurt": [(350593, "copy"), (208031, "hit:0"), (9032, "hit:0")], "death": [(462660, "hit:0"), (208031, "hit:1")]},
    f"{ANIMALS}/fox": {"ambient": [(634005, "hit:0"), (634005, "hit:1")], "hurt": [(554308, "hit:0"), (554308, "hit:1")], "death": [(518135, "hit:0")]},
    f"{ANIMALS}/cat": {"ambient": [(110011, "copy"), (333916, "copy"), (412017, "copy")], "hurt": [(316915, "hit:0"), (316915, "hit:1")], "death": [(45199, "hit:0")]},
    f"{ANIMALS}/big_cat": {"ambient": [(437978, "hit:0"), (437978, "hit:1")], "hurt": [(63668, "copy"), (410343, "hit:0")], "death": [(675497, "hit:0")]},
    f"{ANIMALS}/deer": {"ambient": [(351113, "hit:0"), (407631, "hit:0")], "hurt": [(351113, "hit:1"), (351113, "hit:2")], "death": [(351113, "hit:3")]},
    f"{ANIMALS}/rodent": {"ambient": [(463789, "hit:0"), (463789, "hit:1")], "hurt": [(188043, "copy"), (536753, "copy")], "death": [(445958, "copy")]},
    f"{ANIMALS}/hare": {"hurt": [(536753, "copy"), (188043, "copy")], "death": [(445958, "copy")]},
    f"{ANIMALS}/songbird": {"ambient": [(404663, "hit:0"), (404663, "hit:1"), (404663, "hit:2")], "hurt": [(628951, "hit:0")], "death": [(628951, "hit:1")]},
    f"{ANIMALS}/hawk": {"ambient": [(582183, "hit:0"), (582183, "hit:1")], "hurt": [(582183, "hit:2")], "death": [(582183, "hit:3")]},
    f"{ANIMALS}/owl": {"ambient": [(465697, "copy"), (398734, "copy")], "hurt": [(353173, "hit:0")], "death": [(353173, "hit:1")]},
    f"{ANIMALS}/gull": {"ambient": [(263786, "copy"), (166707, "hit:0"), (510917, "hit:0")], "hurt": [(166703, "hit:0")], "death": [(353416, "hit:0")]},
    f"{ANIMALS}/corvid": {"ambient": [(361470, "copy"), (75162, "hit:0"), (66763, "copy")], "hurt": [(67356, "hit:0")], "death": [(191258, "hit:0")]},
    f"{ANIMALS}/hiss": {"ambient": [(675513, "hit:0")], "hurt": [(846337, "copy"), (706977, "hit:0")], "death": [(675513, "hit:1")]},
    f"{ANIMALS}/bellow": {"ambient": [(529462, "hit:0"), (529462, "hit:1")], "hurt": [(529462, "hit:2"), (529462, "hit:3")], "death": [(529462, "hit:4")]},
    f"{ANIMALS}/small_dino": {"ambient": [(320345, "copy"), (233284, "copy")], "hurt": [(740195, "copy"), (581157, "copy")], "death": [(147527, "copy")]},
    f"{ANIMALS}/dragon": {"ambient": [(85568, "copy"), (442964, "copy")], "hurt": [(185069, "hit:0")], "death": [(366096, "hit:0")]},
}
for folder, events in PLAN.items():
    for event, picks in events.items():
        secs, peak = LEN[event]
        for i, (sid, mode) in enumerate(picks, 1):
            src = glob.glob(f"{SRC}/{sid}_*")[0]
            print(f"{folder}/{event}{i}.ogg\t{src}\t{mode}\t{secs}\t{peak}")
