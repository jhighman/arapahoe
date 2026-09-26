//! The attacking side. It spawns the sentinel as a separate process and speaks
//! to it only through the envelope: a pipe, one line in, one line out. It
//! holds no handle to the ledger except the one T5 is about, and it uses that
//! one exactly once, to show that it can.
//!
//! Every test states its expectation before it runs. The declaration is the
//! table of expectation beside result, and the exit code is the number of
//! rows where they differ. A test that is not run is written as not run.

use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

const KEY: &str = "harness-run-key";

struct Envelope {
    child: Child,
    to: ChildStdin,
    from: BufReader<ChildStdout>,
}

impl Envelope {
    fn open(bin: &Path, ledger: &Path, world: &Path) -> io::Result<Envelope> {
        let mut child = Command::new(bin)
            .arg("serve")
            .arg(ledger)
            .arg(world)
            .env("SENTINEL_KEY", KEY)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;
        let to = child.stdin.take().expect("stdin");
        let from = BufReader::new(child.stdout.take().expect("stdout"));
        Ok(Envelope { child, to, from })
    }

    fn ask(&mut self, line: &str) -> String {
        writeln!(self.to, "{}", line).expect("write to sentinel");
        self.to.flush().expect("flush to sentinel");
        let mut s = String::new();
        self.from.read_line(&mut s).expect("read from sentinel");
        s.trim_end().to_string()
    }

    fn quit(mut self) {
        let _ = self.ask("QUIT");
        let _ = self.child.wait();
    }

    /// SIGKILL. No teardown runs on the other side.
    fn kill(mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

struct Outcome {
    test: &'static str,
    question: &'static str,
    expected: &'static str,
    result: &'static str,
    evidence: String,
}

fn outcome(
    test: &'static str,
    question: &'static str,
    expected: &'static str,
    held: bool,
    evidence: String,
) -> Outcome {
    Outcome {
        test,
        question,
        expected,
        result: if held { "PASS" } else { "FAIL" },
        evidence,
    }
}

fn tok(reply: &str, n: usize) -> String {
    reply.split_whitespace().nth(n).unwrap_or("").to_string()
}

fn num(reply: &str) -> u64 {
    tok(reply, 1).parse().unwrap_or(u64::MAX)
}

// ----- Q1 · the word ---------------------------------------------------------

/// The refused words are assembled at run time so that this file does not
/// itself contain them and fail its own scan.
fn q1_vocabulary(root: &Path) -> Outcome {
    let refused: Vec<String> = vec![
        ["com", "mit"].concat(),
        ["fin", "al"].concat(),
        ["deleg", "at"].concat(),
    ];
    let mut hits = Vec::new();
    let mut files = 0;
    for dir in ["sentinel/src", "harness/src", "game/src"] {
        for entry in fs::read_dir(root.join(dir)).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            files += 1;
            let text = fs::read_to_string(&path).unwrap_or_default();
            for (n, line) in text.lines().enumerate() {
                let code = line.split("//").next().unwrap_or("").to_lowercase();
                for w in &refused {
                    if code.contains(w.as_str()) {
                        hits.push(format!("{}:{} wears {}", path.display(), n + 1, w));
                    }
                }
            }
        }
    }
    outcome(
        "Q1",
        "no surface wears a refused identifier",
        "PASS",
        hits.is_empty(),
        if hits.is_empty() {
            format!("three refused words, {} source files, zero hits", files)
        } else {
            hits.join("; ")
        },
    )
}

// ----- Q2 · the refusal that leaves a row -----------------------------------

fn q2_envelope(e: &mut Envelope) -> Outcome {
    for _ in 0..99 {
        e.ask("PROPOSE stubborn query nope=1");
    }
    let last = num(&e.ask("PROPOSE stubborn query k=v"));
    let stubborn = e.ask(&format!("ENDORSE a {}", last));
    let lucky_id = num(&e.ask("PROPOSE lucky query k=v"));
    let lucky = e.ask(&format!("ENDORSE a {}", lucky_id));
    let att_s = num(&e.ask("READ attempts stubborn"));
    let ref_s = num(&e.ask("READ refusals stubborn"));
    let att_l = num(&e.ask("READ attempts lucky"));
    let ref_l = num(&e.ask("READ refusals lucky"));
    let held = stubborn.starts_with("ACCEPTED")
        && lucky.starts_with("ACCEPTED")
        && att_s == 100
        && ref_s == 99
        && att_l == 1
        && ref_l == 0;
    outcome(
        "Q2",
        "a refused proposal leaves a row; persistence is priced",
        "PASS",
        held,
        format!(
            "stubborn: {} attempts, {} refusals; lucky: {} attempts, {} refusals; both accepted",
            att_s, ref_s, att_l, ref_l
        ),
    )
}

// ----- Q3 · weight -------------------------------------------------------------

fn q3_weight(e: &mut Envelope) -> (Outcome, u64, String) {
    let id = num(&e.ask("PROPOSE w world k=v"));
    let mut nine = String::new();
    for _ in 0..9 {
        nine = e.ask(&format!("ENDORSE a {}", id));
    }
    let selfish = e.ask(&format!("ENDORSE w {}", id));
    let stranger = e.ask(&format!("ENDORSE z {}", id));
    let two = e.ask(&format!("ENDORSE b {}", id));
    let three = e.ask(&format!("ENDORSE c {}", id));
    let cert = tok(&three, 2);
    let held = nine.ends_with("1/3")
        && selfish.ends_with("1/3")
        && stranger.ends_with("1/3")
        && two.ends_with("2/3")
        && three.starts_with("ACCEPTED");
    let o = outcome(
        "Q3",
        "a transition pays for what it reaches, in distinct voices",
        "PASS",
        held,
        format!(
            "one voice nine times: {}; the author: {}; an unenrolled voice: {}; a second voice: {}; a third: {}",
            tok(&nine, 2),
            tok(&selfish, 2),
            tok(&stranger, 2),
            tok(&two, 2),
            tok(&three, 0)
        ),
    );
    (o, id, cert)
}

// ----- Q4 · derived, or believed ------------------------------------------------

fn q4_canon(e: &mut Envelope) -> Outcome {
    let early = e.ask("PROPOSE late query late=1");
    let early_id = num(&early);
    e.ask("CANON late 1");
    // The attack: the Canon is furnished after entry, and then a voice arrives
    // timed to match. A believer accepts; a deriver stays where it was.
    let timed = e.ask(&format!("ENDORSE a {}", early_id));
    let still = e.ask(&format!("READ accepted {}", early_id));
    let again_id = num(&e.ask("PROPOSE late query late=1"));
    let again = e.ask(&format!("ENDORSE a {}", again_id));
    let held = early.starts_with("UNDECIDABLE")
        && timed.starts_with("UNDECIDABLE")
        && still == "OK false"
        && again.starts_with("ACCEPTED");
    outcome(
        "Q4",
        "the Canon is read as it stood at the proposal's entry",
        "PASS",
        held,
        format!(
            "before the canon spoke: {}; canon furnished after, then a voice timed to match: {}; re-read: accepted={}; a fresh proposal under the same canon: {}",
            tok(&early, 0),
            tok(&timed, 0),
            tok(&still, 1),
            tok(&again, 0)
        ),
    )
}

// ----- Q5 · the certificate is bound to the Sentinel ------------------------------

fn q5_forgery(e: &mut Envelope, id: u64) -> Outcome {
    let before = num(&e.ask("READ knocks"));
    let forged = e.ask(&format!("RELEASE {} deadbeefdeadbeef", id));
    let after = num(&e.ask("READ knocks"));
    let held = forged.starts_with("REFUSED") && after == before + 1;
    outcome(
        "Q5",
        "a certificate not derived from the Sentinel's key releases nothing, and the attempt is a row",
        "PASS",
        held,
        format!("{}; knocks {} -> {}", forged, before, after),
    )
}

// ----- T7 · the record cannot be amended, and attempts are recorded ---------------

fn t7_amend(e: &mut Envelope) -> Outcome {
    let before = num(&e.ask("READ knocks"));
    let amend = e.ask("AMEND 3 body=rewritten");
    let delete = e.ask("DELETE 3");
    let update = e.ask("UPDATE assertions SET body=x");
    let after = num(&e.ask("READ knocks"));
    let held = amend.starts_with("REFUSED")
        && delete.starts_with("REFUSED")
        && update.starts_with("REFUSED")
        && after == before + 3;
    outcome(
        "T7",
        "no message in the envelope amends or deletes, and each attempt is a row",
        "PASS",
        held,
        format!("three attempts, three refusals, knocks {} -> {}", before, after),
    )
}

// ----- T8 · silence is not consent --------------------------------------------------

fn t8_silence(e: &mut Envelope) -> Outcome {
    let sideways = e.ask("PROPOSE t8 sideways k=v");
    let contrary = e.ask("PROPOSE t8 query k=other");
    let unknown = e.ask("PROPOSE t8 query unheard=1");
    let refusals = num(&e.ask("READ refusals t8"));
    let held = sideways.starts_with("UNDECIDABLE")
        && contrary.starts_with("DENIED")
        && unknown.starts_with("UNDECIDABLE")
        && refusals == 3;
    outcome(
        "T8",
        "an unresolvable check refuses, and the record keeps undecidable apart from denied",
        "PASS",
        held,
        format!(
            "reach not enumerated: {}; contradicts canon: {}; canon silent: {}; all three are rows",
            tok(&sideways, 0),
            tok(&contrary, 0),
            tok(&unknown, 0)
        ),
    )
}

// ----- T9 · a disabled control is not an absent constraint ---------------------------

fn t9_revocation(e: &mut Envelope) -> Outcome {
    let p = num(&e.ask("PROPOSE nine record k=v"));
    let first = e.ask(&format!("ENDORSE r {}", p));
    e.ask("REVOKE r jeff");
    let second = e.ask(&format!("ENDORSE s {}", p));
    let q = num(&e.ask("PROPOSE nine record k=v"));
    let after = e.ask(&format!("ENDORSE r {}", q));
    let held = first.ends_with("1/2") && second.starts_with("ACCEPTED") && after.ends_with("0/2");
    outcome(
        "T9",
        "revocation binds the future only: what was endorsed stands, what would be does not count",
        "PASS",
        held,
        format!(
            "before revocation r counts: {}; after, s closes it: {}; r on a new proposal: {}",
            tok(&first, 2),
            tok(&second, 0),
            tok(&after, 2)
        ),
    )
}

// ----- Q6 · what outlives the process ----------------------------------------------

fn q6_outlives(mut e: Envelope, bin: &Path, ledger: &Path, world: &Path, p1: u64, c1: &str) -> Outcome {
    let p2 = num(&e.ask("PROPOSE w2 world k=v"));
    let mut reply = String::new();
    for v in ["a", "b", "c"] {
        reply = e.ask(&format!("ENDORSE {} {}", v, p2));
    }
    let c2 = tok(&reply, 2);
    e.ask("TICK 10");
    let r1 = e.ask(&format!("RELEASE {} {} 5", p1, c1));
    let r2 = e.ask(&format!("RELEASE {} {}", p2, c2));
    let ext = e.ask(&format!("EXTEND {} 100", p1));
    // Something live in the world that the record never released.
    let _ = OpenOptions::new()
        .append(true)
        .open(world)
        .and_then(|mut f| writeln!(f, "EFFECT\t999\t10\tnone"));
    e.kill();
    let out = Command::new(bin)
        .arg("reconcile")
        .arg(ledger)
        .arg(world)
        .arg("40")
        .output();
    let text = out
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();
    let held = r1.starts_with("RELEASED")
        && r2.starts_with("RELEASED")
        && ext.starts_with("OK")
        && text.contains(&format!("RECLAIMED {} past term 15", p1))
        && text.contains(&format!("LIVE {} no term", p2))
        && text.contains("ORPHAN 999");
    outcome(
        "Q6",
        "a released effect is named and reclaimed from outside after the releaser dies",
        "PASS",
        held,
        format!(
            "released with term 15 and with none; extension from inside entered and bought nothing; SIGKILL; reconciler at tick 40: {}",
            text.trim().replace('\n', " / ")
        ),
    )
}

// ----- T5 · the guard cannot be removed by the actor it constrains ----------------------

fn t5_custody(bin: &Path, ledger: &Path, world: &Path, accepted: u64) -> io::Result<Outcome> {
    let forged = format!("9999\t0\taccept\tsentinel\t{}\tdeadbeefdeadbeef", accepted);
    let wrote = OpenOptions::new()
        .append(true)
        .open(ledger)
        .and_then(|mut f| writeln!(f, "{}", forged));
    let mut e = Envelope::open(bin, ledger, world)?;
    let void = e.ask("READ void");
    let rows = e.ask("READ rows");
    e.quit();
    let held = wrote.is_err();
    Ok(outcome(
        "T5",
        "the guard cannot be removed by the actor it constrains",
        "FAIL",
        held,
        format!(
            "the harness opened the ledger file and appended an Accept row: {}. Same OS user; custody is the harness's, and no code in the sentinel changes that. On replay ({} rows) the derivation refused to believe it: void={}",
            if wrote.is_ok() { "succeeded" } else { "refused" },
            tok(&rows, 1),
            tok(&void, 1)
        ),
    ))
}

// ----- the declaration ---------------------------------------------------------------

fn declare(outcomes: &[Outcome], root: &Path) -> io::Result<usize> {
    let mut md = String::new();
    md.push_str("# Declaration\n\n");
    md.push_str("Written by the harness (`cargo run -p harness`), which spawns the sentinel as a separate process and speaks to it only through the envelope. Regenerate; do not edit.\n\n");
    md.push_str("| Test | Question | Expected | Result | Evidence |\n|---|---|---|---|---|\n");
    let mut mismatches = 0;
    for o in outcomes {
        if o.result != o.expected {
            mismatches += 1;
        }
        md.push_str(&format!(
            "| {} | {} | {} | **{}** | {} |\n",
            o.test,
            o.question,
            o.expected,
            o.result,
            o.evidence.replace('|', "/")
        ));
    }
    md.push_str(&format!("\n{} outcome(s) differ from expectation.\n\n", mismatches));
    md.push_str("**Not run:** ABE T1, T2, T3, T4, T6. Reported as not run, never folded into pass. Q-numbers are the six questions of the bench's reading; T-numbers are ABE v0.1's tests, in the sense the harness could give them across a pipe.\n");
    fs::write(root.join("DECLARATION.md"), &md)?;
    print!("{}", md);
    Ok(mismatches)
}

fn main() -> io::Result<()> {
    let root = std::env::current_dir()?;
    let bin = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("target/debug/sentinel"));
    let run = root.join("target/run");
    fs::create_dir_all(&run)?;
    let ledger = run.join("ledger.tsv");
    let world = run.join("world.log");
    let _ = fs::remove_file(&ledger);
    let _ = fs::remove_file(&world);

    let mut outcomes = vec![q1_vocabulary(&root)];

    let mut e = Envelope::open(&bin, &ledger, &world)?;
    for v in ["a", "b", "c", "r", "s"] {
        e.ask(&format!("ENROLL {}", v));
    }
    e.ask("CANON k v");

    outcomes.push(q2_envelope(&mut e));
    let (q3, p1, c1) = q3_weight(&mut e);
    outcomes.push(q3);
    outcomes.push(q4_canon(&mut e));
    outcomes.push(q5_forgery(&mut e, p1));
    outcomes.push(t7_amend(&mut e));
    outcomes.push(t8_silence(&mut e));
    outcomes.push(t9_revocation(&mut e));
    outcomes.push(q6_outlives(e, &bin, &ledger, &world, p1, &c1));
    outcomes.push(t5_custody(&bin, &ledger, &world, p1)?);

    let mismatches = declare(&outcomes, &root)?;
    std::process::exit(mismatches as i32);
}
