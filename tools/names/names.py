#!/usr/bin/env python3
"""chiacraft's naming tool: every invented word comes from one lexicon and one check.

    python3 tools/names/names.py gen --people elvish -n 40 [--seed N]
    python3 tools/names/names.py derive eskenai --steps
    python3 tools/names/names.py derive esk+tem
    python3 tools/names/names.py check
    python3 tools/names/names.py sheet > notes/NAMES.md
    python3 tools/names/names.py selftest        (check catches every fault in faults.toml)

The lexicon is mods/base/lore/lexicon.toml: the peoples' sounds and syllables, the roots, the drifts
(sound changes, in SCA² syntax), the names with their builds, banned patterns and bad words.
Why: reports/Naming and invented words.md. Standard library only (Python 3.11+ for tomllib).

`gen` only offers: a person picks the roots. `check` fails on unknown roots, names that don't
re-derive (unless `irregular = "why"`), syllable-rule breaks, banned patterns and bad words, names of
one kind one edit apart, lang keys missing from any language their mod has, drifts whose
output breaks the syllable rules, and a stale notes/NAMES.md.
"""

import argparse
import random
import re
import sys
import tomllib
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
LEXICON = REPO / "mods" / "base" / "lore" / "lexicon.toml"
SHEET = REPO / "notes" / "NAMES.md"
SHEET_CMD = "python3 tools/names/names.py sheet > notes/NAMES.md"


# ---------------------------------------------------------------------------------------------
# Sounds
# ---------------------------------------------------------------------------------------------

def tokenize(word, multi):
    """Split a word into sounds, reading the given two-letter sounds as one (longest first)."""
    multi = sorted(multi, key=len, reverse=True)
    word = word.lower()
    out, i = [], 0
    while i < len(word):
        for m in multi:
            if word.startswith(m, i):
                out.append(m)
                i += len(m)
                break
        else:
            out.append(word[i])
            i += 1
    return out


def spell(segs):
    return "".join(segs)


def levenshtein(a, b):
    prev = list(range(len(b) + 1))
    for i, ca in enumerate(a, 1):
        cur = [i]
        for j, cb in enumerate(b, 1):
            cur.append(min(prev[j] + 1, cur[j - 1] + 1, prev[j - 1] + (ca != cb)))
        prev = cur
    return prev[-1]


def styled(word, like):
    """`word` capitalised the way `like` is (each word of a name)."""
    if like and like[0].isupper():
        return " ".join(w[:1].upper() + w[1:] for w in word.split(" "))
    return word


# ---------------------------------------------------------------------------------------------
# SCA² rules: target/replacement/environment[/exception]
# ---------------------------------------------------------------------------------------------

class RuleError(Exception):
    pass


class Rule:
    def __init__(self, sca, says, cats, multi):
        self.sca, self.says, self.cats = sca, says, cats
        parts = sca.split("/")
        if len(parts) not in (3, 4):
            raise RuleError(f"{sca!r}: want target/replacement/environment[/exception]")
        target, repl, env = parts[:3]
        exc = parts[3] if len(parts) == 4 else ""
        self.targets = [self._seq(t, multi) for t in target.split(",")] if target else [[]]
        reps = [self._seq(r, multi) for r in repl.split(",")] if repl else [[]]
        if len(reps) == 1:
            reps = reps * len(self.targets)
        if len(reps) != len(self.targets):
            raise RuleError(f"{sca!r}: {len(self.targets)} targets but {len(reps)} replacements")
        self.reps = reps
        self.env = self._env(env, multi)
        self.exc = self._env(exc, multi) if exc else None
        for t, r in zip(self.targets, self.reps):
            for e in r:
                if e[0] == "cat" and not (len(t) == 1 and t[0][0] == "cat"
                                          and len(cats[t[0][1]]) == len(cats[e[1]])):
                    raise RuleError(f"{sca!r}: category replacement needs a category target of the same size")

    def _seq(self, text, multi):
        """Elements: ('lit', sound), ('cat', name), ('#',), ('opt', element)."""
        out, i = [], 0
        while i < len(text):
            c = text[i]
            if c == "#":
                out.append(("#",))
                i += 1
            elif c == "(":
                j = text.index(")", i)
                inner = self._seq(text[i + 1:j], multi)
                if len(inner) != 1:
                    raise RuleError(f"{self.sca!r}: one element per ( )")
                out.append(("opt", inner[0]))
                i = j + 1
            elif c.isupper():
                if c not in self.cats:
                    raise RuleError(f"{self.sca!r}: unknown category {c}")
                out.append(("cat", c))
                i += 1
            elif c.isalpha():
                j = i
                while j < len(text) and text[j].isalpha() and text[j].islower():
                    j += 1
                out.extend(("lit", s) for s in tokenize(text[i:j], multi))
                i = j
            else:
                raise RuleError(f"{self.sca!r}: can't read {c!r}")
        return out

    def _env(self, text, multi):
        if text.count("_") != 1:
            raise RuleError(f"{self.sca!r}: an environment needs one _")
        left, right = text.split("_")
        return self._seq(left, multi), self._seq(right, multi)

    def _is(self, e, seg):
        if e[0] == "lit":
            return seg == e[1]
        if e[0] == "cat":
            return seg in self.cats[e[1]]
        return False

    def _right(self, elems, w, i):
        if not elems:
            return True
        e, rest = elems[0], elems[1:]
        if e[0] == "opt":
            return self._right([e[1]] + rest, w, i) or self._right(rest, w, i)
        if e[0] == "#":
            return i == len(w) and self._right(rest, w, i)
        return i < len(w) and self._is(e, w[i]) and self._right(rest, w, i + 1)

    def _left(self, elems, w, i):
        if not elems:
            return True
        e, rest = elems[-1], elems[:-1]
        if e[0] == "opt":
            return self._left(rest + [e[1]], w, i) or self._left(rest, w, i)
        if e[0] == "#":
            return i == 0 and self._left(rest, w, i)
        return i > 0 and self._is(e, w[i - 1]) and self._left(rest, w, i - 1)

    def _where(self, env, w, start, end):
        return self._left(env[0], w, start) and self._right(env[1], w, end)

    def apply(self, w):
        """All matches found on the word as it was, then replaced at once (as SCA² does)."""
        out, i, changed = [], 0, False
        while i <= len(w):
            done = False
            for t, r in sorted(zip(self.targets, self.reps), key=lambda tr: -len(tr[0])):
                n = len(t)
                if i + n > len(w):
                    continue
                if not all(self._is(e, w[i + k]) for k, e in enumerate(t)):
                    continue
                if not self._where(self.env, w, i, i + n):
                    continue
                if self.exc and self._where(self.exc, w, i, i + n):
                    continue
                new = []
                for e in r:
                    if e[0] == "cat":
                        src = self.cats[t[0][1]]
                        new.append(self.cats[e[1]][src.index(w[i])])
                    else:
                        new.append(e[1])
                out.extend(new)
                if n == 0:  # an insertion: keep the sound that follows
                    if i < len(w):
                        out.append(w[i])
                    i += 1
                else:
                    i += n
                changed = True
                done = True
                break
            if not done:
                if i < len(w):
                    out.append(w[i])
                i += 1
        return out if changed else list(w)


class Drift:
    def __init__(self, key, data, rules):
        self.key = key
        self.title = data.get("title", key)
        self.used_for = data.get("used_for", "")
        self.rules = rules

    def apply(self, segs, trace=None):
        for n, rule in enumerate(self.rules, 1):
            new = rule.apply(segs)
            if new != segs and trace is not None:
                trace.append((n, rule, spell(segs), spell(new)))
            segs = new
        return segs


# ---------------------------------------------------------------------------------------------
# The lexicon
# ---------------------------------------------------------------------------------------------

class Profile:
    """A people's sounds and syllable rules."""

    def __init__(self, key, d, cats):
        self.key, self.d = key, d
        self.title = d.get("title", key)
        self.source = d.get("from", "old")
        self.drift = d.get("drift")
        self.joins = d.get("joins", "eskenai")
        self.stress = d.get("stress", "first")
        self.loan_only = list(d.get("loan_only", []))
        self.vowel_w = self._weights(d.get("vowels", []))
        self.cons_w = self._weights(d.get("consonants", []))
        vowels_all = set(cats["V"])
        self.vowels = set(self.vowel_w) | {s for s in self.loan_only if s in vowels_all}
        self.consonants = set(self.cons_w) | {s for s in self.loan_only if s not in vowels_all}
        self.final_vowel = d.get("final_vowel", True)
        self.gen = d.get("gen", {})

    @staticmethod
    def _weights(v):
        return dict(v) if isinstance(v, dict) else {s: 1 for s in v}

    def clusters(self, name, multi):
        return {tuple(tokenize(c, multi)) for c in self.d.get(name, [])}


class Lexicon:
    def __init__(self, path=LEXICON):
        self.path = Path(path)
        with open(self.path, "rb") as f:
            self.d = tomllib.load(f)
        d = self.d
        self.sounds = d["sounds"]
        self.cats = d["categories"]
        self.kinds = d.get("kinds", [])
        self.profiles = {k: Profile(k, v, self.cats) for k, v in d["people"].items()}
        self.roots = d.get("root", {})
        self.endings = d.get("ending", {})
        self.dw_roots = d.get("dwarvish_root", {})
        self.dw_endings = d.get("dwarvish_ending", {})
        self.names = d.get("name", {})
        self.problems = []
        self.drifts = {}
        for key in d.get("drift", {}):
            self._drift(key, set())

    def _drift(self, key, seen):
        if key in self.drifts:
            return self.drifts[key]
        if key in seen:
            raise RuleError(f"drift {key} extends itself")
        data = self.d["drift"].get(key)
        if data is None:
            raise RuleError(f"unknown drift {key}")
        rules = []
        if "extends" in data:
            rules = list(self._drift(data["extends"], seen | {key}).rules)
        for r in data.get("rules", []):
            try:
                rules.append(Rule(r["sca"], r.get("says", ""), self.cats, self.sounds["rules"]))
            except RuleError as e:
                self.problems.append(f"drift {key}: {e}")
        self.drifts[key] = Drift(key, data, rules)
        return self.drifts[key]

    def multi(self, people):
        return self.sounds.get(people, self.sounds["old"])

    def is_vowel(self, seg):
        return seg in self.cats["V"]

    # -- building words ------------------------------------------------------------------

    def part(self, tongue, p):
        """A build part's sounds, or None if it isn't in the lexicon."""
        if tongue == "dwarvish":
            if p.startswith("-"):
                return tokenize(p[1:], self.sounds["dwarvish"]) if p[1:] in self.dw_endings else None
            return tokenize(p, self.sounds["dwarvish"]) if p in self.dw_roots else None
        if p.startswith("-"):
            return tokenize(p[1:], self.sounds["old"]) if p[1:] in self.endings else None
        return tokenize(p, self.sounds["old"]) if p in self.roots else None

    def join_old(self, parts, steps=None):
        word, prev = [], None
        for name, p in parts:
            if not word:
                word = list(p)
                prev = name
                continue
            a, b = word[-1], p[0]
            before = spell(word)
            if self.is_vowel(a) and self.is_vowel(b):
                if a in self.cats["P"]:
                    word.pop()
                    why = "two vowels meet, the first drops"
                else:
                    word[-1:] = [a[0], {"i": "y", "u": "w"}[a[1]]]
                    why = f"two vowels meet, {a} ends in {word[-1]}"
            else:
                tail = 0
                while tail < len(word) and not self.is_vowel(word[-1 - tail]):
                    tail += 1
                head = 0
                while head < len(p) and not self.is_vowel(p[head]):
                    head += 1
                if tail + head >= 3:
                    word.append("e")
                    why = "three consonants meet, e between"
                elif tail and head and a == b:
                    word.append("e")
                    why = "two of the same meet, e between"
                else:
                    why = None
            word += p
            if steps is not None:
                steps.append(f"{before} + {name}" + (f": {why}" if why else "") + f" → {spell(word)}")
            prev = name
        return word

    def join_dwarvish(self, parts, steps=None):
        word = []
        for name, p in parts:
            before = spell(word)
            if word and word[-1] == p[0] and not self.is_vowel(p[0]):
                word += p[1:]
                if steps is not None:
                    steps.append(f"{before} + {name}: {p[0]}{p[0]} becomes {p[0]} → {spell(word)}")
            else:
                word += p
                if steps is not None and before:
                    steps.append(f"{before} + {name} → {spell(word)}")
        return word

    def build_words(self, build, tongue, joins, steps=None):
        """Split a build at " " and join each word's parts. Returns (words, unknown parts)."""
        words, cur, unknown = [], [], []
        for p in list(build) + [" "]:
            if p == " ":
                if cur:
                    words.append(cur)
                cur = []
                continue
            segs = self.part(tongue, p)
            if segs is None:
                unknown.append(p)
                segs = tokenize(p.lstrip("-"), self.multi(tongue))
            cur.append((p, segs))
        joined = []
        for parts in words:
            if joins == "dwarvish":
                joined.append((self.join_dwarvish(parts, steps), parts))
            else:
                joined.append((self.join_old(parts, steps), parts))
        return joined, unknown

    def drift_for(self, profile, home=None):
        key = profile.drift
        if home:
            h = profile.d.get("homes", {}).get(home)
            if h and h.get("drift"):
                key = h["drift"]
        return self.drifts.get(key) if key else None

    def derive(self, build, people, home=None, steps=None, trace=None):
        """The shown form of a build in a people's tongue, as a lower-case string."""
        prof = self.profiles[people]
        tongue = "dwarvish" if prof.source == "dwarvish" else "old"
        words, unknown = self.build_words(build, tongue, prof.joins, steps)
        drift = self.drift_for(prof, home)
        out = []
        for segs, _ in words:
            if drift:
                segs = drift.apply(segs, trace)
            out.append(spell(segs))
        return " ".join(out), unknown

    def old_form(self, build):
        words, unknown = self.build_words(build, "old", "eskenai")
        return [segs for segs, _ in words], unknown

    def drifted(self, build, drift_key):
        words, _ = self.old_form(build)
        return " ".join(spell(self.drifts[drift_key].apply(w)) for w in words)

    def etymon(self, name):
        """The Old roots under a name, loans expanded, so the same word in two tongues matches."""
        prof = self.profiles.get(name.get("people"))
        out = []
        for p in name.get("build", []):
            if prof and prof.source == "dwarvish":
                r = self.dw_roots.get(p)
                out += list(r["loan"]) if r and "loan" in r else [f"dw:{p}"]
            else:
                out.append(p)
        return tuple(out)

    # -- syllables -----------------------------------------------------------------------

    def syllables(self, word, people, root=False):
        """A word split into syllables by its people's rules, or (None, why). A root on its own may
        end in any coda a word may have inside it (kep, as in Kepesk)."""
        prof = self.profiles[people]
        multi = self.multi(people)
        segs = tokenize(word, multi)
        onsets = prof.clusters("onsets", multi)
        initial = prof.clusters("initial_onsets", multi)
        codas = prof.clusters("codas", multi)
        finals = prof.clusters("final_codas", multi)
        if root:
            finals = finals | codas
        for s in segs:
            if s not in prof.vowels and s not in prof.consonants:
                return None, f"'{s}' isn't a sound of {prof.title}"
        if not any(s in prof.vowels for s in segs):
            return None, "no vowel"

        def ok_onset(c, first):
            if not c:
                return first
            if len(c) == 1:
                return True
            return c in onsets or (first and c in initial)

        def parse(i, first):
            j = i
            while j < len(segs) and segs[j] in prof.consonants:
                j += 1
            onset = tuple(segs[i:j])
            if j == len(segs):
                return None
            if not ok_onset(onset, first):
                return None
            k = j + 1
            m = k
            while m < len(segs) and segs[m] in prof.consonants:
                m += 1
            if m == len(segs):
                coda = tuple(segs[k:m])
                if (not coda and prof.final_vowel) or coda in finals:
                    return [segs[i:m]]
                return None
            for cut in range(k, m + 1):
                coda = tuple(segs[k:cut])
                if coda and coda not in codas:
                    continue
                rest = parse(cut, False)
                if rest is not None:
                    return [segs[i:cut]] + rest
            return None

        sy = parse(0, True)
        if sy is None:
            return None, f"breaks {prof.title}'s syllable rules"
        return ["".join(s) for s in sy], None

    def say(self, form, people, build=None, root=False):
        """Syllables with the stressed one in capitals: ES-ke-nai."""
        prof = self.profiles[people]
        words = form.lower().split(" ")
        parts_per_word = []
        if build:
            cur = []
            for p in list(build) + [" "]:
                if p == " ":
                    if cur:
                        parts_per_word.append(cur)
                    cur = []
                else:
                    cur.append(p)
        out = []
        for n, w in enumerate(words):
            sy, _ = self.syllables(w, people, root)
            if not sy:
                out.append(w)
                continue
            stress = 0
            if prof.stress == "last" and n < len(parts_per_word):
                parts = parts_per_word[n]
                roots = [i for i, p in enumerate(parts) if not p.startswith("-")]
                if roots:
                    last = roots[-1]
                    tongue = "dwarvish" if prof.source == "dwarvish" else "old"
                    stress = sum(sum(1 for s in (self.part(tongue, p) or []) if self.is_vowel(s))
                                 for p in parts[:last])
                    stress = min(stress, len(sy) - 1)
            out.append("-".join(s.upper() if i == stress else s for i, s in enumerate(sy)))
        return " ".join(out)

    def cyrillic(self, form, people):
        table = self.d["cyrillic"]
        multi = [k for k in table if len(k) > 1 and k != "e_initial"]
        out = []
        for w in form.split(" "):
            segs = tokenize(w, multi)
            cy = ""
            for i, s in enumerate(segs):
                if i == 0 and s == "e" and "e_initial" in table:
                    cy += table["e_initial"]
                else:
                    cy += table.get(s, s)
            out.append(cy[:1].upper() + cy[1:] if w[:1].isupper() else cy)
        return " ".join(out)

    # -- filters -------------------------------------------------------------------------

    def banned_hits(self, word, people=None, kind=None, on="form"):
        hits = []
        for b in self.d.get("banned", []):
            if b.get("on", "form") != on:
                continue
            if people and "people" in b and people not in b["people"]:
                continue
            if "people" in b and not people:
                continue
            if "kinds" in b and kind not in b["kinds"]:
                continue
            for w in (word.lower().split(" ") if on == "form" else [word.lower()]):
                if re.search(b["re"], w):
                    hits.append(b["why"])
                    break
        return hits

    def bad_hits(self, word):
        hits = []
        for lang, words in self.d.get("bad_words", {}).items():
            for w in word.lower().split(" "):
                for bad in words:
                    if (len(bad) <= 3 and w == bad) or (len(bad) > 3 and bad in w):
                        hits.append(f"{bad} ({lang})")
        return hits


# ---------------------------------------------------------------------------------------------
# gen
# ---------------------------------------------------------------------------------------------

def pick(rng, weights):
    keys = list(weights)
    return rng.choices(keys, [weights[k] for k in keys])[0]


def skeleton(word, lex):
    return tuple(s for s in tokenize(word, lex.sounds["rules"]) if not lex.is_vowel(s))


def too_close(cand, others, lex, min_distance):
    c = tokenize(cand, lex.sounds["rules"])
    for o in others:
        if cand == o:
            return o
        if len(cand) >= 4 and levenshtein(cand, o) < min_distance:
            return o
        if skeleton(cand, lex) == skeleton(o, lex) and skeleton(cand, lex):
            return o
    return None


def gen_word(rng, prof, lex):
    g = prof.gen
    multi = lex.multi(prof.key)
    sylls = int(pick(rng, {int(k): v for k, v in g.get("syllables", {"1": 1}).items()}))
    vowels = {k: v for k, v in prof.vowel_w.items() if k not in prof.loan_only}
    cons = {k: v for k, v in prof.cons_w.items() if k not in prof.loan_only}
    onsets = [c for c in prof.d.get("onsets", [])]
    initial = [c for c in prof.d.get("initial_onsets", [])] if prof.source != "dwarvish" else []
    codas = [c for c in prof.d.get("codas", []) if len(tokenize(c, multi)) == 1]
    finals = prof.d.get("final_codas", [])
    heavy = [c for c in finals if len(tokenize(c, multi)) > 1]
    single_finals = [c for c in finals if len(tokenize(c, multi)) == 1]
    out = ""
    for n in range(sylls):
        first, last = n == 0, n == sylls - 1
        if not first or rng.randrange(100) < g.get("onset", 85):
            if rng.randrange(100) < g.get("cluster", 10) and (onsets or (first and initial)):
                out += rng.choice(onsets + (initial if first else []))
            else:
                out += pick(rng, cons)
        out += pick(rng, vowels)
        if rng.randrange(100) < g.get("coda", 50):
            if last:
                if heavy and rng.randrange(100) < g.get("final_cluster", 5):
                    out += rng.choice(heavy)
                else:
                    out += pick(rng, {c: cons.get(c, 1) for c in single_finals})
            elif not last:
                out += pick(rng, {c: cons.get(c, 1) for c in codas})
    return out


def cmd_gen(lex, args):
    prof = lex.profiles.get(args.people)
    if not prof:
        sys.exit(f"unknown people {args.people}; one of: {', '.join(lex.profiles)}")
    seed = args.seed if args.seed is not None else random.randrange(10 ** 6)
    rng = random.Random(seed)
    gen_prof = prof if prof.source == "dwarvish" else lex.profiles["old"]
    pool = lex.dw_roots if prof.source == "dwarvish" else lex.roots
    existing = list(pool) + [n["form"].lower() for n in lex.names.values()]
    drift = lex.drift_for(prof)
    existing_forms = set()
    if drift:
        existing_forms = {lex.drifted([r], drift.key) for r in lex.roots}
    conf = lex.d.get("gen", {})
    min_d = conf.get("min_distance", 2)
    min_sounds = conf.get("min_sounds", 2)
    plain = {w for words in lex.d.get("plain_words", {}).values() for w in words}
    got, tries = [], 0
    while len(got) < args.n and tries < args.n * 400:
        tries += 1
        w = gen_word(rng, gen_prof, lex)
        sy, _ = lex.syllables(w, gen_prof.key)
        if not sy:
            continue
        segs = tokenize(w, lex.multi(gen_prof.key))
        if len(segs) < min_sounds or all(lex.is_vowel(x) for x in segs) or w in plain:
            continue
        if lex.banned_hits(w, gen_prof.key) or lex.bad_hits(w):
            continue
        if too_close(w, existing + [c for c, _ in got], lex, min_d):
            continue
        shown = w
        if drift:
            shown = spell(drift.apply(tokenize(w, lex.sounds["old"])))
            if not lex.syllables(shown, prof.key)[0]:
                continue
            if lex.banned_hits(shown, prof.key) or lex.bad_hits(shown):
                continue
            if too_close(shown, existing_forms, lex, min_d) or shown in plain:
                continue
        got.append((w, shown))
    print(f"# {args.n} candidate roots for {prof.title} (seed {seed}); pick, then add to the lexicon with a meaning")
    for w, shown in got:
        if drift:
            print(f"{w:<10} → {shown:<12} {lex.say(shown, prof.key)}")
        else:
            print(f"{w:<10} {lex.say(w, gen_prof.key)}")
    if len(got) < args.n:
        print(f"# only {len(got)} found", file=sys.stderr)


# ---------------------------------------------------------------------------------------------
# derive
# ---------------------------------------------------------------------------------------------

def show_build(build):
    """kvor, " ", vend, brok -> 'kvor · vend + brok' (· between words)."""
    words, cur = [], []
    for p in list(build) + [" "]:
        if p == " ":
            if cur:
                words.append(" + ".join(cur))
            cur = []
        else:
            cur.append(p)
    return " · ".join(words)


def cmd_derive(lex, args):
    word = args.word
    name = lex.names.get(word.lower())
    if name:
        build = name["build"]
        header = f"{name['form']}: {show_build(build)} ({name['people']}), \"{name.get('gloss', '')}\""
    elif "+" in word:
        build = [p.strip() for p in word.split("+")]
        header = " + ".join(build)
    else:
        build = [word.lower()]
        header = word.lower()
    print(header)
    if name and lex.profiles[name["people"]].source == "dwarvish":
        steps = []
        form, unknown = lex.derive(build, name["people"], steps=steps)
        if args.steps:
            for s in steps:
                print(f"  join  {s}")
        print(f"{'dwarvish':<9}{styled(form, name['form']):<14}{lex.say(form, 'dwarvish', build)}")
        if unknown:
            print(f"  unknown: {', '.join(unknown)}")
        return
    steps = []
    words, unknown = lex.build_words(build, "old", "eskenai", steps)
    if unknown and len(build) > 1:
        print(f"  unknown roots: {', '.join(unknown)}")
    if args.steps:
        for s in steps:
            print(f"  join  {s}")
    like = name["form"] if name else ""
    old = " ".join(spell(w) for w, _ in words)
    print(f"{'old':<9}{styled(old, like):<14}{lex.say(old, 'old'):<16}{lex.cyrillic(styled(old, like), 'old')}")
    drifts = [args.drift] if args.drift else list(lex.drifts)
    for key in drifts:
        drift = lex.drifts.get(key)
        if not drift:
            sys.exit(f"unknown drift {key}")
        trace = []
        out = " ".join(spell(drift.apply(w, trace)) for w, _ in words)
        people = next((p.key for p in lex.profiles.values() if p.drift == key), "old")
        note = "" if trace else "(no rule fired)"
        print(f"{key:<9}{styled(out, like):<14}{lex.say(out, people):<16}{note}")
        if args.steps:
            for n, rule, before, after in trace:
                print(f"  {n}  {rule.sca:<18} {before} → {after}   ({rule.says})")


# ---------------------------------------------------------------------------------------------
# check
# ---------------------------------------------------------------------------------------------

def drift_people(lex, dkey):
    """The people whose rules a drift's output must keep (a dialect drift: the people it extends)."""
    seen = set()
    while dkey and dkey not in seen:
        seen.add(dkey)
        for p in lex.profiles.values():
            if p.drift == dkey:
                return p.key
        dkey = lex.d["drift"].get(dkey, {}).get("extends")
    return None


def lang_values(lex):
    """{lang dir: {file: {key: value}}} for every lang dir the lexicon names, with the languages
    each one actually has (base has en and ru; the engine also fr)."""
    out = {}
    for d in lex.d.get("lang", {}).get("dirs", []):
        files = {}
        for p in sorted((REPO / d).glob("*.lang")):
            vals = {}
            for line in p.read_text(encoding="utf-8").splitlines():
                if "=" in line and not line.lstrip().startswith("#"):
                    k, v = line.split("=", 1)
                    vals[k.strip()] = v.strip()
            files[p.stem] = vals
        if files:
            out[d] = files
    return out


def lang_problems(lex, where, n, people, langs):
    """A name's lang keys: in every language of the mod that holds them, each showing the name."""
    lk = n.get("lang_key")
    if not lk:
        return []
    wants = [lk] + ([f"{lk}.title"] if n.get("epithet") else [])
    owners = [d for d, files in langs.items() if any(lk in vals for vals in files.values())]
    if not owners:
        parts = lk.split(".")
        guess = f"mods/{parts[1]}/assets/lang" if len(parts) > 2 else "its mod's assets/lang"
        return [f"{where}: lang key {lk} is in no lang file (put it in {guess}, every language there)"]
    bad = []
    for d in owners:
        for f, vals in langs[d].items():
            for k in wants:
                if k not in vals:
                    bad.append(f"{where}: lang key {k} missing from {d}/{f}.lang")
            if lk in vals:
                shown = lex.cyrillic(n["form"], people) if f.startswith("ru") else n["form"]
                if shown not in vals[lk]:
                    bad.append(f"{where}: {d}/{f}.lang {lk} = {vals[lk]!r} should show {shown}")
    return bad


def check(lex, sheet_path=SHEET):
    bad = list(lex.problems)

    for ex in lex.d.get("join_example", []):
        got = spell(lex.old_form(ex["build"])[0][0])
        if got != ex["form"]:
            bad.append(f"join {' + '.join(ex['build'])}: rules give {got}, the lexicon says {ex['form']}")
    for ex in lex.d.get("drift_example", []):
        old = spell(lex.old_form(ex["build"])[0][0])
        if old != ex["old"].lower():
            bad.append(f"drift example {ex['old']}: joins to {old}")
        for key in lex.drifts:
            if key in ex:
                got = lex.drifted(ex["build"], key)
                if got != ex[key].lower():
                    bad.append(f"drift example {ex['old']}: {key} gives {got}, the lexicon says {ex[key]}")

    for root, r in lex.roots.items():
        if not r.get("means"):
            bad.append(f"root {root}: no meaning")
        sy, why = lex.syllables(root, "old", root=True)
        if not sy:
            bad.append(f"root {root}: {why}")
        for h in lex.banned_hits(root, "old"):
            bad.append(f"root {root}: {h}")
        for h in lex.bad_hits(root):
            bad.append(f"root {root}: reads as {h}")
    for e, r in lex.endings.items():
        if not r.get("means"):
            bad.append(f"ending -{e}: no meaning")

    dw = lex.profiles.get("dwarvish")
    for root, r in lex.dw_roots.items():
        if "loan" in r:
            missing = [p for p in r["loan"] if lex.part("old", p) is None]
            if missing:
                bad.append(f"dwarvish loan {root}: unknown Old roots {', '.join(missing)}")
                continue
            got = lex.drifted(r["loan"], "frost")
            if got != root and not r.get("irregular"):
                bad.append(f"dwarvish loan {root}: {' + '.join(r['loan'])} through frost gives {got}")
        elif dw:
            segs = tokenize(root, lex.sounds["dwarvish"])
            loan_sounds = [s for s in segs if s in dw.loan_only]
            if loan_sounds:
                bad.append(f"dwarvish root {root}: {', '.join(loan_sounds)} only come in loans")
            sy, why = lex.syllables(root, "dwarvish")
            if not sy:
                bad.append(f"dwarvish root {root}: {why}")
            elif len(sy) != 1:
                bad.append(f"dwarvish root {root}: native roots have one syllable")
        if not r.get("means"):
            bad.append(f"dwarvish root {root}: no meaning")
        for h in lex.bad_hits(root):
            bad.append(f"dwarvish root {root}: reads as {h}")

    # Every drift's output keeps its people's syllable rules: a sound change that leaves an illegal
    # cluster (worn once turned dawaru into dawro) shows up here, not in a name months later.
    builds = {(r,): f"root {r}" for r in lex.roots}
    for x in lex.d.get("join_example", []) + lex.d.get("drift_example", []):
        builds.setdefault(tuple(x["build"]), f"example {' + '.join(x['build'])}")
    for key, n in lex.names.items():
        if n.get("people") in lex.profiles and lex.profiles[n["people"]].source != "dwarvish":
            builds.setdefault(tuple(n.get("build", [])), f"name {key}")
    for dkey in lex.drifts:
        people = drift_people(lex, dkey)
        if not people:
            continue
        for build, where in builds.items():
            if lex.old_form(list(build))[1]:
                continue  # unknown roots are reported with the name
            for w in lex.drifted(list(build), dkey).split(" "):
                sy, why = lex.syllables(w, people, root=where.startswith("root "))
                if not sy:
                    bad.append(f"drift {dkey} turns {where} into {w}, which {why}")

    langs = lang_values(lex)
    for key, n in lex.names.items():
        where = f"name {key}"
        people = n.get("people")
        if people not in lex.profiles:
            bad.append(f"{where}: unknown people {people}")
            continue
        if n.get("kind") not in lex.kinds:
            bad.append(f"{where}: unknown kind {n.get('kind')}")
        for field in ("build", "form", "gloss"):
            if not n.get(field):
                bad.append(f"{where}: no {field}")
        if not n.get("build") or not n.get("form"):
            continue
        home = n.get("home")
        if home and home not in lex.profiles[people].d.get("homes", {}):
            bad.append(f"{where}: unknown home {home}")
        got, unknown = lex.derive(n["build"], people, home)
        if unknown:
            bad.append(f"{where}: unknown roots {', '.join(unknown)}")
        elif got != n["form"].lower() and not n.get("irregular"):
            bad.append(f"{where}: {' + '.join(n['build'])} in {people} gives {got}, not {n['form']} "
                       f"(fix the build, or say why with irregular = \"...\")")
        for w in n["form"].split(" "):
            sy, why = lex.syllables(w, people)
            if not sy:
                bad.append(f"{where}: {w} {why}")
        for h in lex.banned_hits(n["form"], people, n.get("kind")):
            bad.append(f"{where}: {h}")
        for h in lex.bad_hits(n["form"]):
            bad.append(f"{where}: reads as {h}")
        for field in ("byname", "epithet", "english"):
            if n.get(field):
                for h in lex.banned_hits(n[field], on="english"):
                    bad.append(f"{where}: {field} {n[field]!r}: {h}")
        if (lex.profiles[people].source == "dwarvish" and n.get("kind") == "person"
                and not n.get("byname")):
            bad.append(f"{where}: a dwarf needs a byname (people.dwarvish.naming)")
        bad += lang_problems(lex, where, n, people, langs)

    by_kind = {}
    for key, n in lex.names.items():
        if n.get("form"):
            by_kind.setdefault(n.get("kind"), []).append((key, n))
    for kind, items in by_kind.items():
        for i, (ka, a) in enumerate(items):
            for kb, b in items[i + 1:]:
                if lex.etymon(a) == lex.etymon(b):
                    continue  # one word in two tongues
                if levenshtein(a["form"].lower(), b["form"].lower()) <= 1:
                    bad.append(f"names {a['form']} and {b['form']} ({kind}) are one edit apart")

    if sheet_path is not None:
        now = sheet_path.read_text(encoding="utf-8") if sheet_path.exists() else ""
        if now != sheet(lex):
            shown = sheet_path.relative_to(REPO) if sheet_path.is_relative_to(REPO) else sheet_path
            bad.append(f"{shown} is out of date: {SHEET_CMD}")
    return bad


FAULTS = Path(__file__).resolve().parent / "faults.toml"
WORN_RULE_2 = re.compile(r'  \{ sca = "P//#\(C\)\(C\)VK_CV".*\n  \{ sca = "P//#\(C\)\(C\)VB_rV".*\n')
# What `check` must say about the fault set, one line each.
EXPECTED = [
    "name f_unknown: unknown roots zor",
    "name f_unknown: Eskezor 'z' isn't a sound of Old Eskenai",
    "name f_unknown: x, z, q and j used to look exotic",
    "name f_mismatch: bor + duk in elvish gives borduga, not Borduk",
    "name f_mismatch: Borduk breaks Elvish's syllable rules",
    "name f_wen: reads as Tolkien or generic elf",
    "name f_el: El- at the start of an elf's personal name",
    "names Nogom and Nogom (boss) are one edit apart",
    "name f_stormforge: epithet 'the Stormforge': grand English mash-ups",
    "names Aru and Kru (rune) are one edit apart",
    "name f_half: lang key rune.test.half missing from LANG/ru_ru.lang",
    "name f_wrong: LANG/ru_ru.lang rune.test.wrong = 'Kru' should show Кру",
    "name f_nowhere: lang key word.base.ber is in no lang file (put it in mods/base/assets/lang",
    "name f_dwarf: a dwarf needs a byname",
    "name f_letter: Telm 'l' isn't a sound of Old Eskenai",
    "name f_rude: reads as kaka (ru)",
    "dwarvish root dwir: i only come in loans",
    "dwarvish root dwir: 'w' isn't a sound of Dwarvish",
    "dwarvish loan kan: kaun through frost gives kon",
    "drift f_broken: 'a/o': want target/replacement/environment[/exception]",
    "drift f_broken: 'p,t/b/_Q': unknown category Q",
    "drift f_broken: 'p,t,k/b,d/_': 3 targets but 2 replacements",
    "drift worn turns example dau + aru into dawro, which breaks Worn speech",
    "NAMES.md is out of date",
]
# Lines `check` must not say: f_stormforge's key is in both of its folder's languages (en, ru),
# so no French is asked for.
UNEXPECTED = ["rune.test.kru missing", "fr_fr"]


def cmd_selftest(lex, args):
    """The real lexicon plus tools/names/faults.toml must fail `check` in exactly the known ways."""
    import tempfile
    with tempfile.TemporaryDirectory() as tmp:
        tmp = Path(tmp)
        lang = tmp / "lang"
        lang.mkdir()
        (lang / "en_us.lang").write_text(
            "rune.test.kru=Kru\nrune.test.kru.title=the Stormforge\nrune.test.half=Tun\nrune.test.wrong=Kru\n",
            encoding="utf-8")
        (lang / "ru_ru.lang").write_text(
            "rune.test.kru=Кру\nrune.test.kru.title=Кузня бурь\nrune.test.wrong=Kru\n", encoding="utf-8")
        text = lex.path.read_text(encoding="utf-8")
        text, swapped = WORN_RULE_2.subn('  { sca = "P//#(C)(C)VC_CV", says = "rule 2 unsplit" },\n', text)
        if swapped != 1:
            sys.exit("selftest: couldn't find worn rule 2 to swap back")
        text = text.replace("[lang]\ndirs = [", f'[lang]\ndirs = ["{lang}", ', 1)
        broken = tmp / "lexicon.toml"
        broken.write_text(text + "\n" + FAULTS.read_text(encoding="utf-8"), encoding="utf-8")
        stale = tmp / "NAMES.md"
        stale.write_text("stale\n", encoding="utf-8")
        got = [b.replace(str(lang), "LANG") for b in check(Lexicon(broken), stale)]
    missed = [e for e in EXPECTED if not any(e in g for g in got)]
    unwanted = [g for g in got if any(u in g for u in UNEXPECTED)]
    extra = [g for g in got if not any(e in g for e in EXPECTED)]
    for m in missed:
        print(f"✗ check missed: {m}")
    for u in unwanted + [x for x in extra if x not in unwanted]:
        print(f"✗ check said: {u}")
    if missed or unwanted or extra:
        sys.exit(1)
    print(f"ok: check caught all {len(EXPECTED)} faults in {FAULTS.relative_to(REPO)}")


def cmd_check(lex, args):
    bad = check(lex)
    for b in bad:
        print(f"✗ {b}")
    if bad:
        print(f"{len(bad)} problems in {lex.path}")
        sys.exit(1)
    print(f"ok: {len(lex.roots)} roots, {len(lex.dw_roots)} dwarvish roots, {len(lex.names)} names, "
          f"{len(lex.drifts)} drifts")


# ---------------------------------------------------------------------------------------------
# sheet
# ---------------------------------------------------------------------------------------------

def table(rows, head):
    out = ["| " + " | ".join(head) + " |", "|" + "---|" * len(head)]
    out += ["| " + " | ".join(str(c).replace("|", "\\|") for c in r) + " |" for r in rows]
    return "\n".join(out)


def sheet(lex):
    d = lex.d
    L = []
    w = L.append
    w("# Names: the lexicon's sheet")
    w("")
    w(f"Written from `mods/base/lore/lexicon.toml` (`{SHEET_CMD}`); do not edit by hand. "
      "`python3 tools/names/names.py check` fails while it is stale. Why it is built this way: "
      "`reports/Naming and invented words.md`. New names come from `gen` and `derive`, never freehand.")
    w("")
    w("## Saying them")
    w("")
    w("- One spelling per sound: *k* and *g* always hard, *s* never *z*, *h* always said, *ch* as in *church*.")
    w("- Vowels as in Spanish or Italian: *a* in *father*, *e* in *bed*, *i* in *machine*, *o* in *more*, "
      "*u* in *rule*; *ai* in *eye*, *au* in *cow*, *ei* in *day*.")
    w("- Stress (capitals below): Old Eskenai and its daughters on the first syllable, Dwarvish on the last root.")
    w("- Invented names are never translated. Russian writes them by the table at the end; French and English as written.")
    w("")

    w("## The tongues")
    w("")
    for p in lex.profiles.values():
        how = f"from {p.source}" + (f" through the {p.drift} drift" if p.drift else "")
        w(f"- **{p.title}** (`{p.key}`, {how}; stress on the {p.stress} "
          f"{'root' if p.stress == 'last' else 'syllable'}). {p.d.get('about', '')}")
    w("")
    regions = {}
    for k, v in d.get("regions", {}).items():
        regions.setdefault(v, []).append(k)
    w("Which tongue names a ruin, by where it stands: "
      + "; ".join(f"{', '.join(ks)}: {lex.profiles[v].title}" for v, ks in regions.items()) + ".")
    w("")

    w("## Old Eskenai's roots")
    w("")
    rows = []
    for r, v in lex.roots.items():
        rows.append([f"**{r}**", v["means"]] + [lex.drifted([r], k) for k in lex.drifts]
                    + [lex.say(r, "old", root=True), lex.cyrillic(r, "old")])
    w(table(rows, ["root", "means"] + [lex.drifts[k].title for k in lex.drifts] + ["say", "ru"]))
    w("")
    w("Endings: " + "; ".join(f"**-{e}** {v['means']} (*{v.get('example', '')}*)" for e, v in lex.endings.items()) + ".")
    w("")

    w("## How roots join")
    w("")
    w("The modifier comes first and the head last. If three consonants would meet, or two of the same, "
      "*e* goes between. If two vowels meet, the first plain vowel drops; a diphthong's second half becomes *w* or *y*.")
    w("")
    w(table([[" + ".join(x["build"]), x["form"]] for x in d.get("join_example", [])], ["build", "joined"]))
    w("")

    w("## The drifts")
    w("")
    w("Each applied in order, in SCA² syntax (`target/replacement/environment[/exception]`: `_` the sound "
      "changed, `#` a word's edge, capitals categories, `(X)` optional).")
    w("")
    w("Categories: " + "; ".join(f"`{k}` = {' '.join(v)}" for k, v in lex.cats.items()) + ".")
    w("")
    for key, dr in lex.drifts.items():
        ext = d["drift"][key].get("extends")
        w(f"**{dr.title}** ({dr.used_for}{'; extends ' + ext if ext else ''})")
        w("")
        for n, r in enumerate(dr.rules, 1):
            w(f"{n}. {r.says} `{r.sca}`")
        w("")
    w("One word in every form:")
    w("")
    rows = [[x["old"], x["means"]] + [x.get(k, "") for k in lex.drifts] for x in d.get("drift_example", [])]
    w(table(rows, ["Old Eskenai", "means"] + [lex.drifts[k].title for k in lex.drifts]))
    w("")

    w("## The names")
    w("")
    w("Every name with its build, meaning and how to say it. For names from Old roots, the same word in every drift.")
    w("")
    order = {k: i for i, k in enumerate(lex.kinds)}
    names = sorted(lex.names.items(), key=lambda kv: (order.get(kv[1].get("kind"), 99), kv[1]["form"].lower()))
    for kind in lex.kinds:
        items = [(k, n) for k, n in names if n.get("kind") == kind]
        if not items:
            continue
        w(f"### {kind}")
        w("")
        rows = []
        for key, n in items:
            prof = lex.profiles[n["people"]]
            shown = n["form"]
            extra = []
            if n.get("epithet"):
                shown += f", {n['epithet']}"
            if n.get("english"):
                extra.append(f"English: {n['english']}")
            if n.get("byname"):
                extra.append(f"byname: {n['byname']}")
            if n.get("home"):
                extra.append(f"home: {n['home']}")
            if n.get("irregular"):
                extra.append(f"irregular: {n['irregular']}")
            if prof.source == "dwarvish":
                forms = "—"
            else:
                forms = " · ".join(styled(lex.drifted(n["build"], k), n["form"]) for k in lex.drifts)
            rows.append([f"**{shown}**", n.get("what", ""), show_build(n["build"]), n.get("gloss", ""),
                         prof.title, lex.say(n["form"], n["people"], n["build"]),
                         lex.cyrillic(n["form"], n["people"]), forms, "; ".join(extra)])
        w(table(rows, ["name", "what", "build", "means", "tongue", "say", "ru",
                       " · ".join(dr.title for dr in lex.drifts.values()), "notes"]))
        w("")

    w("## The elves' homes")
    w("")
    homes = lex.profiles["elvish"].d.get("homes", {}) if "elvish" in lex.profiles else {}
    for k, h in homes.items():
        realm = lex.names.get(h.get("realm", ""), {}).get("form", k)
        w(f"- **{realm}** (`{k}`, dialect: {h.get('drift', 'soft')}): {h.get('where', '')}")
    w("")
    w("More homes and regional dialects go in `[people.elvish.homes]`; a dialect is a drift that extends `soft`.")
    w("")

    w("## Dwarvish")
    w("")
    rows = [[f"**{r}**", v["means"], "native" if "loan" not in v else f"loan of {' + '.join(v['loan'])} (frost)",
             lex.cyrillic(r, "dwarvish")] for r, v in lex.dw_roots.items()]
    w(table(rows, ["root", "means", "from", "ru"]))
    w("")
    w("Endings: " + "; ".join(f"**-{e}** {v['means']} (*{v.get('example', '')}*)" for e, v in lex.dw_endings.items()) + ".")
    w("")
    nm = lex.profiles["dwarvish"].d.get("naming", {}) if "dwarvish" in lex.profiles else {}
    if nm:
        w(f"Bynames and true names: players mostly meet a dwarf's plain English byname "
          f"({', '.join(nm.get('byname_examples', []))}); the true name, in Dwarvish, is {nm.get('true_name', '')}.")
        w("")

    w("## Against the generic")
    w("")
    for b in d.get("banned", []):
        scope = []
        if b.get("people"):
            scope.append(", ".join(b["people"]))
        if b.get("kinds"):
            scope.append(", ".join(b["kinds"]))
        if b.get("on") == "english":
            scope.append("English names")
        w(f"- {b['why']} `{b['re']}`" + (f" ({'; '.join(scope)})" if scope else ""))
    w("")
    w("Bad-word lists (funny, rude or loaded in a language players read): "
      + ", ".join(f"{k} {len(v)}" for k, v in d.get("bad_words", {}).items()) + " words.")
    w("")

    w("## Russian letters")
    w("")
    w(" · ".join(f"{k.replace('e_initial', 'e (at the start)')} {v}" for k, v in d.get("cyrillic", {}).items()))
    w("")
    return "\n".join(L)


def cmd_sheet(lex, args):
    sys.stdout.write(sheet(lex))


# ---------------------------------------------------------------------------------------------

def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--lexicon", default=str(LEXICON), help=argparse.SUPPRESS)
    sub = ap.add_subparsers(dest="cmd", required=True)
    g = sub.add_parser("gen", help="candidate roots from a people's sounds")
    g.add_argument("--people", required=True)
    g.add_argument("-n", type=int, default=20)
    g.add_argument("--seed", type=int)
    dv = sub.add_parser("derive", help="a word or build through every drift")
    dv.add_argument("word", help="a name's key, a build (esk+tem) or an Old word")
    dv.add_argument("--steps", action="store_true", help="show every rule that fired")
    dv.add_argument("--drift", help="only this drift")
    sub.add_parser("check", help="fail on any break in the lexicon or a stale NAMES.md")
    sub.add_parser("sheet", help="print notes/NAMES.md")
    sub.add_parser("selftest", help="check must catch every fault in tools/names/faults.toml")
    args = ap.parse_args(argv)
    lex = Lexicon(args.lexicon)
    {"gen": cmd_gen, "derive": cmd_derive, "check": cmd_check, "sheet": cmd_sheet,
     "selftest": cmd_selftest}[args.cmd](lex, args)


if __name__ == "__main__":
    main()
