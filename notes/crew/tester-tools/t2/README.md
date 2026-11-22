# PC job queue: Mac-side client (Tester 2)

Rebuilt 11-22 after `~/Coding/chiacraft-archive` was wiped (the old `t2q.py` and `pcwatch.sh` went with it). The runner lives on the
PC (`ronaldg@192.168.1.13`, `~/looktest/queue/runner.py`, log `queue/runner.log`, started by `restart_runner.sh`).

- `t2q.py` queues builds, looks, benches and multi-player runs, and shows `status`, sets a `hold`, fetches results and cancels jobs
  (`python3 notes/crew/tester-tools/t2/t2q.py --help`). The job files it writes are the runner's own format (kind, id, tree, script,
  extra, needs, limit, view, prio, ...); `look <id> @dev script.txt world` builds the tip of `dev` once as tree `C_<short>` and runs on it.
  `--preset max` adds the Max look settings (the benchmark default); a second `--config` in the extra args replaces the first.
- `pcwatch.sh` prints a line when the PC does not answer, the runner is down, `/home` has under 30 GB free, the GPU is over 85 C or a
  job runs far past its limit.
- `restart_runner.sh` restarts the runner (it refuses new builds below 30 GB free and prunes old `C_*/target` dirs below 40 GB).
- Results: shots and logs on the PC in `~/looktest/t2_<id>/`; `t2q.py fetch <id> [date]` copies them to
  `~/Coding/chiacraft-archive/images/tester-2/<date>/runs/t2_<id>/`.
- A build needs its source staged first (`~/stage_<commit>`, written by `git archive`); `t2q.py` does it, and drops the job only once
  the copy is whole (queueing a job by hand before the copy ends builds a half tree).
