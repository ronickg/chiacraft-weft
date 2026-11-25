"""Fetch Freesound HQ previews of CC0 sounds into the archive, refusing any that isn't CC0.
Usage: python3 fetch_freesound.py <user>/<id> [...]     (e.g. Villehardt/613995)
Each lands in ~/Coding/chiacraft-archive/references/sound/freesound/<id>_<user>_<title>.mp3, and
one line per sound is printed: id, user, licence, seconds, title, file (for SOURCES.md).
Freesound answers 429 to parallel fetches: this goes one at a time, with a pause.
"""
import os, re, sys, time, urllib.request

OUT = os.path.expanduser("~/Coding/chiacraft-archive/references/sound/freesound")
UA = {"User-Agent": "Mozilla/5.0 (sound fetch for a CC0 game asset list)"}


def get(url):
    with urllib.request.urlopen(urllib.request.Request(url, headers=UA), timeout=60) as r:
        return r.read()


def fetch(spec):
    user, sid = spec.split("/")
    page = get(f"https://freesound.org/people/{user}/sounds/{sid}/").decode("utf-8", "replace")
    # The sound's own licence: the first licence link on its page.
    lic = re.search(r"creativecommons\.org/(publicdomain/zero/1\.0|licenses/[a-z-]+/[0-9.]+)", page)
    lic = lic.group(1) if lic else "?"
    title = re.search(r"<title>Freesound - (.*?) by ", page, re.S)
    title = title.group(1).strip() if title else sid
    dur = re.search(r'duration="([0-9.]+)"', page)
    prev = re.search(rf"https://cdn\.freesound\.org/previews/\d+/{sid}_\d+-hq\.mp3", page)
    if lic != "publicdomain/zero/1.0":
        print(f"REFUSED {sid}\t{user}\t{lic}\t{title}")
        return
    if not prev:
        print(f"NO PREVIEW {sid}\t{user}\t{title}")
        return
    slug = re.sub(r"[^a-z0-9]+", "_", os.path.splitext(title)[0].lower()).strip("_")[:48]
    path = os.path.join(OUT, f"{sid}_{user}_{slug}.mp3")
    if not os.path.exists(path):
        data = get(prev.group(0))
        with open(path, "wb") as f:
            f.write(data)
    print(f"{sid}\t{user}\tCC0\t{float(dur.group(1)) if dur else 0:.0f}s\t{title}\t{path}")


if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)
    for i, spec in enumerate(sys.argv[1:]):
        if i:
            time.sleep(2)
        try:
            fetch(spec)
        except Exception as e:  # one bad page doesn't stop the rest
            print(f"FAILED {spec}: {e}")
