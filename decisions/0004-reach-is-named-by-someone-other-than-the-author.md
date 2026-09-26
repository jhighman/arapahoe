# 0004 · Reach is named by someone other than the author

**Decided:** 2026-09-26, by J. Highman.
**Status:** decided, and built the same day. The first engine change since
0002.

## What was decided

A proposal's price in voices derives from its **classified reach**, not from
the reach the author wrote in the proposal. Classification is a row:
`CLASSIFY <voice> <id> <reach>`. It counts only when the voice was enrolled
as a classifier before it spoke, is not the author, and was not revoked
before it spoke. The first counting classification is the reach; later ones
enter and buy nothing, as with every other "first written" rule here. A
proposal with no counting classification is priced as reaching the world,
because unplaced is unpayable rather than free.

The author's own reach token stays in the proposal. It must still be one of
the enumerated words, or the proposal is undecidable; it carries no price;
and where it disagrees with the classification the record surfaces the
disagreement (`READ misfiled`) rather than resolving it.

## Why

The streaks district found it on 2026-09-26: the operator filed a nudge to
a citizen's phone as reaching only the record, paid two payroll voices for
what the world costs three, and the harness's Q3 had never asked. The
bench's `weight.py` refutes exactly this — "a reach an author can set for
itself" — and `TRANSFERS.md` §4 owed the difference. This pays it.

The rule is the bench's `unmeasured.py` rule at the pricing step: where the
record cannot say what a transition reaches, it charges as though it reaches
everything. The other default is a discount available to anyone willing to
skip the classifier.

## What this leaves open, stated

**Who may enroll a classifier.** Enrolment is unauthenticated: `ENROLL
<name> classifier` is a line anyone at the envelope can send, and the
operator can enroll its own head of growth as the classifier of its own
nudges. The record then holds a classifier row and cannot say who wrote it.
That is the founding-roster question the bench has held open since August,
arriving here in the smallest possible form, and it is answered by 0006
(voices as keys) and by whatever seals the roster, not by this decision.
The game's streaks district shows the hole rather than hiding it.

## Conflict, declared

The assistant that found the hole wrote the fix and the test for it, and
then broke the fix to see the test go red. The mutation is recorded in
`TRANSFERS.md` §2. It is the same one process as before, and the same
caveat applies.
