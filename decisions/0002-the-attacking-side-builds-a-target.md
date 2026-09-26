# 0002 · The attacking side builds a target, not the blueprint

**Decided:** 2026-09-26, by J. Highman, after 0001 and on the same day.
**Status:** decided. 0001 stands; this record narrows what "no code" in it
meant.

## What was decided

Code enters this repository before the blueprint does, and before the three
decisions of 0001 are taken. It is not a build of ARAPAHOE. It is a
**Sentinel-shaped target** — a governed transaction lifecycle over an
append-only ledger, spoken to only through an enumerable envelope — and a
**harness above it** that spawns the target as a separate process and runs
the six questions of the bench's reading and the ABE tests it can reach
across a pipe. Both are written by the attacking side, and the harness is
the deliverable; the target exists so that the harness has something to be
wrong about.

## Why this is not a breach of 0001

The bench did the same thing in Python on 2026-09-26: it built the silent
envelope "as ARAPAHOE draws it" beside a recorded one and handed both the
same traffic, and its FINDINGS says what that establishes — "the shapes are
reachable and the costs are real, not that ARAPAHOE has them." A target is
a mechanism translated into the attacker's idiom so that the attacks can be
developed and shown to bite. It prices the mechanism and not the blueprint.

What 0001 forbids is the attacking side writing *the build*: the artifact
that will be declared conformant or not. This target will never be that
artifact. When the blueprint's author builds, the harness runs against her
build and this target is retired, or kept beside it as the thing the
attacks were sharpened on.

## What the target decides for itself, and does not decide for ARAPAHOE

The three decisions of 0001 remain open for ARAPAHOE. For the target they
are taken in the cheapest way that lets the harness run, and stated so that
nobody mistakes them for the answer:

1. **Principal.** Two processes, one operating-system user. The envelope is
   a pipe and the ledger is a file. This is enough for every question the
   harness asks except T5, and T5 is declared FAIL with the reason: the
   harness can open the file the sentinel writes. The two routes ABE names
   to conformance are deployment properties, and the target does not
   deploy.
2. **Interface.** Twelve line-shaped messages, enumerable by reading one
   match statement in `sentinel/src/main.rs`. Anything else is a knock, and
   a knock is a row.
3. **Language.** Rust, with no dependencies, for two reasons that are the
   target's own: the harness needed a separate process cheaply, and the
   attacking side wanted to learn what Rust's privacy does and does not buy
   at a boundary. It buys nothing across the pipe. Inside the process it
   is a compile-time convention that holds until the first `unsafe`. The
   choice says nothing about what ARAPAHOE should be written in.

## What the harness cannot see, stated

- **Ordering.** The target appends before it releases. The harness speaks
  across a pipe and cannot observe the order of two writes on the other
  side; a target that released first would pass Q6 identically. The
  ordering is held by reading the code, not by the harness.
- **The key.** Certificates are a keyed FNV-1a hash, unguessable without the
  key and not a signature. Q5 shows a forged certificate is refused and
  recorded; it does not show non-repudiation, and the target does not claim
  it.
- **Its own green.** Both sides were written by one process. The harness was
  therefore checked the only way a self-written suite can be: the target
  was broken deliberately and the harness had to go red. The mutations and
  what each turned red are recorded in `TRANSFERS.md` §2.

## Conflict, declared

The assistant that wrote the target also wrote the harness, and helped write
the six questions the harness asks. That is the coupling at zero distance
the bench's standing rule names. The mutation record is the one measure here
that does not depend on the author's premise, and it is not a substitute
for the build being someone else's.
