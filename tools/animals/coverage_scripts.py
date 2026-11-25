#!/usr/bin/env python3
"""Writes the animal AI coverage checks (mods/testkit/scripts/cov_*.txt), one per behaviour group,
each covering every species of its group in parts on flat pads 200 blocks apart on seed 1 (the
coverage table: notes/crew/plans/animals-coverage.md). Run: python3 tools/animals/coverage_scripts.py"""
import os

OUT = os.path.join(os.path.dirname(__file__), "..", "..", "mods", "testkit", "scripts")
Z = 544
# Each pad starts at noon: the runs are long, and their last shots were taken at night.
TIME = 6000


def head(title, lines):
    out = [f"# {l}" for l in lines] + [
        "loaded", "idle", "chat /tk mode creative", "chat /tp 0 90 544", "wait 4", "idle", "where",
        "chat /season summer 1", "chat /time set 6000", "chat /rule core:mob_spawning 0",
        "chat /anim log", "chat /tk mode spectator",
    ]
    return out


def pad(x, pond=1, deep=1):
    """A flat 33 x 33 pad round (x, Z) at ground 69, a pond of (2*pond+1) blocks square, `deep` deep."""
    return [
        f"chat /tp {x} 90 {Z}", "wait 4",
        f"chat /tk fill {x-16} 60 {Z-16} {x+16} 68 {Z+16} base:dirt",
        f"chat /tk fill {x-16} 69 {Z-16} {x+16} 69 {Z+16} base:grass",
        f"chat /tk fill {x-16} 70 {Z-16} {x+16} 85 {Z+16} core:air",
        f"chat /time set {TIME}",
        f"chat /tk fill {x-pond} {70-deep} {Z-pond} {x+pond} 69 {Z+pond} base:water",
        f"chat /tk fill {x-pond} 70 {Z-pond} {x+pond} 70 {Z+pond} core:air",
        "wait 1",
    ]


FISH = {"brown_trout", "pacu", "lungfish", "lycoptera"}


def spawns(x, groups):
    """groups: [(kind, count, dx, dz)]: a cluster of `count` 2 blocks apart at (x+dx, Z+dz); fish
    in the water (cov_small: two lycoptera spawned on its top suffocated)."""
    out = []
    for kind, n, dx, dz in groups:
        for i in range(n):
            y = 68 if kind in FISH else 70
            out.append(f"chat /tk spawn animals:{kind} {x+dx+(i%3)*2} {y} {Z+dz+(i//3)*2}")
    return out + ["wait 2"]


def watch(x, rows, every=10, r=64):
    out = [f"chat /tp {x} 95 {Z+30}", "wait 1"]
    for _ in range(rows):
        out += [f"chat /animalprof where {r}", f"wait {every}"]
    return out


def shot(x, name, z_back=14, pitch=-38):
    return [
        "chat /tick freeze", f"chat /tp {x} 82 {Z+z_back}", f"look 0 {pitch}", "wait 2",
        f"shot {name}.png", "chat /tick unfreeze", "wait 1",
    ]


def scare(x, dz=6):
    """A creative player walks in among them, then hits the nearest (`/animalprof hit`)."""
    return [
        "chat /tk mode creative", f"chat /tp {x} 70 {Z+dz+8}", "look 0 0", "wait 1",
        "hold core:forward 2", "chat /animalprof hit 2", "wait 5", f"chat /animalprof where 64",
        "chat /tk mode spectator",
    ]


def write(name, lines):
    p = os.path.join(OUT, f"cov_{name}.txt")
    with open(p, "w") as f:
        f.write("\n".join(lines + ["quit"]) + "\n")
    print(p, len(lines))


# 1. Grazers and herds: eat, drink, flee, the herd holds together.
GRAZERS = [
    ("white_tailed_deer", 3, "red_deer", 3), ("reindeer", 3, "capybara", 3),
    ("protoceratops", 3, "psittacosaurus", 3), ("gastonia", 2, "pinacosaurus", 2),
    ("iguanodont", 2, "cedarosaurus", 2), ("citipati", 2, None, 0),
]
s = head("grazers", [
    "Animal AI coverage, grazers and herds (Planner 11-25; table: notes/crew/plans/animals-coverage.md). Each part: a pad 200",
    "blocks on, two species' groups at 30 % food and water, a pond in the middle. Expected per species: \"plays graze\" and",
    "\"plays drink\" (anim log) or doing eat/drink in the where rows, food and water climbing; a herd's members within 16",
    "blocks of one another (where rows); after the walk-in and `/animalprof hit`: \"runs from\" / \"calls the alarm\" lines,",
    "or a big plant eater standing its ground (\"charges\"). Report per species: eat, drink, flee, holds together; the shot.",
])
for i, (a, na, b, nb) in enumerate(GRAZERS):
    x = i * 200
    g = [(a, na, -8, -8)] + ([(b, nb, 4, 4)] if b else [])
    s += pad(x) + spawns(x, g) + ["chat /animalprof food 0.3", "chat /animalprof water 0.3"]
    s += watch(x, 6) + scare(x) + shot(x, f"cov_grazers_{i+1}_{a}")
write("grazers", s)

# 2. Hunters other than wolves: stalk, chase, share, give up.
HUNTERS = [
    [("jaguar", 1, 10, -12), ("white_tailed_deer", 3, -8, 6)],
    [("coyote", 2, 10, -12), ("hare", 3, -8, 6)],
    [("red_fox", 1, 10, -12), ("arctic_fox", 1, 12, -10), ("wood_mouse", 3, -8, 6), ("field_vole", 3, -2, 8)],
    [("raptor", 1, 10, -12), ("velociraptor_osmolskae", 1, 12, -10), ("kryptobaatar", 3, -8, 6), ("priscagama", 2, -2, 8)],
    [("utahraptor", 3, 8, -12), ("psittacosaurus", 3, -8, 6)],
    [("crocodyliform", 1, 2, 0), ("yacare_caiman", 1, -2, 0), ("spectacled_caiman", 1, 0, 2), ("capybara", 3, -8, 6)],
]
s = head("hunters", [
    "Animal AI coverage, hunters other than wolves. Each part: hungry hunters (30 % food, `/animalprof hunters 0.3`) and",
    "fed prey (90 %) on a pad. Expected per hunter: \"hunts\" (whom, a pack of N), stalking (\"breaks cover and charges\" for",
    "ambushers) or a chase, then a kill (\"eats N's kill\", each pack member's share) or \"gives up on\"; prey \"runs from\".",
    "The last part's pond is 5 x 5 and 2 deep for the caimans. Report per hunter: hunts, stalk/ambush, chase, kill, share,",
    "gives up (each with its log line or none); the shot.",
])
for i, g in enumerate(HUNTERS):
    x = i * 200
    s += pad(x, pond=2 if i == 5 else 1, deep=2 if i == 5 else 1) + spawns(x, g)
    s += ["chat /animalprof food 0.9", "chat /animalprof water 0.9", "chat /animalprof hunters 0.3"]
    s += watch(x, 8) + shot(x, f"cov_hunters_{i+1}_{g[0][0]}")
write("hunters", s)

# 3. Birds and fliers.
BIRDS = [
    ("songbirds", "6000", [("house_sparrow", 3, -8, -8), ("snow_bunting", 3, 4, -8), ("confuciusornis", 3, -8, 4), ("gobipteryx", 2, 4, 4)], False),
    ("raptors", "6000", [("golden_eagle", 1, 10, -12), ("savanna_hawk", 1, 12, -10), ("hare", 3, -8, 6), ("wood_mouse", 3, -2, 8)], True),
    ("owls", "18000", [("great_horned_owl", 1, 10, -12), ("snowy_owl", 1, 12, -10), ("wood_mouse", 3, -8, 6), ("field_vole", 3, -2, 8)], True),
    ("scavengers", "6000", [("turkey_vulture", 2, 8, -10), ("raven", 1, 10, -8), ("herring_gull", 2, -8, -10)], False),
]
s = head("birds", [
    "Animal AI coverage, birds and fliers. Parts: songbirds (flock, feed, flee the walk-in), birds of prey hungry over hares",
    "and mice, owls at night over mice and voles, scavengers (vultures, a raven, gulls) with a deer killed on the pad",
    "(`/animalprof kill`). Expected: birds in the air and landing (the where rows' heights), \"plays\" clips, a flock",
    "together, flight from the walk-in, \"hunts\"/kills by the hunters, \"smells carrion\" / \"scavenger N eats\". Report per",
    "species: flies, lands/perches, flocks, flees, hunts or scavenges; the shots.",
])
for i, (name, time, g, hungry) in enumerate(BIRDS):
    x = i * 200
    s += pad(x) + [f"chat /time set {time}"]
    if name == "scavengers":
        s += spawns(x, [("white_tailed_deer", 1, 0, 6)]) + [f"chat /tp {x} 71 {Z+9}", "chat /animalprof kill", "wait 1", "chat /tp " + f"{x} 90 {Z}"]
    s += spawns(x, g) + ["chat /animalprof food 0.9", "chat /animalprof water 0.9"]
    if hungry:
        s += ["chat /animalprof hunters 0.3"]
    if name == "scavengers":
        s += ["chat /animalprof food 0.3"]
    s += watch(x, 7) + (scare(x, -6) if name == "songbirds" else []) + shot(x, f"cov_birds_{i+1}_{name}", z_back=18, pitch=-25)
write("birds", s)

# 4. Small animals (and fish).
SMALL = [
    ("hares_mice", [("hare", 2, -8, -8), ("mountain_hare", 2, 4, -8), ("wood_mouse", 3, -8, 4), ("field_vole", 3, 4, 4)], 1, 1),
    ("cretaceous_small", [("kryptobaatar", 3, -8, -8), ("priscagama", 3, 4, 4)], 1, 1),
    ("cat_dog", [("cat", 2, -8, -8), ("chihuahua", 2, 4, 4)], 1, 1),
    ("fish", [("brown_trout", 2, -2, -2), ("pacu", 2, 0, -2), ("lungfish", 2, -2, 1), ("lycoptera", 3, 1, 1)], 4, 3),
]
s = head("small", [
    "Animal AI coverage, small animals and fish. Parts: hares, mountain hares, mice and voles; the Cretaceous's kryptobaatar",
    "and Priscagama; wild cats and chihuahuas; fish in a 9 x 9 pool 3 deep. Each at 30 % food and water, then a creative",
    "walk-in and `/animalprof hit`. Expected: eat (\"plays graze\"/doing eat), drink, wander, flee (\"runs from\"), fish",
    "swimming (y under 70 in the where rows) and darting off. Report per species: eat, drink, wander, flee; the shots.",
])
for i, (name, g, pond, deep) in enumerate(SMALL):
    x = i * 200
    s += pad(x, pond, deep) + spawns(x, g) + ["chat /animalprof food 0.3", "chat /animalprof water 0.3"]
    s += watch(x, 6) + scare(x, -6 if name != "fish" else 0) + shot(x, f"cov_small_{i+1}_{name}")
write("small", s)

# 5. Big dinosaurs and bosses: aggro, leash, death.
s = head("bosses", [
    "Animal AI coverage, big dinosaurs and bosses. Part 1-4: an iguanodont, a gastonia, a cedarosaurus and a wyvern, each",
    "hit by the player (`/animalprof hit 4`, creative): expected a charge or a stand (\"charges\", \"stands\") or flight, logged.",
    "Parts 5-6: Drenkaun and Borkon summoned at their altars on sky pads (as drenkaun.txt / borkon.txt): it rises and fights",
    "(\"summoned\", \"hits\"), the player leaves (spectator 90 blocks off for 70 s): \"no one to fight, back to its place\";",
    "back, `/boss kill`: \"died\", \"loot\". Report per species: aggro, leash, death, with the lines; the shots.",
])
for i, kind in enumerate(["iguanodont", "gastonia", "cedarosaurus", "wyvern"]):
    x = i * 200
    s += pad(x) + spawns(x, [(kind, 1, 0, -6)])
    s += watch(x, 2) + ["chat /tk mode creative", f"chat /tp {x} 70 {Z+2}", "look 0 0", "wait 1",
                        "chat /animalprof hit 4", "wait 2", "chat /animalprof where 64", "wait 4",
                        "chat /animalprof where 64", "chat /tk mode spectator"]
    s += watch(x, 3, every=5) + shot(x, f"cov_bosses_{i+1}_{kind}", z_back=18, pitch=-25)
for j, (boss, offering, sword, armour) in enumerate([("drenkaun", "forest_offering", "flint", "leather"), ("borkon", "mountain_offering", "bronze", "bronze")]):
    x = 1000 + j * 200
    s += [f"chat /tp {x}.5 206 0.5", "wait 3", "loaded",
          f"chat /tk fill {x-22} 196 -22 {x+22} 198 22 stone",
          f"chat /tk fill {x-22} 199 -22 {x+22} 199 22 " + ("base:grass" if boss == "drenkaun" else "stone"),
          f"chat /tk fill {x-22} 200 -22 {x+22} 214 22 core:air"]
    if boss == "drenkaun":
        for dx, dz in [(-4, -4), (4, -4), (-4, 4), (4, 4)]:
            s += [f"chat /tk fill {x+dx} 200 {dz} {x+dx} 204 {dz} base:oak_log",
                  f"chat /tk fill {x+dx-1} 205 {dz-1} {x+dx+1} 206 {dz+1} base:oak_leaves"]
    else:
        s += [f"chat /tk fill {x+10} 200 -18 {x+18} 204 -10 stone"]
    s += [f"chat /tk fill {x} 200 0 {x} 200 0 animals:{boss}_altar", f"chat /tp {x}.5 200 2.5", "wait 1",
          "chat /tk mode survival", "chat /tk attr_mod me core:max_health testkit:hp core:add 300 0",
          f"chat /tk put me 0 animals:{offering} 8", f"chat /gear sword {sword} slot=1", "chat /tk heal",
          "key 1", "look 0 -29", "wait 0.5", "press core:use", "wait 3", "look 0 8", "chat /boss state",
          "press core:attack", "wait 0.7", "press core:attack", "wait 3", "chat /boss state", "chat /tk heal",
          "chat /tick freeze", f"chat /tp {x}.5 204 12", "look 0 -15", "wait 2", f"shot cov_bosses_{5+j}_{boss}.png",
          "chat /tick unfreeze", "chat /tk mode spectator", f"chat /tp {x}.5 210 92", "wait 70", "chat /boss state",
          f"chat /tp {x}.5 204 8", "chat /tk mode survival", "chat /tk heal", "wait 2", "chat /boss state",
          "chat /boss kill", "wait 3", "chat /boss state", "chat /tk mode spectator"]
write("bosses", s)

# 6. Young: follow the mother, grow up.
YOUNG = [
    [("white_tailed_deer", 6, -8, -8), ("reindeer", 6, 4, 4)],
    [("capybara", 6, -8, -8), ("grey_wolf", 4, 4, 4)],
    [("jaguar", 4, -8, -8), ("hare", 6, 4, 4)],
    [("protoceratops", 6, -8, -8), ("iguanodont", 4, 4, 4)],
]
s = head("young", [
    "Animal AI coverage, the young. `/tk spawn` gives some of a kind young (\"(young)\" in the where rows). Each part: six of",
    "a kind fed and watered, watched 40 s (a young one keeps near its mother: its distance to the nearest grown one of its",
    "kind), then `/animal hurry` (growing up within a second) and watched again (\"(young)\" gone, max health up). Report per",
    "species: young spawned (how many), keeps by a grown one (distance), grows up (before/after health); the shots.",
])
for i, g in enumerate(YOUNG):
    x = i * 200
    s += pad(x) + spawns(x, g) + ["chat /animalprof food 0.9", "chat /animalprof water 0.9"]
    s += watch(x, 4) + shot(x, f"cov_young_{i+1}_{g[0][0]}") + [f"chat /tp {x} 72 {Z}", "chat /animal hurry", "wait 3"] + watch(x, 2)
write("young", s)

# 7. Taming and eggs.
s = head("taming", [
    "Animal AI coverage, taming and eggs. Part 1: passive kinds (grey wolf, cat, chihuahua, capybara) hand-fed kibble",
    "(aimed, `core:use`): expected \"tamed at N%\" or the taming screen's progress. Part 2: knock-out kinds (raptor,",
    "osmolskae, utahraptor, jaguar, wyvern) knocked out (`/animalprof ko 1`), given kibble (aimed, `core:use`) and made to eat (`/animal bite` x8):",
    "\"knocked out\", \"tamed at\". Part 3: laying kinds (protoceratops, psittacosaurus, citipati, raptor, house sparrow):",
    "`/animal nest` then `/animal hurry`: \"nested at\", eggs, hatching (\"born\"/young in the where rows). Report per",
    "species: tamed (yes/no, the line), laid, hatched; the shots.",
])
x = 0
s += pad(x) + spawns(x, [("grey_wolf", 1, -8, -6), ("cat", 1, -4, -6), ("chihuahua", 1, 0, -6), ("capybara", 1, 4, -6)])
s += ["chat /tk mode creative", "chat /tk put me 0 animals:kibble 64", "key 1"]
for dx in [-8, -4, 0, 4]:
    s += [f"chat /tp {x+dx} 70 {Z-3}", "look 0 0", "wait 1"] + ["aim 6", "press core:use", "wait 0.6"] * 4
s += ["chat /animalprof where 64", "chat /tk mode spectator"] + shot(x, "cov_taming_1_passive")
x = 200
s += pad(x) + spawns(x, [("raptor", 1, -8, -6), ("velociraptor_osmolskae", 1, -4, -6), ("utahraptor", 1, 0, -6), ("jaguar", 1, 6, -6), ("wyvern", 1, 0, 8)])
s += ["chat /animalprof ko 1", "wait 2", "chat /tk mode creative", "chat /tk put me 0 animals:kibble 64", "key 1"]
for dx, dz in [(-8, -6), (-4, -6), (0, -6), (6, -6), (0, 8)]:
    s += [f"chat /tp {x+dx} 70 {Z+dz+2}", "look 0 0", "wait 1", "aim 6", "press core:use", "wait 0.5"] + ["chat /animal bite", "wait 0.5"] * 8
s += ["chat /animalprof where 64", "chat /tk mode spectator"] + shot(x, "cov_taming_2_knockout")
x = 400
s += pad(x) + spawns(x, [("protoceratops", 2, -8, -8), ("psittacosaurus", 2, 4, -8), ("citipati", 2, -8, 4), ("raptor", 2, 4, 4), ("house_sparrow", 2, 0, 0)])
for dx, dz in [(-8, -8), (4, -8), (-8, 4), (4, 4), (0, 0)]:
    s += [f"chat /tp {x+dx} 72 {Z+dz}", "chat /animal nest", "wait 1"]
s += watch(x, 2) + shot(x, "cov_taming_3_nests")
for dx, dz in [(-8, -8), (4, -8), (-8, 4), (4, 4), (0, 0)]:
    s += [f"chat /tp {x+dx} 72 {Z+dz}", "chat /animal hurry", "wait 1"]
s += ["wait 3"] + watch(x, 3) + shot(x, "cov_taming_4_hatched")
write("taming", s)
