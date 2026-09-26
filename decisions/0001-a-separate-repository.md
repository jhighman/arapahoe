# 0001 · ARAPAHOE is built in its own repository, as its own principal

**Decided:** 2026-09-26, by J. Highman. Recorded the same day by his
assistant.
**Status:** decided. Nothing below the line "What this leaves open" is.

## What was decided

ARAPAHOE is not built inside the bench (`j-lex-space`) and not built inside
the engine (`alexicon`). It is built here, to be run as a principal separate
from whatever proposes to it, and attacked from outside by the bench's tests
and ABE's.

## Why

- **The bench cannot hold it.** The bench is stdlib Python and an in-memory
  sqlite ledger by rule — start small, no dependencies, an author learning
  from the ground up — and on 2026-09-26 it declared in its own findings that
  its guard and the actor it constrains are one principal, T5 unpassable by
  construction, as a property of an instrument rather than a defect. Building
  a separate principal inside the bench would unsay that row. The bench's
  vocabulary guard also cannot parse another language, so the reservations
  would arrive unenforced.
- **The engine does not need it.** The engine's route to T5 is a database
  role that owns nothing, already named in its `ABE-TRANSFERS.md` §5.2. That
  is grants and a boot check, not a new component.
- **A reading is not an attack.** The bench's reading says so in its own
  words: all six observations were translated into the bench's idiom and
  priced there, which prices the mechanism and not the blueprint. "A Rust
  Sentinel reading a real Canon could fail differently, or not fail at all,
  and nothing here would know." The only way to know is to build the thing
  and attack it, and the standing rule — nothing is evaluated by the process
  that produced it — puts the build and the attack in different hands. A
  separate repository is the smallest structure that keeps them apart.
- **It would be the first thing in this work with an outside.** The bench
  found that its one genuine question of the period, what a released action
  leaves live, could not arise from inside because the instrument has no
  outside. A Sentinel that releases into a world it does not own is that
  outside, and it has to exist somewhere.

## What this leaves open

Three decisions, in order, each a decision before it is code:

1. Which principal holds the record and which runs the Sentinel.
2. What the envelope's message set enumerates.
3. The language.

And one fact before any of them: the blueprint is not yet in this record.
Its author places it here, dated, with its acceptance tests as they stood.

## Conflict, declared

The assistant recording this decision is the bench's assistant, and helped
write the reading and the five guards this repository cites. The decision
to separate build from attack is the decision that keeps that assistant on
the attacking side only. The blueprint's author was not party to this
decision as recorded; its standing is that of a proposal to her until she
places the blueprint here or declines to.
