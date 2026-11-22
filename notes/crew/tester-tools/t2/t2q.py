#!/usr/bin/env python3
"""Mac-side front end for the PC job runner (Tester 2). Rebuilt 11-22 from the runner's job format after the archive was wiped.
  t2q.py build <commit> <TREE> [base] [--features F] [--env K=V]   stage the commit on the PC and queue its build
  (TREE may be @<commit|branch>: builds that commit once as tree C_<short> and runs on it)
  t2q.py look <id> <TREE> <script> <world> [--needs TREE] [--view N] [--limit S] [--scale N] [--prio N] [--sample] [--mobs] [--weather-on]
                                           [--fresh] [--nofreeze] [--noworld] [--account shared] [--preset max] [-- extra weft args]
  t2q.py bench <id> <TREE> <script> [--size WxH] [--limit S] [--needs TREE] [--prio N] [--view N] [--preset max] [--fresh] [--low] [-- extra weft args]
  t2q.py multi <id> <TREE|@ref> <world> [--limit S] [--prefix P] [--size 960x540] [--needs TREE] name:script name:script ...   N copies in one world (host first), counts as N games
  t2q.py status                                 pending/running/held counts and names, recent done lines, a HOLD, disk free, the runner process
  t2q.py hold <minutes> "<reason>" [exclusive]  no new benches (soft) or looks, multis and benches (exclusive) start for that long; builds go on; clears itself
  t2q.py hold 0                                 clears a hold early
  t2q.py fetch <id> [date]                      copy the PC results ~/looktest/t2_<id>/ into ~/Coding/chiacraft-archive/images/tester-2/<date>/runs/t2_<id>/
  t2q.py cancel <id>                            move a pending job to cancelled/
Job ids are unique: the script refuses an id that is already pending, running, held or done (a result folder t2_<id> would be overwritten)."""
import json, subprocess, sys, os, time, re, shlex
PC = "ronaldg@192.168.1.13"
REPO = os.path.expanduser("~/Coding/chiacraft-weft")
ARCHIVE = os.path.expanduser("~/Coding/chiacraft-archive/images/tester-2")
Q = "~/looktest/queue"
FIX = '3ae9c6e1'   # run-config fix in dev: /rule takes 0/1/true/false there
MAX_CONFIG = ("core:render_scale=1,core:shadows=2,core:shadow_size=4096,base:fx_reflect=2,base:fx_contact=2,base:fx_volume=2,base:fx_shafts=2,"
              "core:lod_distance=512,core:hdr_world=1,core:aa=1,core:froxels=2,core:cloud_shadows=1,core:bloom=2,core:shadow_far=0,core:post_low=0,"
              "base:look_leaf_holes=1,core:rt_shadows=1,base:fx_wind_life=3,core:soft_clouds=1")   # the Max preset (the benchmark default); a second --config replaces the first


def ssh(cmd, inp=None):
    return subprocess.run(["ssh", "-o", "BatchMode=yes", PC, "bash -c " + shlex.quote(cmd)], input=inp, text=True, capture_output=True)   # bash: the PC's login shell is zsh, which aborts a command whose glob matches nothing


def has_fix(ref):
    try:
        return subprocess.run(["git", "-C", REPO, "merge-base", "--is-ancestor", FIX, ref], capture_output=True).returncode == 0
    except Exception:
        return False


def check_script(path, newer=False):
    """Gate for new scripts: before dev 3ae9c6e1 `/rule` took only a whole number (`false` left mobs ON); a `shot` right before a `bench` stalls a frame into it."""
    L = open(path).read().split("\n")
    bad = [i + 1 for i, l in enumerate(L) if re.search(r"rule\s+core:mob_spawning\s+(false|true)\b", l)]
    if bad and not newer:
        sys.exit(f"REJECTED {path}: line(s) {bad}: this build's `/rule` takes a whole number: write `/rule core:mob_spawning 0` (off) or 1 (on).")
    for i, l in enumerate(L[:-1]):
        if l.startswith("shot ") and L[i + 1].startswith("bench"):
            sys.exit(f"REJECTED {path}: line {i + 1}: a `shot` directly before a `bench` stalls a frame into the bench. Put `wait 1` between them.")


def put_script(path, newer=False, sub=""):
    """Upload a script as ~/looktest/<sub>t2q_<name> (multi copies live in ~/looktest/pt/, where ptmulti.sh reads them)."""
    check_script(path, newer)
    name = "t2q_" + os.path.basename(path)
    r = ssh(f"cat > ~/looktest/{sub}{name}", open(path).read())
    if r.returncode:
        sys.exit("script upload failed: " + r.stderr)
    return f"/home/ronaldg/looktest/{sub}{name}"


def seed_packs(job):
    """The runner fails a look whose --config names resource packs the seed dir lacks: copy each named pack (from any seed/*/resourcepacks/<name>) into
    ~/looktest/seed/<id>/resourcepacks/ (a reflink copy; shader packs are seeded by the runner)."""
    cfg = " ".join(str(e) for e in job.get("extra") or [])
    names = []
    for m in re.finditer(r"core:resource_packs=([A-Za-z0-9_,.-]+)", cfg):
        names += [n for n in m.group(1).split(",") if n and n != "core"]   # the list ends where the next key (`core:...`) starts
    jid = job["id"]
    for n in dict.fromkeys(names):
        r = ssh(f"d=~/looktest/seed/{jid}/resourcepacks; mkdir -p $d; [ -e $d/{n} ] || {{ src=$(ls -d ~/looktest/seed/*/resourcepacks/{n} 2>/dev/null | head -1); [ -n \"$src\" ] && cp -a --reflink=auto $src $d/ && echo seeded {n} || echo NO SOURCE FOR PACK {n}; }}")
        print((r.stdout + r.stderr).strip())


def drop(job):
    """Write the job file into pending/ (tmp name first, then a move, so the runner never reads half a file)."""
    jid = job["id"]
    if job["kind"] in ("look", "bench"):
        seed_packs(job)
    r = ssh(f"cd {Q}; ls pending/{jid}.json running/{jid}.json held/{jid}.json done/{jid}.done 2>/dev/null | head -1")
    if r.stdout.strip():
        sys.exit(f"id {jid} already exists ({r.stdout.strip()}): pick a new id")
    r = ssh(f"cd {Q}; cat > pending/.{jid}.tmp && mv pending/.{jid}.tmp pending/{jid}.json", json.dumps(job))
    if r.returncode:
        sys.exit("queueing failed: " + r.stderr)
    print("queued", jid)


def full_sha(ref):
    return subprocess.check_output(["git", "-C", REPO, "rev-parse", ref], text=True).strip()


def stage(full):
    p1 = subprocess.Popen(["git", "-C", REPO, "archive", "--format=tar", full], stdout=subprocess.PIPE)
    r = subprocess.run(["ssh", "-o", "BatchMode=yes", PC, f"rm -rf ~/stage_{full}; mkdir ~/stage_{full} && tar -x -C ~/stage_{full}"], stdin=p1.stdout)
    p1.wait()
    if r.returncode:
        sys.exit("staging failed")


def build_job(name, commit, base="UF_before", features=None, env=None, prio=1):
    j = {"kind": "build", "id": f"build_{name}_{commit}", "name": name, "commit": commit, "base": base, "prio": prio}
    if features:
        j["features"] = features
    if env:
        j["env"] = env
    return j


def resolve_tree(tree):
    """`@ref` -> (commit, 'C_<short>') building the commit once (staged if no build of that tree is pending, running or finished); other names pass through."""
    if not tree.startswith("@"):
        return tree
    full = full_sha(tree[1:])
    short = full[:8]
    name = f"C_{short}"
    r = ssh(f"grep -l Finished ~/build_{name}.log 2>/dev/null; ls {Q}/pending/build_{name}_* {Q}/running/build_{name}_* 2>/dev/null")
    if not r.stdout.strip():
        stage(full)
        drop(build_job(name, full))
        # the runner reads the commit as given; staged dir is ~/stage_<commit>, so job commit == the full hash
    return name


a = sys.argv[1:]
if not a or a[0] in ("-h", "--help"):
    print(__doc__)
    sys.exit(0)
cmd = a[0]
extra = []
if "--" in a:
    k = a.index("--")
    extra = a[k + 1:]
    a = a[:k]
FLAGS = {"--sample", "--mobs", "--weather-on", "--fresh", "--nofreeze", "--noworld", "--low"}


def opt(name, default=None):
    if name in a:
        i = a.index(name)
        v = a[i + 1]
        del a[i:i + 2]
        return v
    return default


def flag(name):
    if name in a:
        a.remove(name)
        return True
    return False


if cmd == "build":
    features = opt("--features")
    envs = {}
    while "--env" in a:
        k, _, v = opt("--env").partition("=")
        envs[k] = v
    commit, name = a[1], a[2]
    base = a[3] if len(a) > 3 else "UF_before"
    if name.startswith("@"):
        name = resolve_tree(name)
    else:
        full = full_sha(commit)
        stage(full)
        drop(build_job(name, full, base, features, envs or None))
elif cmd in ("look", "bench"):
    needs = opt("--needs"); size = opt("--size", "1280x720"); limit = int(opt("--limit", "190" if cmd == "look" else "150"))
    scale = int(opt("--scale", "1")); view = int(opt("--view", "12")); prio = int(opt("--prio", "5")); preset = opt("--preset"); account = opt("--account")
    sample = flag("--sample"); mobs = flag("--mobs"); wthr = flag("--weather-on"); fresh = flag("--fresh"); nofreeze = flag("--nofreeze"); noworld = flag("--noworld"); low = flag("--low")
    jid, tree, script_path = a[1], a[2], a[3]
    ref = tree[1:] if tree.startswith("@") else "dev"
    script = put_script(script_path, has_fix(ref))
    tree = resolve_tree(tree)
    j = {"kind": cmd, "id": jid, "tree": tree, "script": script, "extra": list(extra), "needs": needs or tree, "limit": limit, "scale": scale, "view": view, "prio": prio}
    if preset == "max":
        j["preset"] = "max"
        if "--config" not in j["extra"]:
            j["extra"] += ["--config", MAX_CONFIG]
    if cmd == "look":
        if len(a) > 4:
            j["world"] = a[4]
        if wthr:
            j["weather"] = "on"
    else:
        w, h = size.split("x")
        j["w"], j["h"] = int(w), int(h)
    for k, v in (("sample", sample), ("mobs", mobs), ("fresh", fresh), ("nofreeze", nofreeze), ("noworld", noworld), ("low", low)):
        if v:
            j[k] = True
    if account:
        j["account"] = account
    drop(j)
elif cmd == "multi":
    needs = opt("--needs"); limit = int(opt("--limit", "480")); prefix = opt("--prefix"); size = opt("--size", "640x360"); prio = int(opt("--prio", "5"))
    jid, tree, world = a[1], a[2], a[3]
    copies = []
    for spec in a[4:]:
        n, _, s = spec.partition(":")
        copies.append({"name": n, "script": os.path.basename(put_script(s, True, "pt/"))})
    tree = resolve_tree(tree)
    sw, sh = size.split("x")
    drop({"kind": "multi", "id": jid, "tree": tree, "world": world, "limit": limit, "copies": copies, "needs": needs or tree, "sw": int(sw), "sh": int(sh), "prefix": prefix or jid, "prio": prio})
elif cmd == "status":
    r = ssh(f"""cd {Q}; echo "runner: $(pgrep -f '[r]unner.py' | head -1 || echo NOT RUNNING)  disk free: $(df -h /home | tail -1 | awk '{{print $4}}')"; [ -f HOLD ] && echo "HOLD: $(tail -n +2 HOLD | tr '\\n' ' ') until $(date -d @$(head -1 HOLD) +%T)";
for d in running pending held; do echo "== $d: $(ls $d | wc -l)"; ls -t $d | head -25 | sed 's/.json$//' | tr '\\n' ' '; echo; done; echo "== done (last 15)"; ls -t done/*.done | head -15 | xargs -n1 cat; tail -3 runner.log""")
    print(r.stdout + r.stderr)
elif cmd == "hold":
    mins = float(a[1])
    if mins == 0:
        ssh(f"rm -f {Q}/HOLD"); print("hold cleared")
    else:
        reason = a[2] if len(a) > 2 else "hold"
        excl = "\nexclusive" if len(a) > 3 and a[3] == "exclusive" else ""
        ssh(f"cat > {Q}/HOLD", f"{int(time.time() + mins * 60)}\n{reason}{excl}\n")
        print(f"hold for {mins:g} min: {reason}")
elif cmd == "fetch":
    jid = a[1]
    date = a[2] if len(a) > 2 else time.strftime("%Y-%m-%d")
    dest = f"{ARCHIVE}/{date}/runs/t2_{jid}/"
    os.makedirs(dest, exist_ok=True)
    sys.exit(subprocess.run(["rsync", "-a", "-e", "ssh -o BatchMode=yes", f"{PC}:looktest/t2_{jid}/", dest]).returncode)
elif cmd == "cancel":
    jid = a[1]
    print(ssh(f"cd {Q}; mkdir -p cancelled; mv pending/{jid}.json cancelled/ && echo cancelled {jid}").stdout)
else:
    print(__doc__)
    sys.exit(1)
