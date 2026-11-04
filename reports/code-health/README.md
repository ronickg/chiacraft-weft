# Code health check

`health.py` reports, in under two seconds and with nothing but Python's standard library:
- lines per crate and mod;
- growth since the fork, split into Weft's merges and ours;
- the biggest files and functions;
- duplicate blocks and functions written more than once;
- dependencies a crate never names;
- debug leftovers;
- big checked-in files that aren't code.

It only reads. Run it from the root of a checkout:

```bash
python3 -I reports/code-health/health.py > health.md
```

The first report, for `dev` at `30de857`, is `2026-11-04 dev 30de857.md`: compare the next run with it. Round 5 (`reports/Research round 5.md`) reads it.

## In CI

A job that runs on pushes to `dev` and the trunk and puts the report in the run's summary. It
fails nothing yet. Add it to `.github/workflows/ci.yml`:

```yaml
  code-health:
    runs-on: ubuntu-latest
    timeout-minutes: 5
    steps:
      - uses: actions/checkout@v7
        with:
          fetch-depth: 0 # the growth table needs the fork point and upstream's history
      - name: Upstream refs
        run: git fetch --no-tags https://github.com/VortexSo4/Weft master:refs/remotes/upstream/master || true
      - name: Report
        run: |
          python3 -I reports/code-health/health.py --json health.json > health.md
          cat health.md >> "$GITHUB_STEP_SUMMARY"
      - uses: actions/upload-artifact@v7
        with:
          name: code-health
          path: |
            health.md
            health.json
```

**Gating, once the round 5 fixes land.** Add `--max-dup-blocks N --max-big-fns M` to the Report
step, with N and M set at the numbers of the day plus a little. A pull request that adds a new
copy-pasted block or a 150-line function then fails with the reason, and nothing else does.

**Optional, for unused dependencies with more certainty:** `cargo machete`, which needs no build.
It can run in the same job after `cargo install cargo-machete --locked`, and catches what the
script's name search misses (crates used only through a re-export).

## What the numbers mean

- **Code lines** are lines that aren't blank or comments. **Test lines** are `#[cfg(test)]`
  modules and files under `tests/`, `benches/` and `fuzz/`.
- **Growth.**
  - "Weft" is `git diff <fork> <upstream merged>`, what our merges of Weft brought.
  - "Ours" is `git diff <upstream merged> HEAD`.
  - Both count raw `.rs` lines, comments and blanks included.
- **Duplicate blocks** are 8 or more identical trimmed code lines in two places, with at least
  200 characters between them so runs of braces don't count.
- **Functions written more than once:**
  - same signature apart from the name, and the same body;
  - 5+ lines, or a smaller helper copied under one name into 3+ places (`hex`, `dist2`).
- **Dependencies never named:** a name search, so check each one by hand.
