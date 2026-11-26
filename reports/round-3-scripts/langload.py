import re, os
def load(p):
    keys = {}
    if not os.path.exists(p):
        return keys
    lines = open(p, encoding="utf-8").read().split("\n")
    last = None
    for l in lines:
        if not l.strip() or l.lstrip().startswith("#"):
            continue
        if l.startswith((" ", "\t")) and last:
            continue
        if "=" in l:
            k, v = l.split("=", 1)
            last = k.strip()
            keys[last] = v.strip()
    return keys

L = {
    "eng_en": load(f"{ROOT}/crates/engine/assets/lang/en_us.lang"),
    "eng_ru": load(f"{ROOT}/crates/engine/assets/lang/ru_ru.lang"),
    "eng_fr": load(f"{ROOT}/crates/engine/assets/lang/fr_fr.lang"),
    "base_en": load(f"{ROOT}/mods/base/assets/lang/en_us.lang"),
    "base_ru": load(f"{ROOT}/mods/base/assets/lang/ru_ru.lang"),
    "an_en": load(f"{ROOT}/mods/animals/assets/lang/en_us.lang"),
    "po_en": load(f"{ROOT}/mods/portals/assets/lang/en_us.lang"),
}
