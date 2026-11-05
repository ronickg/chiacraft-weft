"""The canids' behaviour clips (`mods/animals/src/anim.rs`, `notes/ANIMALS.md` "Animations"), first
written for the wolf and shared by every dog-shaped animal on the house rig (the wolf's bones:
root, spine_0/1, neck, head, jaw, ears, eyelids that swing about x, legs upper/lower/paw, a tail
of `TAIL` segments). A species' build binds its own helpers and fitted numbers, then takes the
clips:

    import canid
    canid.bind(sys.modules[__name__])      # gait, tail_swing, tail_sway, TAIL, L, pose_lie...
    ANIMS.update(canid.anims(howl=True))   # and LOOPS, BLENDS, BLINKS from canid
    canid.fit(spec, GROUPS)                # in its fit_poses: head down to the ground, the death fall

`L` needs "lie_drop" (its lying pose's) and gets "low_neck", "low_head", "low_crouch", "low_drop"
and "death_drop" from `fit`. Gaits (stalk, approach) are grounded in the build (`ground.grounded`).
"""

import math

import numpy as np

S = math.sin
TAU = 2 * math.pi
NEEDS = ("gait", "tail_swing", "tail_sway", "TAIL", "L", "pose_lie")
gait = tail_swing = tail_sway = pose_lie = None
TAIL, L = [], {}
# A species' own scales for what its body allows: how far the scratch turns its neck, how hard
# play wags its tail (a fox's short neck and big brush poke through at the wolf's).
K = {"scratch": 1.0, "wag": 1.0}


def bind(m, **scales):
    """Uses species module `m`'s helpers and numbers (`NEEDS`); its `L` gets the fitted ones;
    `scales` (`K`) where its body differs."""
    K.update(scales)
    g = globals()
    for n in NEEDS:
        g[n] = getattr(m, n)
    for k, v in {"low_neck": 60, "low_head": 20, "low_crouch": 0.0, "low_drop": 0.0, "death_drop": 8}.items():
        g["L"].setdefault(k, v)


def still(t, period=4.0):
    """Standing still and breathing, nothing else: what a clip builds on."""
    br = S(TAU * t / period * 2)
    q = {"root": (0, 0.1 * br, 0), "spine_1": (0.5 * br, 0, 0)}
    q.update(tail_sway(t / period, 1, 2))
    return q


def crouch(k):
    """The legs bent by `k` (1: the stalk's crouch), feet still flat."""
    q = {}
    for leg in ("FL", "FR"):
        q.update({f"leg_{leg}_upper": (22 * k, 0, 0), f"leg_{leg}_lower": (-30 * k, 0, 0), f"leg_{leg}_paw": (8 * k, 0, 0)})
    for leg in ("BL", "BR"):
        q.update({f"leg_{leg}_upper": (-18 * k, 0, 0), f"leg_{leg}_lower": (28 * k, 0, 0), f"leg_{leg}_paw": (-10 * k, 0, 0)})
    return q


def head_down(P=None, d=1.0):
    """Head down to the ground by `d` (1: the nose half a unit over it, fitted): the legs bent, the
    body dropped onto them, the neck and head lowered."""
    P = L if P is None else P
    q = crouch(P["low_crouch"] * d)
    q.update({"root": (0, -P["low_drop"] * d, 0), "neck": (P["low_neck"] * d, 0, 0), "head": (P["low_head"] * d, 0, 0)})
    return q


def pose_stare(t):
    """Half aware of something: frozen, head up and pointed at it, ears pricked, the tail held
    still and level; only the breath moves (2 s, held)."""
    q = still(t, 2.0)
    q.update({"neck": (-10, 0, 0), "head": (-4, 0, 0), "ear_L": (-10, 0, 0), "ear_R": (-10, 0, 0)})
    q.update({f"tail_{k}": tail_swing(k, 0, (10 if k == 0 else 0, 0, 0)) for k in range(len(TAIL))})
    return q


def pose_sniff(t):
    """Nose to the ground, sniffing along a little, and up again (2.5 s, once)."""
    p = t / 2.5
    d = float(np.interp(p, [0, .2, .8, 1], [0, 1, 1, 0]))
    q = still(t)
    q.update(head_down(d=d))
    q["neck"] = (q["neck"][0], 6 * d * S(TAU * p), 0)
    q.update({"jaw": (2 * d * max(0.0, S(TAU * p * 8)), 0, 0), "ear_L": (6 * d, 0, 0), "ear_R": (6 * d, 0, 0)})
    return q


def pose_drink(t):
    """Head down at the water, lapping (1 s, held)."""
    p = t / 1.0
    q = still(t, 2.0)
    q.update(head_down())
    q["head"] = (q["head"][0] + 3 * S(TAU * p * 2), 0, 0)
    q["jaw"] = (10 * max(0.0, S(TAU * p * 2)), 0, 0)
    return q


def pose_look(t):
    """Looking round: the head turns one way, holds, the other way, back (3 s, once)."""
    p = t / 3.0
    yaw = float(np.interp(p, [0, .15, .4, .55, .8, 1], [0, 45, 45, -40, -40, 0]))
    q = still(t)
    q.update({"neck": (-6, 0.7 * yaw, 0), "head": (-2, 0.3 * yaw, 0), "ear_L": (-6, -0.3 * yaw, 0), "ear_R": (-6, -0.3 * yaw, 0)})
    return q


def pose_scratch(t):
    """Nibbling an itch on its flank: the head turned round to the side, the teeth working (2 s, once)."""
    p = t / 2.0
    d = float(np.interp(p, [0, .2, .85, 1], [0, 1, 1, 0]))
    q = still(t)
    k = K["scratch"]
    q.update({"spine_1": (0, 16 * d, 0), "neck": (18 * d * k, 55 * d * k, 0), "head": (8 * d, 14 * d, 0),
              "jaw": (10 * d * max(0.0, S(TAU * p * 6)), 0, 0), "ear_L": (10 * d, 0, 0), "ear_R": (10 * d, 0, 0)})
    return q


def pose_stretch(t):
    """The bow stretch on getting up: front legs reaching out, chest down, rump up, then the
    back legs pushed out behind (2.5 s, once)."""
    p = t / 2.5
    bow = float(np.interp(p, [0, .15, .45, .55, 1], [0, 1, 1, 0, 0]))
    back = float(np.interp(p, [0, .55, .7, .9, 1], [0, 0, 1, 1, 0]))
    q = still(t)
    q.update({"root": (0, -2.5 * bow, 0), "spine_0": (-10 * bow + 4 * back, 0, 0), "spine_1": (16 * bow, 0, 0),
              "neck": (-10 * bow, 0, 0), "head": (-8 * bow, 0, 0), "jaw": (14 * bow, 0, 0)})
    for leg in ("FL", "FR"):
        q[f"leg_{leg}_upper"] = (-55 * bow, 0, 0)
        q[f"leg_{leg}_lower"] = (10 * bow, 0, 0)
        q[f"leg_{leg}_paw"] = (30 * bow, 0, 0)
    for leg in ("BL", "BR"):
        q[f"leg_{leg}_upper"] = (12 * bow + 30 * back, 0, 0)
        q[f"leg_{leg}_lower"] = (-8 * back, 0, 0)
        q[f"leg_{leg}_paw"] = (-12 * bow - 22 * back, 0, 0)
    q.update(tail_sway(p, 2, 6, [(14 * bow, 0, 0)] + [(0, 0, 0)] * (len(TAIL) - 1)))
    q["tail_0"] = tail_swing(0, 0, (14 * bow, 0, 0))
    return q


def pose_sleep(t, P=None):
    """Asleep: lying down, curled a little more, the head on its paws, eyes shut, slow breath (6 s, held)."""
    P = L if P is None else P
    q = pose_lie(t * 5.0 / 6.0, P)
    br = S(TAU * t / 6.0 * 2)
    q.update({"spine_1": (0.8 * br, 0, 0), "neck": (30, 0, 0), "head": (-18, 4, 0), "ear_L": (14, 0, 0), "ear_R": (14, 0, 0)})
    q["root"] = (0, -P["lie_drop"] + 0.15 * br, 0)
    q.update({"eyelid_L": (-90, 0, 0), "eyelid_R": (-90, 0, 0)})
    return q


def pose_stalk(t):
    """Stalking: crouched low, head level and pointed ahead, ears forward, tail low and still,
    slow deliberate steps (1.2 s a cycle, in place of the walk)."""
    p = t / 1.2
    q = gait(p, (0.0, 0.5, 0.75, 0.25), 14, 30)
    # The legs bent the way a step lifts them, the paws kept flat: each pair 3.2 units shorter, so the
    # body sinks 3.1 with the feet on the ground (bent the other way the feet sank 4 units into it
    # and the gait moonwalked: the lift straightened the knee, so a foot was lowest mid-swing).
    for leg in ("FL", "FR", "BL", "BR"):
        bend = (-40, 60, -20) if leg[0] == "F" else (30, -50, 20)
        for part, b in zip(("upper", "lower", "paw"), bend):
            q[f"leg_{leg}_{part}"] = (q[f"leg_{leg}_{part}"][0] + b, 0, 0)
    q.update({"root": (0, -3.1, 0), "spine_1": (2, 0, 0), "neck": (16, 0, 0), "head": (-14, 0, 0), "ear_L": (-12, 0, 0), "ear_R": (-12, 0, 0)})
    q.update({f"tail_{k}": tail_swing(k, 0, (30 if k == 0 else -2, 0, 0)) for k in range(len(TAIL))})
    return q


def pose_approach(t):
    """Curious, coming to look: slow steps, the head low and stretched forward, nose working, the
    ears pricked ahead, the tail level (1.4 s a cycle, in place of the walk). Not a stalk's crouch."""
    p = t / 1.4
    q = gait(p, (0.0, 0.5, 0.75, 0.25), 16, 32)
    q.update({"root": (0, -1.0, 0), "spine_1": (2, 0, 0), "neck": (14, 0, 0), "head": (-10 + 2 * S(TAU * p * 4), 0, 0),
              "ear_L": (-10, 0, 0), "ear_R": (-10, 0, 0)})
    q.update({f"tail_{k}": tail_swing(k, 0, (16 if k == 0 else 0, 0, 0)) for k in range(len(TAIL))})
    return q


def pose_alarm(t):
    """Alarm: the head snaps up, ears high, a sharp bark-huff, the tail raised stiff (1.2 s, once)."""
    p = t / 1.2
    up = float(np.interp(p, [0, .12, .75, 1], [0, 1, 1, 0]))
    bark = float(np.interp(p, [0, .2, .3, .4, .5, .6, 1], [0, 0, 1, 0, 1, 0, 0]))
    q = still(t)
    q.update({"spine_1": (-4 * up, 0, 0), "neck": (-22 * up, 0, 0), "head": (-6 * up, 0, 0), "jaw": (18 * bark, 0, 0),
              "ear_L": (-14 * up, 0, 0), "ear_R": (-14 * up, 0, 0)})
    q.update({f"tail_{k}": tail_swing(k, 0, (-10 * up if k == 0 else -8 * up, 0, 0)) for k in range(len(TAIL))})
    return q


def pose_howl(t):
    """The howl: the head thrown back, the muzzle up, the mouth rounding and closing slowly (4 s, once)."""
    p = t / 4.0
    howl = float(np.interp(p, [0, .12, .85, 1], [0, 1, 1, 0]))
    yip = howl * (0.7 + 0.3 * max(0.0, S(TAU * p * 2)))
    q = still(t)
    q.update({"spine_1": (-6 * howl, 0, 0), "neck": (-40 * howl, 0, 0), "head": (-14 * howl, 0, 0), "jaw": (22 * yip, 0, 0),
              "ear_L": (12 * howl, 0, 0), "ear_R": (12 * howl, 0, 0)})
    q.update(tail_sway(p, 1, 2))
    return q


def pose_play(t):
    """Play: the bow, then a bounce up and a wheel to the side, the tail wagging hard (1.6 s, once)."""
    p = t / 1.6
    bow = float(np.interp(p, [0, .12, .4, .5, 1], [0, 1, 1, 0, 0]))
    hop = max(0.0, S(TAU * (p - 0.5) * 2)) if p > 0.5 else 0.0
    q = still(t)
    q.update({"root": (0, -2.5 * bow + 3.0 * hop, 0), "spine_0": (-10 * bow, 0, 0), "spine_1": (16 * bow, 30 * hop, 0),
              "neck": (-10 * bow, 20 * hop, 0), "head": (-8 * bow, 0, 12 * hop), "jaw": (12 * (bow + hop), 0, 0)})
    for leg in ("FL", "FR"):
        q[f"leg_{leg}_upper"] = (-55 * bow - 25 * hop, 0, 0)
        q[f"leg_{leg}_lower"] = (10 * bow + 30 * hop, 0, 0)
        q[f"leg_{leg}_paw"] = (30 * bow, 0, 0)
    for leg in ("BL", "BR"):
        q[f"leg_{leg}_upper"] = (12 * bow, 0, 0)
        q[f"leg_{leg}_paw"] = (-12 * bow, 0, 0)
    q.update(tail_sway(p, 6, 16 * K["wag"], [(-8, 0, 0)] + [(-6 * K["wag"], 0, 0)] * (len(TAIL) - 1)))
    q["tail_0"] = tail_swing(0, 0, (-8, 0, 0))  # the wag in the brush, its root still
    return q


def pose_groom(t):
    """Grooming: licking a front leg, the head down to it, the jaw working (2.5 s, once)."""
    p = t / 2.5
    d = float(np.interp(p, [0, .2, .85, 1], [0, 1, 1, 0]))
    q = still(t)
    q.update({"neck": (36 * d, 10 * d, 0), "head": (12 * d, 4 * d, 0), "jaw": (8 * d * max(0.0, S(TAU * p * 5)), 0, 0),
              "leg_FL_upper": (-20 * d, 0, 0), "leg_FL_lower": (24 * d, 0, 0)})
    return q


def pose_nuzzle(t):
    """Nuzzling a young one: the head low and forward, a gentle push and a lick, the tail soft (2 s, once)."""
    p = t / 2.0
    d = float(np.interp(p, [0, .2, .85, 1], [0, 1, 1, 0]))
    push = max(0.0, S(TAU * p * 2))
    q = still(t)
    q.update({"neck": (32 * d + 4 * push, 0, 0), "head": (6 * d - 6 * push, 0, 0), "jaw": (6 * d * push, 0, 0),
              "ear_L": (12 * d, 0, 0), "ear_R": (12 * d, 0, 0)})
    q.update(tail_sway(p, 3, 6))
    q["tail_0"] = tail_swing(0, 0)
    return q


def pose_flinch(t):
    """Hit: a jerk away, the head ducking, ears flat, the tail tucked, and back (0.45 s, once)."""
    p = t / 0.45
    d = float(np.interp(p, [0, .15, .45, 1], [0, 1, 0.8, 0]))
    q = still(t)
    q.update({"root": (0, -1.0 * d, 0), "spine_1": (4 * d, 0, 2 * d), "neck": (14 * d, 0, 0), "head": (10 * d, 0, 0),
              "ear_L": (28 * d, 0, 0), "ear_R": (28 * d, 0, 0)})
    q.update({f"tail_{k}": tail_swing(k, 0, (40 * d if k == 0 else 8 * d, 0, 0)) for k in range(len(TAIL))})
    return q


def pose_death(t, P=None):
    """Dying: the legs give, it rolls onto its side, the head drops; it stays (1.2 s, holds)."""
    P = L if P is None else P
    p = t / 1.2
    fall = float(np.interp(p, [0, .35, .7, 1], [0, 0.35, 1, 1]))
    q = {"root": (0, -P["death_drop"] * fall, 0), "spine_0": (0, 0, 84 * fall), "neck": (14 * fall, 0, -10 * fall),
         "head": (8 * fall, 0, 0), "jaw": (8 * fall, 0, 0), "ear_L": (20 * fall, 0, 0), "ear_R": (20 * fall, 0, 0)}
    for leg in ("FL", "FR", "BL", "BR"):
        q[f"leg_{leg}_upper"] = ((10 if leg[0] == "F" else -10) * fall, 0, 0)
    q.update({f"tail_{k}": tail_swing(k, 0, (4 * fall, 0, 0)) for k in range(len(TAIL))})
    q.update({"eyelid_L": (-90 * fall, 0, 0), "eyelid_R": (-90 * fall, 0, 0)})
    return q


def anims(howl=False):
    """{clip: (pose, seconds)}: every behaviour clip (the howl only for a kind that howls)."""
    a = {"stare": (pose_stare, 2.0), "sniff": (pose_sniff, 2.5), "drink": (pose_drink, 1.0), "look": (pose_look, 3.0),
         "scratch": (pose_scratch, 2.0), "stretch": (pose_stretch, 2.5), "sleep": (pose_sleep, 6.0), "stalk": (pose_stalk, 1.2),
         "alarm": (pose_alarm, 1.2), "play": (pose_play, 1.6), "groom": (pose_groom, 2.5), "nuzzle": (pose_nuzzle, 2.0),
         "flinch": (pose_flinch, 0.45), "death": (pose_death, 1.2), "approach": (pose_approach, 1.4)}
    if howl:
        a["howl"] = (pose_howl, 4.0)
    return a


# How each clip loops and eases (`to_bedrock.as_action`): held ones loop, the rest play once; the
# flinch snaps in, the death holds its last frame. Blinks: when the eyes shut in each.
LOOPS = {"stare": True, "drink": True, "sleep": True, "stalk": True, "approach": True, "death": "hold_on_last_frame"}
BLENDS = {"flinch": (0.05, 0.2), "death": (0.1, 0.0), "stalk": (0.3, 0.3), "approach": (0.3, 0.3), "sleep": (0.6, 0.6)}
BLINKS = {"stare": [1.4], "look": [1.5], "stalk": [], "drink": [0.5]}
SHUT = ("sleep", "death")  # clips that keep the eyes shut (no blinks laid over them)
FRAMES = {"sleep": 36, "stalk": 24, "play": 30, "stretch": 30, "howl": 32, "flinch": 12, "death": 24, "approach": 28}


def fit(spec, groups):
    """Fits head down (sniffing, drinking: the nose half a unit over the ground, the legs bent as
    little as it takes) and the death fall (dropped until nothing is below the ground) into `L`.
    `groups` needs "front paws", "back paws" and "head" (`ground.lowest`)."""
    from ground import lowest
    from rig import fit as grid_fit

    def low_pose(P):
        g = lowest(spec, head_down(dict(L, **P, low_drop=0)), groups)
        drop = min(g["front paws"], g["back paws"])
        return g["head"] - drop, drop

    def low_cost(P):
        nose, _ = low_pose(P)
        return (nose - 0.5) ** 2 + 0.0002 * (P["low_neck"] ** 2 + P["low_head"] ** 2) + 2.0 * P["low_crouch"] ** 2
    best = grid_fit({"low_neck": list(range(40, 131, 5)), "low_head": list(range(0, 61, 5)), "low_crouch": [0, .5, 1, 1.5, 2, 2.5, 3]},
                    low_cost)[1]
    nose, drop = low_pose(best)
    L.update(best, low_drop=round(float(drop), 2))
    print("head down, nose over the ground:", round(float(nose), 2), best, "dropped", L["low_drop"])
    g = lowest(spec, pose_death(1.2, dict(L, death_drop=0)), {"all": lambda n: True})
    L["death_drop"] = round(float(g["all"]), 2)
    print("dead, dropped", L["death_drop"])
