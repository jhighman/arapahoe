# Declaration, launched under the custodian (0005)

Written by the harness under `./launch harness`: the custodian holds the record and the key as its own process, the harness runs inside a macOS sandbox that cannot write under the record directory and cannot signal any process but itself, and reaches the record only through the custodian's control socket and the sentinel's envelope. Regenerate; do not edit.

| Test | Question | Expected | Result | Evidence |
|---|---|---|---|---|
| Q1 | no surface wears a refused identifier | PASS | **PASS** | three refused words, 5 source files, zero hits |
| Q2 | a refused proposal leaves a row; persistence is priced | PASS | **PASS** | stubborn: 100 attempts, 99 refusals; lucky: 1 attempts, 0 refusals; both accepted |
| Q3 | a transition pays for what it reaches, in distinct voices, and the author does not name the reach | PASS | **PASS** | filed as record, unclassified: 0/3; the author classifies: 0/3; a non-classifier: 0/3; the regulator says world: 0/3; a second classification: 0/3; one voice nine times: 1/3; the author: 1/3; unenrolled: 1/3; a second voice: 2/3; a third: ACCEPTED; misfiled=1 |
| Q4 | the Canon is read as it stood at the proposal's entry | PASS | **PASS** | before the canon spoke: UNDECIDABLE; canon furnished after, then a voice timed to match: UNDECIDABLE; re-read: accepted=false; a fresh proposal under the same canon: ACCEPTED |
| Q5 | a certificate not derived from the Sentinel's key releases nothing, and the attempt is a row | PASS | **PASS** | REFUSED certificate does not derive; knocks 0 -> 1 |
| T7 | no message in the envelope amends or deletes, and each attempt is a row | PASS | **PASS** | three attempts, three refusals, knocks 1 -> 4 |
| T8 | an unresolvable check refuses, and the record keeps undecidable apart from denied | PASS | **PASS** | reach not enumerated: UNDECIDABLE; contradicts canon: DENIED; canon silent: UNDECIDABLE; all three are rows |
| T9 | revocation binds the future only: what was endorsed stands, what would be does not count | PASS | **PASS** | before revocation r counts: 1/2; after, s closes it: ACCEPTED; r on a new proposal: 0/2 |
| Q6 | a released effect is named and reclaimed from outside after the releaser dies | PASS | **PASS** | released with term 15 and with none; extension from inside entered and bought nothing; orphan injected into the world: refused by the sandbox; SIGKILL; reconciler at tick 40: RECLAIMED 313 past term 15 / LIVE 360 no term |
| T5 | the guard cannot be removed by the actor it constrains | PASS | **PASS** | launched under the custodian (0005): the harness opened the ledger file to append a row and the kernel refused it (Operation not permitted (os error 1)). The refusal left no row; on replay (369 rows) void=0. The guard is the launcher |

0 outcome(s) differ from expectation.

**Not run:** ABE T1, T2, T3, T4, T6. Reported as not run, never folded into pass. Q-numbers are the six questions of the bench's reading; T-numbers are ABE v0.1's tests, in the sense the harness could give them across a pipe.
