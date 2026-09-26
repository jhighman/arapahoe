# ARAPAHOE

A small engine that keeps a record and refuses to be persuaded, a harness
that attacks it from outside its own process, and a game that plays a
regulator's questions against it. Rust, no dependencies, one binary for the
engine. Everything here exists to demonstrate one principle and to find out
where it fails.

> The Sentinel Principle requires every consequential increase in a claim's
> or action's authority to cross an independently governed boundary, with
> its warrant, scope, uncertainty, and accountable history preserved.

If you have five minutes, skip to **Five minutes**. If you want to know
what you are looking at first, read on.

## The principle, and where each part of it lives

The principle has six parts. Each is a rule in this code, and each is also
something the harness breaks on purpose to check that the rule is real.

1. **Trust attaches to a governed passage, not to a credible-looking
   result.** A proposal enters the record as a row. Its verdict is derived
   every time it is read, from the rows as they stood when it entered, and
   never stored and later believed. A certificate is bound to one proposal
   and spends once. (`sentinel/src/lib.rs`, `verdict`, `certificate`.)
2. **Independence must be real at the boundary.** A proposal is priced in
   distinct voices, and neither the author nor the judge is one of them. The
   author cannot name what its proposal reaches; a classifier does. The
   harness itself never links the engine; it launches it and speaks to it
   only through an enumerable envelope. (`voices`, `reach_of`; `harness/`;
   `decisions/0004`.)
3. **Uncertainty is a legitimate state.** Verdicts are three-valued. Where
   the Canon is silent the answer is UNDECIDABLE, recorded as its own act,
   kept apart from DENIED, and never treated as permission. (`Verdict`,
   `Act::Undecided`.)
4. **Authority is bounded and time-sensitive.** A release into the world
   carries a term or the record says it carries none. A term cannot be
   extended from inside. A reconciler that shares no fate with the releaser
   reads the record afterwards and says what is still live. (`release`,
   `reconcile`.)
5. **Accountability requires memory and correction.** The ledger is
   append-only: no update, no delete, and a correction is a new row that a
   later derivation prefers. A refusal is a row. A line the envelope does
   not understand is a row. (`sentinel/src/ledger.rs`; `Act::Knock`.)
6. **The strongest implementation is structural.** The record can be held
   by a process the operator cannot write to, with the operator's side run
   under a kernel sandbox that the operator's own code cannot remove.
   (`sentinel custodian`, `launch`; `decisions/0005`.)

**The principle's limit is kept in view.** None of this shows that a verdict
is correct. What it shows is that a change in a claim's status is visible,
conditional, attributable, and challengeable from outside. The engine has
been broken four ways on purpose and had to go red each time; one of those
mutations turned nothing red on its first run, and that is recorded rather
than smoothed over (`TRANSFERS.md` §2). Two of the game's verdicts are
answers nobody can improve: UNSOLVED, for a question no record can answer,
and UNPASSABLE HERE, for one that is deployment rather than code.

## Five minutes

You need a Rust toolchain (`cargo`). On macOS the launcher also uses
`sandbox-exec`, which ships with the system.

```
cargo build
cargo run -p harness            # attacks the engine; writes DECLARATION.md
./launch harness                # the same, with the record held by its own process
cargo run -p game               # the game, asking you at each choice
./launch game -- --invariant    # the game under the custodian, best move everywhere
```

What you will see:

- **The harness** prints a table of ten tests with the expected result
  beside the actual one, and exits with the number that differ. Run
  directly, one test is expected to fail and does: the harness can open the
  ledger file, because it runs as the same user. Under `./launch` that same
  test passes, because the kernel refuses the write. Both declarations are
  kept in the repository.
- **The game**, *Protocol: Fail-Closed*, is a prologue and thirteen
  districts. Every line that crosses the envelope is shown, so you watch the
  record being written. Each district ends with a verdict derived from the
  rows: SAFEGUARD, INVARIANT, UNSOLVED, or UNPASSABLE HERE. The closing
  screen tells you what your invariants are resting on.

## What is here

| Path | What it is |
| --- | --- |
| `sentinel/` | The engine. An append-only ledger, a thirteen-message envelope, derived verdicts, releases with terms, a reconciler, and a custodian mode that holds the record as its own principal |
| `harness/` | The attack. Spawns or connects to the engine, runs the six questions of the bench's reading and the ABE tests it can reach across a pipe, and writes the declaration |
| `game/` | *Protocol: Fail-Closed*. A prologue (the door; the definition of "invariant" you sign), three engagement loops (streaks, scroll, autoplay), and nine districts from Alexandra Krížová's [nine questions for a machine that remembers a child](https://www.linkedin.com/pulse/safeguards-invariants-nine-questions-machine-child-alexandra-kr%C3%AD%C5%BEov%C3%A1-bvvrf/) |
| `launch` | Starts the custodian unconstrained and runs the operator side inside a sandbox that cannot write under the record and cannot signal any process but itself |
| `decisions/` | One file per decision, numbered, never edited afterwards. What was decided, by whom, why, what it leaves open, and whose conflict it is |
| `TRANSFERS.md` | The ledger between this repository and the two it cites: what was adopted, what was found, what is owed |
| `DECLARATION.md`, `DECLARATION-0005.md` | The harness's results, direct and under the custodian. Regenerated, never edited |
| `private/` | Local-only correspondence between the authors. Nothing under it is tracked or quoted |

## What this is not

- **Not the ARAPAHOE blueprint.** The name belongs to a design for a
  governed transaction lifecycle that is not yet in any public record. The
  engine here is a target shaped like it, built by the attacking side so
  the attacks would have something to be wrong about (`decisions/0002`). The
  blueprint's own build, when it exists, will be attacked by the same
  harness, and this target retired.
- **Not conformant.** Against ABE v0.1, *Assignment-Bounded Execution*, three
  tests pass, one passes only under the custodian, and five are reported
  as not run. Nothing here is folded into a pass.
- **Not a proof.** A green harness measures the questions its author thought
  to ask. The mutation record is the only part that does not depend on the
  author's premise, and it is short.
- **Not finished, and not meant to look it.** What is owed is listed in
  `TRANSFERS.md` §4 in order of what each promise is worth.

## Rules kept while working here

- **One word, one act.** Work is *assigned*; authority is *delegated*, by
  a person, with a written reason. What the engine does when the invariants
  hold is *accept*. An episode is *closed*. Two other words are refused
  outright, and the harness scans every crate for them on every run.
- **Append beside, never over.** No path updates or deletes a row, for any
  reason.
- **Derived, never stored.** A verdict, a certificate, a reading: recomputed
  from the rows at the entry of the thing judged.
- **A refusal is a row.** A gate that refuses the row cannot keep its own
  refusal ledger, so the row enters and the spending is refused.
- **Nothing is evaluated by the process that produced it.** The build and
  the attack are different hands. A guard is not trusted until it has been
  broken and seen to go red.
- **A decision before it is code.** Every change to the engine has a file
  in `decisions/` first.

## Adopted by citation

Nothing is vendored and no dependency is taken. Each of these is cited with
the version that was read.

- **The bench**, [j-lex-space](https://github.com/jhighman/j-lex-space): a
  reference instrument in Python and in-memory sqlite, thirteen guards,
  and a failure record kept at the same length as the successes. Its
  reading of the blueprint is the harness's six questions, and its guards
  `envelope.py`, `weight.py`, `canon.py`, `outlives.py` and `vocabulary.py`
  are the rules this engine reproduces one floor up.
- **ABE v0.1**, *Assignment-Bounded Execution*: four invariants, nine
  tests, a substrate, and the form of a conformance claim. Adopted in the
  engine that cites it at `alexicon/docs/abe/`, tree digest `4758b63…`.
- **The signal-boundary papers**, in the bench at `signal-boundary/`: the
  ladder of controls, and the finding that what strands when a process dies
  is what it wrote outside itself.
- **"Safeguards Are Not Invariants: Nine Questions for a Machine That
  Remembers a Child"**, Alexandra Krížová, 22 September 2026: the game's
  nine districts and its definition of an invariant, what remains true after
  the company is still allowed to change the furniture.

## For the authors

Opened 2026-09-26. Five decisions taken: a separate repository (0001), the
attacking side builds a target (0002), the game lives here (0003), reach is
named by someone other than the author (0004), the record is its own
principal (0005). Three decisions still open for the blueprint's build, in
this order and none taken: which principal holds the record and which runs
the Sentinel; what the envelope enumerates; the language. The blueprint
enters this record when its author places it here, dated and unchanged.
