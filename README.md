# ARAPAHOE

The place where the ARAPAHOE blueprint — a governed transaction lifecycle:
an unprivileged proposer's candidate crosses a narrow envelope, a Sentinel
verifies it against the Canon, the accepted transition is appended, a
receipt is returned, and only then is privileged execution released — is
built as a separate principal and attacked from outside.

Opened 2026-09-26. Decided the same day: see
[`decisions/0001-a-separate-repository.md`](decisions/0001-a-separate-repository.md).

## What is here, and what is not yet

**Not yet here: the blueprint.** No copy of ARAPAHOE exists in any public
record. The only public account of it is a reading in the bench,
[`j-lex-space/ARAPAHOE-READING.md`](https://github.com/jhighman/j-lex-space/blob/main/ARAPAHOE-READING.md),
which read a document dated 2026-09-26 and ran nothing. The blueprint enters
this repository when its author places it here, with its date and its
acceptance tests, and that entry is the first row of the record. Until then
every sentence in this repository is about a described architecture.

**Here: a target and a harness, and neither is the build.** Decided in
[`decisions/0002`](decisions/0002-the-attacking-side-builds-a-target.md):
`sentinel/` is a Sentinel-shaped target — a governed transaction lifecycle
over an append-only ledger, spoken to only through an enumerable envelope —
and `harness/` is the attacking side above it, which spawns the target as a
separate process and runs the six questions of the reading and the ABE
tests it can reach across a pipe. The target exists so the harness has
something to be wrong about. It is not ARAPAHOE, and it decides nothing for
ARAPAHOE; the three decisions below stay open. Rust, no dependencies:

```
~/.cargo/bin/cargo run -p harness
```

writes [`DECLARATION.md`](DECLARATION.md), expectation beside result, and
exits with the number of rows where they differ. T5 is expected to FAIL and
does, for the reason the row states.

**Not yet decided: the three decisions before the build.** In this order,
and none is taken:

1. **Which principal holds the record, and which runs the Sentinel.** The
   enforcement point belongs at a process and privilege boundary; a Sentinel
   linked into the proposer's process is one principal with it, whatever
   language it is written in. This is a deployment property and no code
   supplies it.
2. **What the envelope's message set enumerates.** The interface between
   proposer and Sentinel must be narrow enough to list, and the supervising
   side validates rather than trusts.
3. **The language.** Named last, so that it is chosen for what the first two
   decisions need and not for what its name promises. "Rust Sentinel" in the
   blueprint bundles a language claim with a privilege claim; here they are
   kept apart. The target is in Rust for the target's own reasons, stated in
   0002, and that is not this decision.

**Here: the standing questions.** Six, from the reading, each already priced
on the bench in its own idiom and none yet priced against ARAPAHOE itself.
[`TRANSFERS.md`](TRANSFERS.md) carries them, with what any build here owes
against each.

## Two roles, kept apart

Nothing is evaluated by the process that produced it. That is the bench's
standing rule, paid for four times in six days, and it is the reason this
repository exists apart from the bench and the engine. Two roles, and they
are not the same pair of hands or the same assistant:

- **The build.** The blueprint's author and their partner. Whatever is built
  here is built to be attacked, and its own tests are part of what the
  premise produced.
- **The attack.** The bench's authors and theirs. The nine tests of ABE and
  the six questions of the reading, run against the built thing from
  outside its process, results published with the failures in.

The one who will rule does not build ahead (the engine's R36). The
attacking side may write the attacks before the build exists; it may not
write the build.

## Adopted by citation

Nothing is vendored. Each of the following is cited, with the version or
digest that was read, and this repository takes no dependency on any of
them (the engine's R38).

- **The bench**, `~/Developer/j-lex-space`: the four rules that met the
  blueprint (a decline is a row; price in voices, never rows; a reading is
  derived at the entry of the thing judged; the unmeasured is charged as
  easy), and the sixth question, what a released action leaves live. Cited
  by guard: `experiments/envelope.py`, `weight.py`, `canon.py`,
  `outlives.py`, `vocabulary.py`.
- **ABE v0.1**, *Assignment-Bounded Execution*, as pinned in the engine at
  `alexicon/docs/abe/` (tree digest `4758b63…`): four invariants, nine
  tests, the substrate, and the form of a conformance claim. A build here
  publishes its `TESTS.md` results in that form, failures included, and a
  test not run is reported as not run.
- **The signal-boundary papers**, in the bench at `signal-boundary/`: the
  ladder of controls (prompt text; in-process guard; separate process with
  narrow IPC; a boundary the supervised side cannot address), and the
  finding that what strands on death is what a process wrote outside
  itself.

## Words

One word, one act, and the reservation reaches every surface here:

- Work is **assigned**. Authority is **delegated**, by a person, to a
  system, with a written reason, and re-checked at every read.
- What the Sentinel does when the invariants hold is **accept**. The
  blueprint's word for its certificate and its fourth stage is the
  identifier the bench refuses, and it is retired here without being
  written.
- An episode or a transaction is **closed**, at a moment, under named
  premises. The other word is not used.

The bench's `vocabulary.py` parses Python and cannot reach a build in
another language. The harness's Q1 scans both crates' source for the
refused words on every run; a build in a further language owes its own.
