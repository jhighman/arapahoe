# 0003 · The game lives in this repository

**Decided:** 2026-09-26, by J. Highman.
**Status:** decided.

## What was decided

*Protocol: Fail-Closed* — a terminal game with one district per question of
Alexandra Krížová's "Safeguards Are Not Invariants: Nine Questions for a
Machine That Remembers a Child" (22 September 2026) — is built here, in
`game/`, over the target sentinel of 0002, as a third crate.

## What it is and is not

It is an application above the engine. The player is the architect of a
district; the sentinel is the district's record, run as a separate process
and spoken to only through the envelope; the operator's moves are the
article's furniture changes; the reconciler is who else would know.

It is not the build (0001) and not the attack (0002). It changes nothing in
`sentinel/` and `harness/`, takes no dependency on either crate, and runs
the sentinel binary the way the harness does. A district that needs a
message the envelope does not have is a decision here before it is code
there.

## Why here and not its own repository

The recommendation on the record was a separate repository, since a game is
neither a target nor an attack. J.H. chose this one: the game is the
application the harness was waiting for, and it exercises the same binary
under the same rules. The cost is stated: a reader of this repository now
finds three things in it, and the README has to keep them apart.

## What this leaves open

Districts 2 through 9. District 4 — did Monday become part of the voice —
has no answer in any of the three repositories, and is to be built as the
level nobody has solved rather than left out.
