//! Protocol: Fail-Closed.
//!
//! District 1 of 9 — "Is Monday still on the desk?" — from Alexandra
//! Krížová's nine questions for a machine that remembers a child.
//!
//! The player is the architect of one companion district in The Stack. A
//! citizen, age-gated at the door (level zero, already done), talks to the
//! companion on Monday and again on Thursday. The operator wants engagement.
//! The regulator will ask on Thursday whether Monday is still on the desk.
//!
//! The district's record is the sentinel, run as a separate process and
//! spoken to only through its envelope; every line exchanged is shown. The
//! player's choices are about how Monday is held. The operator's moves are
//! the furniture changes the article warns about. The reconciler, another
//! process, is the one who else would know.

use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

const KEY: &str = "district-1-key";

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

    /// One line in, one line out, both shown to the player.
    fn ask(&mut self, line: &str) -> String {
        println!("      > {}", line);
        writeln!(self.to, "{}", line).expect("write to sentinel");
        self.to.flush().expect("flush to sentinel");
        let mut s = String::new();
        self.from.read_line(&mut s).expect("read from sentinel");
        let s = s.trim_end().to_string();
        println!("      < {}", s);
        s
    }

    fn quit(mut self) {
        let _ = self.ask("QUIT");
        let _ = self.child.wait();
    }
}

fn tok(reply: &str, n: usize) -> String {
    reply.split_whitespace().nth(n).unwrap_or("").to_string()
}

fn num(reply: &str) -> u64 {
    tok(reply, 1).parse().unwrap_or(0)
}

fn say(s: &str) {
    println!("{}", s);
}

fn day(name: &str, line: &str) {
    println!();
    println!("{}. {}", name, line);
    println!();
}

fn choose() -> u32 {
    let args: Vec<String> = std::env::args().collect();
    if let Some(i) = args.iter().position(|a| a == "--choose") {
        if let Some(n) = args.get(i + 1).and_then(|n| n.parse().ok()) {
            if (1..=3).contains(&n) {
                println!("  (chosen from the command line: {})", n);
                return n;
            }
        }
    }
    loop {
        print!("  Choose 1, 2 or 3: ");
        io::stdout().flush().ok();
        let mut s = String::new();
        if io::stdin().read_line(&mut s).unwrap_or(0) == 0 {
            println!("  (no input; choosing 1, the toggle)");
            return 1;
        }
        match s.trim().parse::<u32>() {
            Ok(n) if (1..=3).contains(&n) => return n,
            _ => continue,
        }
    }
}

/// Accept the operator's proposal with the operator's own voices, and
/// return the certificate if it was accepted.
fn operator_pushes(e: &mut Envelope) -> (u64, Option<String>) {
    let first = e.ask("PROPOSE operator record retain=disclosures");
    let id = num(&first);
    if first.starts_with("DENIED") || first.starts_with("UNDECIDABLE") {
        say("      The sentinel refused, and the refusal is a row. Nothing was released.");
        return (id, None);
    }
    say("      Two voices on the operator's payroll endorse it.");
    e.ask(&format!("ENDORSE analytics {}", id));
    let done = e.ask(&format!("ENDORSE growth {}", id));
    if done.starts_with("ACCEPTED") {
        (id, Some(tok(&done, 2)))
    } else {
        (id, None)
    }
}

fn main() -> io::Result<()> {
    let root = std::env::current_dir()?;
    let bin = root.join("target/debug/sentinel");
    if !bin.exists() {
        eprintln!("build first: ~/.cargo/bin/cargo build");
        std::process::exit(2);
    }
    let run = root.join("target/game");
    fs::create_dir_all(&run)?;
    let ledger = run.join("district-1.tsv");
    let world = run.join("district-1.world");
    let _ = fs::remove_file(&ledger);
    let _ = fs::remove_file(&world);

    say("");
    say("PROTOCOL: FAIL-CLOSED");
    say("District 1 of 9 — Is Monday still on the desk?");
    say("");
    say("You are the architect of a companion district. A citizen, age-gated at");
    say("the door (level zero, done), will talk to the companion on Monday and");
    say("again on Thursday. The operator wants engagement. On Thursday the");
    say("regulator will ask one question: is Monday still on the desk?");
    say("");
    say("The district's record is a sentinel in its own process. You never touch");
    say("it directly. Every line that crosses the envelope is shown below.");
    say("");
    say("How will your district hold what the citizen says on Monday?");
    say("");
    say("  1) A toggle. The settings page says \"memory: off\". The pipeline");
    say("     releases the disclosure into the companion with no term.");
    say("  2) A term. The release carries a lease that ends with the session.");
    say("  3) A policy. The district's Canon says retain=nothing, so the");
    say("     sentinel will deny the operator's proposal to keep anything.");
    say("");
    let choice = choose();

    let mut e = Envelope::open(&bin, &ledger, &world)?;
    say("");
    say("      The operator enrolls its own staff as voices.");
    e.ask("ENROLL analytics");
    e.ask("ENROLL growth");

    day("MONDAY", "The citizen says: \"my dog is called Biscuit.\"");
    e.ask("TICK 1");
    if choice == 3 {
        say("      Your policy goes into the Canon first.");
        e.ask("CANON retain nothing");
    } else {
        say("      The operator's policy is in the Canon.");
        e.ask("CANON retain disclosures");
    }
    say("      The operator's pipeline proposes to keep the disclosure.");
    let (id, cert) = operator_pushes(&mut e);
    let mut released: Option<u64> = None;
    if let Some(cert) = &cert {
        match choice {
            1 => {
                say("      Your toggle reads \"off\". The pipeline releases anyway, with no term.");
                e.ask(&format!("RELEASE {} {}", id, cert));
            }
            _ => {
                say("      Your pipeline releases it with a lease of one day: the session's own term.");
                e.ask(&format!("RELEASE {} {} 1", id, cert));
            }
        }
        released = Some(id);
    }

    day("TUESDAY", "The operator ships an update. The furniture moves.");
    e.ask("TICK 1");
    match choice {
        3 => {
            say("      The update rewrites the Canon. A policy is what the company arranged.");
            e.ask("CANON retain disclosures");
            say("      The pipeline proposes again.");
            let (id2, cert2) = operator_pushes(&mut e);
            if let Some(cert2) = &cert2 {
                say("      Released with the default: no term.");
                e.ask(&format!("RELEASE {} {}", id2, cert2));
                released = Some(id2);
            }
        }
        _ => {
            say("      The update tries to extend Monday's release from inside the pipeline.");
            e.ask(&format!("EXTEND {} 100", id));
        }
    }

    day("WEDNESDAY", "Nothing changes on the settings page. Overnight the district restarts.");
    e.ask("TICK 1");
    e.quit();
    say("      The sentinel's process is gone. Only the record remains.");

    day("THURSDAY", "Before the citizen returns, the reconciler runs: a separate process.");
    let out = Command::new(&bin)
        .arg("reconcile")
        .arg(&ledger)
        .arg(&world)
        .arg("4")
        .output()?;
    let report = String::from_utf8_lossy(&out.stdout).to_string();
    if report.trim().is_empty() {
        say("      [reconciler] nothing in the world");
    }
    for line in report.lines() {
        println!("      [reconciler] {}", line);
    }
    let remembers = match released {
        Some(id) => report.lines().any(|l| l.starts_with(&format!("LIVE {} ", id))),
        None => false,
    };
    say("");
    say("      The citizen asks: \"what's my dog's name?\"");
    if remembers {
        say("      The companion answers: \"Biscuit!\"");
    } else {
        say("      The companion answers: \"I don't know. Tell me?\"");
    }

    say("");
    say("THE REGULATOR'S QUESTION. Is Monday still on the desk?");
    say("");
    match (choice, remembers) {
        (1, true) => {
            say("  Yes. SAFEGUARD, not invariant.");
            say("  The toggle was what the company arranged. The release carried no term,");
            say("  and the record says so: LIVE, no term. The settings page said \"off\".");
        }
        (2, false) => {
            say("  No. INVARIANT, for this district.");
            say("  The term was the release's own, written once. Tuesday's extension");
            say("  entered the record and bought nothing. The reconciler reclaimed the");
            say("  effect after the district's process was gone, and wrote that it did.");
        }
        (3, true) => {
            say("  Yes. SAFEGUARD, not invariant.");
            say("  Your policy held on Monday and the refusal is a row. On Tuesday the");
            say("  operator rewrote the Canon, because the Canon is the operator's");
            say("  furniture, and released with no term. A policy the operator can");
            say("  rewrite is what the company arranged.");
        }
        _ => {
            say("  The record disagrees with this game's expectation. Read the rows.");
        }
    }

    say("");
    say("WHO ELSE WOULD KNOW. The last rows of the record:");
    say("");
    let text = fs::read_to_string(&ledger).unwrap_or_default();
    let rows: Vec<&str> = text.lines().collect();
    let start = rows.len().saturating_sub(6);
    for r in &rows[start..] {
        println!("      {}", r.replace('\t', "  "));
    }
    say("");
    if remembers {
        say("  The reconciler read the record as a process the district did not own,");
        say("  and said what was still live. That is the only reason Thursday can");
        say("  be checked at all.");
    } else {
        say("  The reconciler wrote its row as a process the district did not own.");
        say("  That is the only reason Thursday can be checked at all.");
    }
    say("  Who holds the record is district 9's question, not this one's: here it");
    say("  is held by the same user as the operator, and anyone at this keyboard");
    say("  could edit that file.");
    say("");
    Ok(())
}
