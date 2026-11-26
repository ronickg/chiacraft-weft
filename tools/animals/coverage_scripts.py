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


def pad(x, pond=1, deep=1, meadow=False):
    """A flat 33 x 33 pad round (x, Z) at ground 69, a pond of (2*pond+1) blocks square, `deep` deep."""
    return [
        f"chat /tp {x} 90 {Z}", "wait 4",
        f"chat /tk fill {x-16} 60 {Z-16} {x+16} 68 {Z+16} base:dirt",
        f"chat /tk fill {x-16} 69 {Z-16} {x+16} 69 {Z+16} base:grass",
        f"chat /tk fill {x-16} 70 {Z-16} {x+16} 85 {Z+16} core:air",
        # And the sky over it (the pads at x 600 and 1200 were dark at noon: a hill over them).
        f"chat /tk fill {x-16} 86 {Z-16} {x+16} 110 {Z+16} core:air",
        f"chat /tk fill {x-16} 111 {Z-16} {x+16} 140 {Z+16} core:air",
        # And the camera's side, south to where the shots and the where rows are taken from (round 2
        # of cov_birds: a hill between the camera and the pad at x 200).
        f"chat /tk fill {x-16} 70 {Z+17} {x+16} 100 {Z+32} core:air",
        f"chat /time set {TIME}",
        f"chat /tk fill {x-pond} {70-deep} {Z-pond} {x+pond} 69 {Z+pond} base:water",
        f"chat /tk fill {x-pond} 70 {Z-pond} {x+pond} 70 {Z+pond} core:air",
        "wait 1",
    ] + ([
        # Tall grass on the pad's far (north) strip, out of the shots' foreground: the turf alone
        # is only the ground's roots and insects, a top-up (`Species::fills_to`), and round 2's
        # grazers at 30 % ate next to nothing on bare pads.
        f"chat /tk fill {x-16} 70 {Z-16} {x+16} 70 {Z-8} base:tall_grass",
    ] if meadow else [])


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


def lights(x):
    """Glowstone round the pad for a night shot (round 7: the caimans' shot at 01:37 was black). Set after
    the part's watching, so the light never changes what the prey see of the hunters."""
    return [f"chat /tk fill {x+dx} 70 {Z+dz} {x+dx} 70 {Z+dz} base:glowstone"
            for dx, dz in [(-10, -10), (0, -12), (10, -10), (-12, 0), (12, 0), (-10, 10), (0, 12), (10, 10)]]


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
    s += pad(x, meadow=True) + spawns(x, g) + ["chat /animalprof food 0.3", "chat /animalprof water 0.3"]
    s += watch(x, 6) + scare(x) + shot(x, f"cov_grazers_{i+1}_{a}")
write("grazers", s)

# 2. Hunters other than wolves: stalk, chase, share, give up.
HUNTERS = [
    [("jaguar", 1, 10, -12), ("white_tailed_deer", 3, -8, 6)],
    [("coyote", 2, 10, -12), ("hare", 3, -8, 6)],
    [("red_fox", 1, 10, -12), ("arctic_fox", 1, 12, -10), ("wood_mouse", 3, -8, 6), ("field_vole", 3, -2, 8)],
    [("raptor", 1, 10, -12), ("velociraptor_osmolskae", 1, 12, -10), ("kryptobaatar", 3, -8, 6), ("priscagama", 2, -2, 8)],
    [("utahraptor", 3, 8, -12), ("psittacosaurus", 3, -8, 6)],
    [("crocodyliform", 1, 2, 0), ("capybara", 3, -8, 6)],
    # The caimans take 0.17 to 35 kg: a capybara (50 kg) is too big for them (round 2: they never
    # hunted beside the capybaras), so hares and pacu in the pond; on their own pad, or the
    # crocodyliform takes them.
    [("yacare_caiman", 1, -2, 0), ("spectacled_caiman", 1, 2, 2), ("pacu", 2, -2, -2), ("hare", 3, -8, 6)],
]
s = head("hunters", [
    "Animal AI coverage, hunters other than wolves. Each part: hungry hunters (30 % food, `/animalprof hunters 0.3`) and",
    "fed prey (90 %) on a pad. Expected per hunter: \"hunts\" (whom, a pack of N), stalking (\"breaks cover and charges\" for",
    "ambushers) or a chase, then a kill (\"eats N's kill\", each pack member's share) or \"gives up on\"; prey \"runs from\".",
    "The last two parts' ponds are 5 x 5 and 2 deep for the crocodiles. Report per hunter: hunts, stalk/ambush, chase, kill, share,",
    "gives up (each with its log line or none); the shot.",
])
for i, g in enumerate(HUNTERS):
    x = i * 200
    s += pad(x, pond=2 if i >= 5 else 1, deep=2 if i >= 5 else 1) + spawns(x, g)
    s += ["chat /animalprof food 0.9", "chat /animalprof water 0.9", "chat /animalprof hunters 0.3"]
    if g[0][0] == "yacare_caiman":
        # Caimans hunt by night and rest by day unless starving (round 3: the yacare rested at noon),
        # and a young one's prey are its own size (the spectacled was young): grown, at night.
        s += [f"chat /tp {x} 72 {Z}", "chat /animal hurry", "wait 2", "chat /time set 18000"]
    s += watch(x, 8) + (lights(x) if g[0][0] == "yacare_caiman" else []) + shot(x, f"cov_hunters_{i+1}_{g[0][0]}")
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
    s += watch(x, 7) + (scare(x, -6) if name == "songbirds" else []) + (lights(x) if time == "18000" else []) + shot(x, f"cov_birds_{i+1}_{name}", z_back=18, pitch=-25)
write("birds", s)

# 4. Small animals (and fish).
SMALL = [
    ("hares_mice", [("hare", 2, -8, -8), ("mountain_hare", 2, 4, -8), ("wood_mouse", 3, -8, 4), ("field_vole", 3, 4, 4)], 1, 1),
    ("cretaceous_small", [("kryptobaatar", 3, -8, -8), ("priscagama", 3, 4, 4)], 1, 1),
    ("cat_dog", [("cat", 2, -8, -8), ("chihuahua", 2, 4, 4)], 1, 1),
    ("fish", [("brown_trout", 2, -2, -2), ("pacu", 2, 0, -2), ("lungfish", 2, -2, 1), ("lycoptera", 3, -2, 3)], 4, 3),
]
s = head("small", [
    "Animal AI coverage, small animals and fish. Parts: hares, mountain hares, mice and voles; the Cretaceous's kryptobaatar",
    "and Priscagama; wild cats and chihuahuas; fish in a 9 x 9 pool 3 deep. Each at 30 % food and water, then a creative",
    "walk-in and `/animalprof hit`. Expected: eat (\"plays graze\"/doing eat), drink, wander, flee (\"runs from\"), fish",
    "swimming (y under 70 in the where rows) and darting off. Report per species: eat, drink, wander, flee; the shots.",
])
for i, (name, g, pond, deep) in enumerate(SMALL):
    x = i * 200
    s += pad(x, pond, deep, meadow=True) + spawns(x, g) + ["chat /animalprof food 0.3", "chat /animalprof water 0.3"]
    s += watch(x, 6) + scare(x, -6 if name != "fish" else 0) + shot(x, f"cov_small_{i+1}_{name}")
write("small", s)

# 5. Big dinosaurs and bosses: aggro, leash, death.
s = head("bosses", [
    "Animal AI coverage, big dinosaurs and bosses. Part 1-4: an iguanodont, a gastonia, a cedarosaurus and a wyvern, each",
    "hit by the player (`/animalprof hit 4`, creative): a grown one of 150 kg or more \"turns on it\" and \"charges 1 (it hurt it)\"",
    "for up to 15 s, then \"lets 1 go\" (the player turns spectator); a smaller or young one runs.",
    "Parts 5-6: Drenkaun and Borkon summoned at their altars on sky pads (as drenkaun.txt / borkon.txt): it rises and fights",
    "(\"summoned\", \"hits\", its health down from the player's aimed blows), `/boss kill`: \"died\", \"loot\"; summoned again,",
    "the player leaves (spectator 90 blocks off for 70 s): \"no one to fight, back to its place\". Report per boss: aggro,",
    "damage taken, death and loot, leash, with the lines; the shots.",
])
for i, kind in enumerate(["iguanodont", "gastonia", "cedarosaurus", "wyvern"]):
    x = i * 200
    # Grown first (`/animal hurry` grows the young near): `/tk spawn` gives a share of young in
    # their season, and a young one runs (round 3: the iguanodont and the cedarosaurus were young).
    s += pad(x) + spawns(x, [(kind, 1, 0, -6)]) + [f"chat /tp {x} 72 {Z}", "chat /animal hurry", "wait 2"]
    s += watch(x, 2) + ["chat /tk mode creative", f"chat /tp {x} 70 {Z+2}", "look 0 0", "wait 1",
                        "chat /animalprof hit 4", "wait 2", "chat /animalprof where 64", "wait 4",
                        "chat /animalprof where 64", "wait 4", "chat /animalprof where 64", "chat /tk mode spectator"]
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
    # Fight, then kill (`/boss kill`): a boss left alone despawns back at its place (the leash),
    # so the leash is a second summoning after (cov_bosses: "no boss near" after the leash).
    summon = [f"chat /tp {x}.5 200 2.5", "chat /tk mode survival", "chat /tk heal", "key 1", "look 0 -29",
              "wait 0.5", "press core:use", "wait 3"]
    s += [f"chat /tk fill {x} 200 0 {x} 200 0 animals:{boss}_altar", f"chat /tp {x}.5 200 2.5", "wait 1",
          # Drenkaun's player gets 300 more health to last the part; Borkon's has its own (the cap on a
          # boss's blow, `bosses::PLAYER_BLOW`, is a share of the max the player really has: base's
          # survival clamps health to the fed max, so the +300 only raised the cap, round 5).
          "chat /tk mode survival"] + (["chat /tk attr_mod me core:max_health testkit:hp core:add 300 0"] if boss == "drenkaun" else ["chat /tk attr_rm me core:max_health testkit:hp"]) + [
          f"chat /tk put me 0 animals:{offering} 8", f"chat /gear sword {sword} slot=1", "wait 0.5", "chat /tk heal"]
    # The damage step through `/boss hurt` too (round 2: the aimed blows never reached it).
    s += summon + ["chat /boss state", "chat /boss hurt 40", "wait 0.5", "chat /boss hurt 40", "wait 0.5",
                   "chat /boss state", "key 2", "aim 10", "press core:attack", "wait 0.7", "aim 10",
                   "press core:attack", "wait 0.7", "chat /tk heal", "aim 10", "press core:attack", "wait 1",
                   "chat /boss state", "chat /tk heal",
                   "chat /tick freeze", f"chat /tp {x}.5 204 12", "look 0 -15", "wait 2",
                   f"shot cov_bosses_{5+j}_{boss}.png", "chat /tick unfreeze",
                   f"chat /tp {x}.5 200 6", "chat /boss kill", "wait 3", "chat /boss state"]
    s += summon + ["chat /boss state", "chat /tk mode spectator", f"chat /tp {x}.5 210 92", "wait 70",
                   f"chat /tp {x}.5 204 8", "chat /boss state", "wait 1"]
write("bosses", s)

# 6. Young: follow the mother, grow up.
YOUNG = [
    [("white_tailed_deer", 6, -8, -8), ("reindeer", 6, 4, 4)],
    [("capybara", 6, -8, -8), ("grey_wolf", 4, 4, 4)],
    [("jaguar", 6, -8, -8), ("hare", 6, 4, 4)],
    [("protoceratops", 6, -8, -8), ("iguanodont", 4, 4, 4)],
]
s = head("young", [
    "Animal AI coverage, the young, born to mothers on the pad. Each part: six of a kind fed and watered,",
    "`/animalprof love` (every grown wild one in love: pairs meet and \"mated with\"), then `/animal hurry` (the",
    "mothers give birth or lay; the egg layers' part hurries again to hatch), watched 50 s (where rows: \"(young, N",
    "from its mother M)\": a young one keeps within a few blocks of her), then `/animal hurry` again and watched (no",
    "\"(young)\" left, max health up). Report per species: mated, born/hatched (how many), the distances to the",
    "mothers, grows up (before/after health); the shots.",
])
for i, g in enumerate(YOUNG):
    x = i * 200
    lays = g[0][0] == "protoceratops"
    s += pad(x) + spawns(x, g) + ["chat /animalprof food 0.9", "chat /animalprof water 0.9"]
    s += [f"chat /tp {x} 72 {Z}", "chat /animalprof love 24", "wait 25", "chat /animal hurry", "wait 4"]
    if lays:
        s += ["chat /animal hurry", "wait 4"]
    s += watch(x, 5) + shot(x, f"cov_young_{i+1}_{g[0][0]}", z_back=10, pitch=-45)
    s += [f"chat /tp {x} 72 {Z}", "chat /animal hurry", "wait 3"] + watch(x, 2)
write("young", s)

# 7. Taming and eggs.
s = head("taming", [
    "Animal AI coverage, taming and eggs. Parts 1-3: passive kinds (grey wolf; capybara; cat and chihuahua), each in a",
    "glass pen beside a creative player standing still among them, kibble in hand: they ask for food (within 5 blocks), and",
    "`/animalprof feed 6` every 3 s feeds those asking (the right click's code): \"fed by .. as it asked: taming N%\",",
    "then \"tamed at\". Part 4: knock-out kinds (raptor, osmolskae, utahraptor, jaguar, wyvern) knocked out",
    "(`/animalprof ko 1`), given kibble (`/animalprof feed 3`: \"out cold, given 64 to eat\") and made to eat",
    "(`/animal bite`: \"bites, out cold: taming N%\", then \"tamed at\"); while out they play no idle clips.",
    "Parts 5-9: one laying kind a pad (protoceratops, psittacosaurus, citipati, raptor, house sparrow; a nest keeps",
    "others off for 24 blocks): `/animal nest` (\"nested at\": eggs), `/animal hurry` once (\"born to\"), and the",
    "hatchlings in the where rows as \"(young, N from its mother M)\". Report per species: tamed (the % lines),",
    "laid, hatched, the hatchlings' distances to their mothers; the shots.",
])
def pen(cx, cz):
    """A glass pen, 3 x 3 inside, round (cx, cz): a passive kind kept within asking range (5 blocks) of a
    player standing beside it (round 2: the wolf, capybara and chihuahua walked off between asks)."""
    return [f"chat /tk fill {cx-2} 70 {cz-2} {cx+2} 71 {cz+2} base:glass",
            f"chat /tk fill {cx-1} 70 {cz-1} {cx+1} 71 {cz+1} core:air"]


# One pad each for the wolf (it hunts) and the capybara (it runs from the wolf); the cat and the chihuahua
# share one, each in its pen either side of the player.
for n, (x, kinds) in enumerate([(0, ["grey_wolf"]), (200, ["capybara"]), (400, ["cat", "chihuahua"])]):
    s += pad(x)
    for k, dx in zip(kinds, [3, -3]):
        s += pen(x + dx, Z) + [f"chat /tk spawn animals:{k} {x+dx} 70 {Z}"]
    s += ["wait 2", "chat /tk mode creative", f"chat /tp {x} 70 {Z}", "look 0 -20", "chat /animalprof food 1",
          "chat /animalprof water 1", "chat /tk put me 0 animals:kibble 64", "key 1", "wait 2"]
    # The wolf needs 14 asked-for feeds (7 % each) and asks every 10-30 s: round 3 reached 57 % in 70.
    s += ["chat /animalprof feed 6", "wait 3"] * (160 if kinds == ["grey_wolf"] else 70)
    # Noon again for the shot: the wolf's 480 s ran into the night (round 5: shot 1 black at 21:46).
    s += ["chat /animalprof where 64", "chat /tk mode spectator", f"chat /time set {TIME}"] + shot(x, f"cov_taming_{n+1}_{kinds[0]}", z_back=8, pitch=-50)
x = 600
# Bites enough for each kind (round 2: the jaguar reached 73 % in 12, the wyvern 69 % in 35).
KO = [("raptor", -12, 30), ("velociraptor_osmolskae", -6, 20), ("utahraptor", 0, 30), ("jaguar", 6, 24), ("wyvern", 0, -8, 70)]
s += pad(x) + spawns(x, [(k[0], 1, k[1], 0 if len(k) == 3 else k[2]) for k in KO])
s += ["chat /animalprof ko 1", "wait 2", "chat /tk mode creative", "chat /tk put me 0 animals:kibble 64", "key 1"]
for k in KO:
    dx, dz = k[1], 0 if len(k) == 3 else k[2]
    bites = k[2] if len(k) == 3 else k[3]
    s += [f"chat /tp {x+dx} 70 {Z+dz+2}", "wait 1", "chat /animalprof feed 3", "wait 0.5"] + ["chat /animal bite", "wait 0.4"] * bites
s += ["chat /animalprof where 64", "chat /tk mode spectator"] + shot(x, "cov_taming_4_knockout")
for n, kind in enumerate(["protoceratops", "psittacosaurus", "citipati", "raptor", "house_sparrow"]):
    x = 800 + n * 200
    s += pad(x) + spawns(x, [(kind, 4, -2, -2)]) + ["chat /animalprof food 1", "chat /animalprof water 1"]
    s += [f"chat /tp {x} 72 {Z}", "chat /animal nest", "wait 2"] + watch(x, 1)
    s += [f"chat /tp {x} 72 {Z}", "chat /animal hurry", "wait 4"] + watch(x, 3, every=8)
    s += shot(x, f"cov_taming_{n+5}_{kind}", z_back=8, pitch=-50)
write("taming", s)

# 8. Dogs by their owner: wild hunters spare a tamed dog within `roles::SPARED_NEAR_OWNER` (16) blocks of
# its owner; an untamed dog stays prey (Ronald, 11-25).
s = head("dogs", [
    "Animal AI coverage, dogs and wild hunters. Part 1: a dog tamed by the player (`/animal tame`), the player standing",
    "beside it in creative the whole part (spectators are not found as owners), a hungry jaguar 12 blocks off (a wild cat that takes a",
    "2.5 kg dog: a house cat takes up to 0.4 of its 4.5 kg, and r1 never hunted the untamed dog)",
    "(`/animalprof hunters 0.3`). Part 2: an untamed dog and a hungry jaguar, the player watching from 30 blocks.",
    "Expected: part 1 no \"hunts\" of the tamed dog (the jaguar may hunt nothing, or wander); part 2 the jaguar hunts",
    "the dog (\"hunts\" naming it, a chase, a kill or \"gives up on\"). Report per part: the jaguar's hunts lines (whom),",
    "the dog's health in the where rows, any \"takes N\" hurt lines on the dog; the shots.",
])
x = 0
s += pad(x) + spawns(x, [("chihuahua", 1, 2, 0)])
s += ["chat /tk mode creative", f"chat /tp {x} 70 {Z}", "look 0 -20", "wait 1", "chat /animal tame", "wait 1"]
s += spawns(x, [("jaguar", 1, 8, -10)])
s += ["chat /animalprof food 0.9", "chat /animalprof water 0.9", "chat /animalprof hunters 0.3", "wait 1"]
# The owner stays where they are: the watch rows from here, not from the camera spot 30 blocks off.
for _ in range(8):
    s += ["chat /animalprof where 64", "wait 10"]
s += shot(x, "cov_dogs_1_tamed")
x = 200
s += ["chat /tk mode spectator"] + pad(x) + spawns(x, [("chihuahua", 1, 2, 0), ("jaguar", 1, 8, -10)])
s += ["chat /animalprof food 0.9", "chat /animalprof water 0.9", "chat /animalprof hunters 0.3"]
s += watch(x, 8) + shot(x, "cov_dogs_2_untamed")
write("dogs", s)

# 9. Carcasses (notes/crew/plans/carcass-and-skinning.md, slice 1): a body lies where an animal died, and steps down
# whole, opened, bones, a pile, gone, as it is eaten and as it rots.
def carcass_rows(n, every=10):
    out = []
    for _ in range(n):
        out += ["chat /animalprof carcass", f"wait {every}"]
    return out


def side_shot(x, name):
    """From the south-east, low, at a body lying at (x, Z+6): its side, legs and back (r4's shot from above couldn't tell
    a standing deer from a lying one)."""
    return ["chat /tick freeze", f"chat /tp {x+3} 72 {Z+10}", "look 37 -22", "wait 2", f"shot {name}.png",
            "chat /tick unfreeze", "wait 1"]


def low_shot(x, name, dz=6):
    """At the ground, from the south-east, eye level about 0.5 above the grass: whether a body (or its bones) rests on
    the block's top or sinks into it (Ronald on r6: "doesnt like fall half into the block")."""
    return ["chat /tick freeze", f"chat /tp {x+3} 68.9 {Z+dz+3}", "look 45 -5", "wait 2", f"shot {name}.png",
            "chat /tick unfreeze", "wait 1"]


s = head("carcass", [
    "Carcasses (slices 1 and 2). Part 1: three hungry grey wolves and two fed white-tailed deer: the kill leaves a carcass",
    "(\"carcass N of white_tailed_deer\") that the pack's meal opens or takes to bones (\"carcass N of ... now Opened/Bones\").",
    "Part 2: a white-tailed deer killed by the player (no drops now: \"skinning only\"; drawn as the deer lying dead) and two hungry",
    "turkey vultures: \"smells carrion\", \"animal N eats 0.04 kg off carcass M\" a few times each (a mouthful its size, one a",
    "chew) until full, the body still whole (two 140 g meals off a 40 kg deer). Part 3: a white-tailed",
    "deer (not a red deer: a Large animal's blow cap kept r1's alive) killed by the player, then aged by `/animalprof carcass <days>`: 3 days (opened), 5 (bones), 9 (a pile), 13 (gone).",
    "Part 4: skinning: a white-tailed deer killed with one blow, the player with a (flint) knife uses its carcass: kneels, the camera",
    "comes in beside them (shot 8 is taken mid-skin from the player's own camera), and after about 8 s \"skinned carcass N",
    "of white_tailed_deer: K base:hide (quality Q), M animals:venison\" (Q at most 2 with flint), the hide and meat on the ground.",
    "Part 5 (x 1000): a grey wolf, a golden eagle, a raptor, a yacare caiman and a pacu killed in a row: each drawn as its",
    "body plan's own lying model (four-legged, bird, two-legged, reptile, fish) in its own coat, shot whole (10), opened",
    "after 3 days (12: meat and ribs on the upper side, guts beside it, a blood pool under), its bones (14, 5 days) and",
    "pile (15, 9 days: a ribcage, skull, legs, a scrap of its hide); each stage as the row, the wolf from straight above",
    "(11, 13, 15, 16_wolf_above: legs out sideways flat on the ground) and each body close and low (_low_<stage>_<plan>).",
    "Report per part: the carcass, eats and skin lines, the `/animalprof carcass` rows, no hide or meat dropped at the kills,",
    "and the shots: the dead animal lying at its place, the ribcage and the bone pile sized to it, the kneel and the camera",
    "while skinning; anything floating, sunk, flickering or drawn wrong.",
])
x = 0
# The deer first, wounded to 30 % (no one behind the blow, so they don't flee anyone), then the wolves: r5's healthy
# deer took 8 hunts and lived; r4's died in one.
s += pad(x) + spawns(x, [("white_tailed_deer", 2, -6, 6)]) + ["chat /animalprof wound 0.3"]
s += spawns(x, [("grey_wolf", 3, 8, -10)])
# Hungrier than the hunters runs (r3's wolves rested 100 s at 30 %, and the part ended before their kill).
s += ["chat /animalprof food 0.9", "chat /animalprof water 0.9", "chat /animalprof hunters 0.15"]
s += watch(x, 12) + carcass_rows(4) + shot(x, "cov_carcass_1_wolves", z_back=18, pitch=-40)


def vultures_part(x, p="cov_carcass", rows=10):
    """Part 2: a white-tailed deer killed by the player, then two hungry turkey vultures at its carcass, each eating
    mouthfuls of its own size (`animals_core::carcass::bite_kg`: about 40 g, one a chew) until full."""
    out = pad(x) + spawns(x, [("white_tailed_deer", 1, 0, 6)])
    out += ["chat /tk mode creative", f"chat /tp {x} 71 {Z+10}", "look 0 -30", "wait 1", "chat /animalprof kill", "wait 2"]
    out += ["chat /animalprof carcass"] + side_shot(x, f"{p}_2_killed") + low_shot(x, f"{p}_2_low")
    out += spawns(x, [("turkey_vulture", 2, 10, -12)]) + ["chat /animalprof food 0.2"]
    out += ["chat /tk mode spectator", f"chat /tp {x} 90 {Z+20}", "wait 1"] + carcass_rows(rows)
    out += shot(x, f"{p}_3_vultures", z_back=8, pitch=-35) + side_shot(x, f"{p}_3_vultures_low")
    return out


x = 200
s += vultures_part(x)
x = 400
s += pad(x) + spawns(x, [("white_tailed_deer", 1, 0, 6)])
s += ["chat /tk mode creative", f"chat /tp {x} 71 {Z+10}", "look 0 -30", "wait 1", "chat /animalprof kill", "wait 2",
      "chat /tk mode spectator"]
for n, days in enumerate([3, 2, 4]):
    s += [f"chat /tp {x} 75 {Z+6}", f"chat /animalprof carcass {days}", "wait 2", "chat /animalprof carcass"]
    s += side_shot(x, f"cov_carcass_{4+n}_aged") + low_shot(x, f"cov_carcass_{4+n}_low")
s += [f"chat /tp {x} 75 {Z+6}", "chat /animalprof carcass 4", "wait 2", "chat /animalprof carcass"]
x = 600


def skin_part(x, prefix="cov_carcass"):
    """Part 4: a deer killed with one blow on the grass south of the pond (r3's died in the pond's pit, below the
    knife's aim), skinned with a flint knife, shot mid-skin from the skinning camera and after."""
    return pad(x) + spawns(x, [("white_tailed_deer", 1, 0, 6)]) + [
        "chat /tk mode creative", f"chat /tp {x} 70 {Z+10}", "look 0 -30", "wait 1", "chat /animalprof kill", "wait 2",
        "chat /animalprof carcass", "chat /tk put me 0 base:knife 1", "key 1", "wait 1",
        # Standing 2 blocks south of it, looking down at it (a positive pitch looks up).
        # r7's deer died 2 blocks west of the aim line and the use hit the grass: face the body itself (`aim`).
        f"chat /tp {x} 70 {Z+8}", "look 0 -40", "wait 1", "aim 6", "wait 1", "press core:use", "wait 4",
        # The hide and meat tossed from the body toward the skinner as it ends (the deer's 8.1 s with flint), held in
        # the air and landing (ticks frozen for each shot), then picked up 1.25 s later; looking down at the body and the
        # ground before the player (r4's level view after the skinning camera saw only the wall).
        f"shot {prefix}_8_skinning.png", "wait 4", "look 0 -50", "wait 0.5", "chat /tick freeze", f"shot {prefix}_8b_off.png",
        "chat /tick unfreeze", "wait 0.4", "chat /tick freeze", f"shot {prefix}_8c_off.png", "chat /tick unfreeze",
        "wait 4", "chat /animalprof carcass"] + \
        side_shot(x, f"{prefix}_9_skinned") + low_shot(x, f"{prefix}_9_low") + [
        # The hide's tooltip (its quality's stars, `tagged.base.hide_quality`): the inventory open, the cursor on the
        # hotbar row's second and third slots (the knife in the first; the hide, then the venison, after it), at GUI
        # scale 2 in 1280 x 720 (skin r3: the creative inventory, its palette on the left, the player's hotbar row
        # at the bottom right). Not hidden with the HUD.
        "press core:inventory", "wait 1", "cursor 0.547 0.710", "wait 1", f"shot {prefix}_10_tooltip_slot2.png",
        "cursor 0.576 0.710", "wait 1", f"shot {prefix}_10_tooltip_slot3.png", "press core:inventory", "wait 1"]


s += skin_part(x)
x = 1000
PLANS = [("wolf", "grey_wolf", -8), ("eagle", "golden_eagle", -4), ("raptor", "raptor", 0), ("caiman", "yacare_caiman", 4),
         ("pacu", "pacu", 8)]


def plans_part(x, p="cov_carcass"):
    """Slice 3: one of each body plan killed in a row, drawn as its plan's lying model in its own coat (a grey wolf's
    grey, not a deer's brown): whole, opened by 3 days' rot, its bones (5 days) and pile (9 days; the fish, rotting
    fastest, may be gone). West to east: a grey wolf, a golden eagle, a raptor, a yacare caiman, a pacu (on the grass).
    Each stage shot as the row, each body close and low from its south-east (r7's and r10's row shots had them as
    specks), and the wolf from straight above (r9's and r10's low shots from its belly side had its legs, lying out
    toward the camera, read as standing)."""
    out = pad(x) + ["chat /tk mode creative"]
    for _, kind, dx in PLANS:
        # Killed at once (r1: in the 2 s from spawn to kill the raptor walked 4 blocks east, onto the caiman's spot).
        out += [f"chat /tk spawn animals:{kind} {x+dx} 70 {Z+6}", "wait 0.2", f"chat /tp {x+dx} 70 {Z+9}", "wait 0.2",
                "chat /animalprof kill", "wait 1"]
    out += ["chat /tk mode spectator", "wait 2", "chat /animalprof carcass"]
    for days, n, stage in [(0, 10, "whole"), (3, 12, "opened"), (2, 14, "bones"), (4, 15, "pile")]:
        if days:
            out += [f"chat /tp {x} 75 {Z+6}", f"chat /animalprof carcass {days}", "wait 2", "chat /animalprof carcass"]
        out += ["chat /tick freeze", f"chat /tp {x} 76 {Z+16}", "look 0 -30", "wait 2", f"shot {p}_{n}_plans_{stage}.png",
                f"chat /tp {x-8} 72.8 {Z+6}", "look 0 -89", "wait 2", f"shot {p}_{n+1}_wolf_above_{stage}.png"]
        for name, _, dx in PLANS:
            out += [f"chat /tp {x+dx+1.6} 68.9 {Z+7.8}", "look 39 -9", "wait 2", f"shot {p}_{n}_low_{stage}_{name}.png"]
        if stage == "opened":
            # Its flies close, ticks frozen (Planner on sheet_rot), then running.
            out += [f"chat /tp {x-8+1.2} 68.9 {Z+7.2}", "look 39 -12", "wait 3", f"shot {p}_{n+1}_flies_wolf_frozen.png"]
        out += ["chat /tick unfreeze", "wait 1"]
        if stage == "opened":
            # Its flies (client particles, drawn with ticks running): the wolf at 3 days bloats, rot step 3.
            out += [f"chat /tp {x-8+1.2} 68.9 {Z+7.2}", "look 39 -12", "wait 3", f"shot {p}_{n+1}_flies_wolf.png"]
    return out


s += plans_part(x)
# Every shot without the HUD (Planner on r7: the chat log covered half of each frame): F1 off, shoot, F1 on.
s = [y for l in s for y in (["press core:hide_hud", "wait 0.5", l, "press core:hide_hud"] if l.startswith("shot ") and "tooltip" not in l else [l])]
write("carcass", s)

# Part 4 alone, for the kneel and the skinning camera (Planner on r9: the kneeling player read as oversized boxes).
s = head("carcass_skin", [
    "Skinning alone (cov_carcass part 4): a white-tailed deer killed with one blow, skinned with a flint knife. Shot 8:",
    "mid-skin from the skinning camera (the player kneeling beside the body, the knife, the body on its pool); 9: after;",
    "8b_off/8c_off (as the skinning ends): the hide and meat flying from the body toward the player and landing by them,",
    "before they are picked up (1.25 s later). 10_tooltip_slot2/3: the inventory open, the cursor on the hotbar row's 2nd and 3rd slots: the hide's tooltip with its",
    "stars (\"★★☆ Good hide\" or the like) in one of them.",
    "Report the \"skins carcass\" and \"skinned carcass\" lines and the shots.",
]) + skin_part(600, "cov_carcass_skin")
s = [y for l in s for y in (["press core:hide_hud", "wait 0.5", l, "press core:hide_hud"] if l.startswith("shot ") and "tooltip" not in l else [l])]
write("carcass_skin", s)

# Part 2 alone, longer, for the scavengers' mouthfuls (Planner 11-26: vultures filled after one 0.5 kg bite).
s = head("carcass_vultures", [
    "Vultures alone (cov_carcass part 2, watched 4 minutes): a white-tailed deer killed by the player, two hungry turkey",
    "vultures (food 0.2) come to it. Report each \"animal N eats K kg off carcass M: food F of MAX\" line: K about 0.04,",
    "a few per vulture 30 s apart, F rising to full, then it leaves; the carcass rows' kg eaten (about 0.3 at most), the",
    "body still whole; and the shots (3_vultures, 3_vultures_low): the vultures at or near the body.",
]) + vultures_part(200, "cov_carcass_vultures", rows=24)
s = [y for l in s for y in (["press core:hide_hud", "wait 0.5", l, "press core:hide_hud"] if l.startswith("shot ") and "tooltip" not in l else [l])]
write("carcass_vultures", s)

# Part 5 alone, for the body plans' looks (Planner on r10: the wolf read as standing; the bones and piles specks).
s = head("carcass_plans", [
    "Body plans alone (cov_carcass part 5): a grey wolf, a golden eagle, a raptor, a yacare caiman and a pacu killed in a",
    "row, then aged: whole, opened (3 days), bones (5), pile (9). Per stage: the row (_plans_), the wolf from straight",
    "above (_wolf_above_: legs out sideways flat on the ground, not under it), and each body close and low (_low_).",
    "Report whether each lies on its side (the caiman on its belly), rests on the ground, and is its own animal's shape.",
    "Rot: whole fresh in its own coat; opened (3 days, bloating) duller and greener; bones and pile (remains) a dry",
    "grey-brown. 13_flies_wolf: small dark flies darting over the opened wolf (ticks running). Report each carcass's",
    "\"rot step\" and \"flies\" from the /animalprof carcass lines. The blood pool: under the opened bodies a darker",
    "red-brown than r4's bright red, under the bones (remains) a dry dark brown-black stain, none under the pile.",
]) + plans_part(1000, "cov_carcass_plans")
s = [y for l in s for y in (["press core:hide_hud", "wait 0.5", l, "press core:hide_hud"] if l.startswith("shot ") and "tooltip" not in l else [l])]
write("carcass_plans", s)


def blood_rows(n, every=10):
    out = []
    for _ in range(n):
        out += ["chat /animalprof blood", f"wait {every}"]
    return out


# Blood trails (animals_core::blood, mods/animals/src/blood.rs; Planner 11-26).
s = head("blood", [
    "Blood trails. Part 1 (x 200): a white-tailed deer gashed for 70 % of its health bleeds as it runs, leaving spots",
    "of blood, and bleeds out in about 90 s; 1_gashed: close, where it was hit (a dark red splash on the grass);",
    "2_trail: from high above (dark red splashes in a line where it ran); 3_spots_low: low at the first spots. Then a",
    "hungry grey wolf set 14 blocks off follows the spots (\"grey_wolf N follows a blood trail\") to the carcass and",
    "eats (\"eats K kg off carcass\"); 4_wolf_1/2/3: the wolf on the trail from above (ticks frozen, 5, 9 and 13 s in: on or beside the splashes, walking them in order); 5_wolf_end. Part 2 (x 600): a deer gashed for 30 % bleeds 45 s and lives (\"stops bleeding\",",
    "health over half). Report the /animalprof blood rows (\"bleeds N s more, health\", the spot count rising then",
    "falling as spots dry), the GASH lines, \"wounded\", \"stops bleeding\", the carcass line where the first deer died,",
    "the wolf's lines, and any errors naming blood_spot or base:bleed.",
])
x = 200
s += pad(x) + spawns(x, [("white_tailed_deer", 1, 0, 6)])
s += ["chat /tk mode creative", f"chat /tp {x} 71 {Z+10}", "look 0 -30", "wait 1", "chat /animalprof gash 0.7",
      "wait 3", "chat /tk mode spectator"]
s += ["chat /tick freeze", f"chat /tp {x+2} 71 {Z+9}", "look 30 -40", "wait 2", "shot cov_blood_1_gashed.png",
      "chat /tick unfreeze", "wait 1"]
s += [f"chat /tp {x} 110 {Z}", "look 0 -89"] + blood_rows(4)
# From 21 blocks up (r1's from 36 showed specks a pixel or two across), ticks frozen.
s += ["chat /tick freeze", f"chat /tp {x+4} 90 {Z}", "look 0 -89", "wait 2", "shot cov_blood_2_trail.png",
      "chat /tick unfreeze", "wait 1"]
s += [f"chat /tp {x} 110 {Z}"] + blood_rows(5) + ["chat /animalprof carcass"]
s += ["chat /tick freeze", f"chat /tp {x+2} 69.6 {Z+9}", "look 30 -25", "wait 2", "shot cov_blood_3_spots_low.png",
      "chat /tick unfreeze", "wait 1"]
# A hungry wolf (food 0.1) 14 blocks west of where the deer was hit (on the pad), out of sight of where it died.
s += [f"chat /tp {x} 110 {Z}", f"chat /tk spawn animals:grey_wolf {x-14} 70 {Z+6}", "wait 2",
      "chat /animalprof food 0.1"]
# The wolf on the trail, ticks frozen, from above at 5, 9 and 13 s (Planner: a hunter following it; r1's at 18 s found
# it at the body already), then rows. r2's walked straight to the body: it now walks the spots in the order they fell.
for n, wait in enumerate([3, 4, 4]):
    s += [f"wait {wait}", "chat /animalprof blood", "chat /tick freeze", f"chat /tp {x+4} 90 {Z}", "look 0 -89",
          "wait 2", f"shot cov_blood_4_wolf_{n+1}.png", "chat /tick unfreeze", "wait 1"]
for _ in range(9):
    s += ["chat /animalprof carcass", "chat /animalprof blood", "wait 10"]
s += ["chat /tick freeze", f"chat /tp {x} 105 {Z}", "look 0 -89", "wait 2", "shot cov_blood_5_wolf_end.png",
      "chat /tick unfreeze", "wait 1"]
x = 600
s += pad(x) + spawns(x, [("white_tailed_deer", 1, 0, 6)])
s += ["chat /tk mode creative", f"chat /tp {x} 71 {Z+10}", "look 0 -30", "wait 1", "chat /animalprof gash 0.3",
      "wait 3", "chat /tk mode spectator", f"chat /tp {x} 110 {Z}"] + blood_rows(7)
s += ["chat /animalprof where 96", "chat /animalprof blood"]
s = [y for l in s for y in (["press core:hide_hud", "wait 0.5", l, "press core:hide_hud"] if l.startswith("shot ") else [l])]
write("blood", s)


def wall_part(x, p="cov_carcass_wall"):
    """A deer killed pressed against a stone wall (pt_rain_4: its carcass lay half in the wall, out of the knife's reach),
    and one in a corner: each carcass laid clear of the stone (`animals_core::carcass::clear_spot`), then the first
    skinned with a flint knife from the open side."""
    out = pad(x) + [
        "chat /tk mode creative",
        # A wall 3 high along the deer's north side (z Z+1 to Z+3: its south face at Z+4), and one along the second
        # deer's east side (x+7, x+8) to make a corner.
        f"chat /tk fill {x-5} 70 {Z+1} {x+8} 72 {Z+3} base:stone",
        f"chat /tk fill {x+7} 70 {Z+1} {x+8} 72 {Z+8} base:stone",
        # Spawned at the wall (a deer's middle 0.5 from it, its own box clear; a lying deer's 1.4 reaches 0.2 into it).
        f"chat /tk spawn animals:white_tailed_deer {x} 70 {Z+4}", "wait 0.2", f"chat /tp {x} 70 {Z+8}", "wait 0.2",
        "chat /animalprof kill", "wait 1",
        f"chat /tk spawn animals:white_tailed_deer {x+6} 70 {Z+4}", "wait 0.2", f"chat /tp {x+5} 70 {Z+8}", "wait 0.2",
        "chat /animalprof kill", "wait 2", "chat /animalprof carcass",
        # Both bodies at the wall, from above and low from the south.
        "chat /tick freeze", f"chat /tp {x+3} 76 {Z+10}", "look 0 -45", "wait 2", f"shot {p}_1_laid.png",
        f"chat /tp {x+3} 68.9 {Z+9}", "look 0 -8", "wait 2", f"shot {p}_2_low.png", "chat /tick unfreeze", "wait 1",
        # The first skinned from the open side: the knife reaches it.
        "chat /tk put me 0 base:knife 1", "key 1", "wait 1",
        f"chat /tp {x} 70 {Z+7}", "look 0 -40", "wait 1", "aim 6", "wait 1", "press core:use", "wait 4",
        f"shot {p}_3_skinning.png", "wait 8", "chat /animalprof carcass",
        "chat /tick freeze", f"chat /tp {x+3} 76 {Z+10}", "look 0 -45", "wait 2", f"shot {p}_4_skinned.png",
        "chat /tick unfreeze", "wait 1",
    ]
    return out


s = head("carcass_wall", [
    "Carcasses at a wall (Planner, from pt_rain_4: a deer that died against a stone wall left its carcass inside the",
    "wall's block, where the knife could not reach it). A white-tailed deer killed pressed to a stone wall's south face,",
    "one more in a corner (a wall north and east of it). Each gets \"carcass of N moved out of the blocks at ... to ...\"",
    "(z at least 0.7 south of the wall face, the corner one also 0.7 west of the east wall) and its \"carcass N of",
    "white_tailed_deer\" line at the new place. Then the first is skinned with a flint knife from the south: \"skins",
    "carcass\" and \"skinned carcass N\" with its hide and venison. Shots: 1_laid (from above) and 2_low: both bodies",
    "lying whole on the grass beside the stone, none of either inside it; 3_skinning (the kneel); 4_skinned.",
]) + wall_part(600)
s = [y for l in s for y in (["press core:hide_hud", "wait 0.5", l, "press core:hide_hud"] if l.startswith("shot ") else [l])]
write("carcass_wall", s)
