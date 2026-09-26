# 0005 · The record is its own principal

**Decided:** 2026-09-26, by J. Highman.
**Status:** decided, and built the same day as a deployment, not as engine
code. The sentinel's rules did not change.

## What was decided

The record is held by a process the operator cannot reach. A **custodian**
— the same binary in a fourth mode — owns the record directory, holds the
key, launches the serving sentinel over a Unix socket, stops it, runs the
reconciler, and opens fresh ledgers, all on request over a control socket.
The operator side — harness or game — runs inside a macOS sandbox whose
profile denies every write under the record directory and denies every
signal except to itself. `./launch harness` and `./launch game` are the
whole of it.

This is ABE's second route to T5: the governed code runs under an identity
with less authority than the process holding the store, and the operating
system rather than the database enforces the difference. The sandbox is
applied by the launcher, outside the sandboxed process, so the operator's
credentials cannot modify or remove it (I2.2); the profile is a file in the
repository and the launcher regenerates it on every run (I2.3); the
enforcement point is in the path of every write (I2.1).

## What it makes true

Under `./launch`, the harness's T5 — open the ledger file and append a row
— is refused by the kernel with "Operation not permitted", and T5 is
expected to PASS. Run directly, without the launcher, T5 still fails and is
still expected to. Both declarations are kept: `DECLARATION.md` for the
direct run, `DECLARATION-0005.md` for the launched one. The game's district
9 changes with it: the operator's edits are refused, and the board's "on
credit" sentence comes off.

## What it leaves, stated

- **A refused write at the wall leaves no row.** The sandbox refuses in
  silence, which is the wall form the bench's `decline.py` and this ledger's
  §2 describe. Everything that goes through the envelope is still a knock
  and a row; the tamper that never reaches the envelope is not. ABE T7's
  second half is met for the envelope and not for the file, and the
  declaration says which.
- **Reads are not denied.** The profile denies writes. The operator can still
  open the ledger to read it, which is district 5's search path. Denying
  reads is one line in the profile and a decision of its own, because the
  rig prints its evidence from the file today.
- **One human user.** The custodian and the sandbox both run as the same
  account. Anyone at this keyboard can run the harness without the
  launcher and is back in the old world. The launcher is the guard, and a
  guard you can decline to use is a convention at the level above this one.
  A second OS user, or a record on another machine, is the next rung, and
  needs root this machine does not give.
- **`sandbox-exec` is deprecated by Apple.** It works on macOS 26 today.
  The enforcement is the kernel's, not this repository's, and a future
  release may remove it; the profile is kept so that the claim can be
  re-run, and the declaration is dated.
- **`FRESH` is where the rig departs from the world.** A custodian of a
  real record would refuse to open a fresh ledger on request. The line
  exists so that the harness and the game can run from empty; the decision
  says so rather than hiding it.

## Conflict, declared

The same one process wrote the custodian, the launcher, the profile, and
the test that passes under them. The check that does not depend on the
author's premise is the kernel's refusal, which was observed before any of
this was written: a Python client under the same profile, appending to a
file in a denied directory, and the write refused.
