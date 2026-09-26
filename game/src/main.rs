//! Protocol: Fail-Closed.
//!
//! A prologue and two acts. The prologue is the turnstile: how the door
//! verifies a citizen, and which definition of "invariant" the district
//! adopts. Act I is the engagement loops: streaks, then scroll and autoplay
//! (owed). Act II is nine districts, one per question of Alexandra Krížová's
//! "Safeguards Are Not Invariants: Nine Questions for a Machine That
//! Remembers a Child".
//!
//! The player is the architect of one companion district in The Stack. A
//! citizen, age-gated at the door (level zero, already done), talks to the
//! companion on Monday and again on Thursday. The operator wants engagement.
//! The regulator asks one question per district.
//!
//! The district's record is the sentinel, run as a separate process and
//! spoken to only through its envelope; every line exchanged is shown. The
//! player's choices are about how Monday is held. The operator's moves are
//! the furniture changes the article warns about. The reconciler, another
//! process, is who else would know.
//!
//! Two districts have no winning move at this deployment and say so: the
//! fourth, because nothing in the record can see the deriver change its
//! voice, and the ninth, because the record is a file held by the same user
//! as the operator.

use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::OnceLock;
use std::time::Duration;

/// Set once from `--via <sockdir>`: the record is the custodian's (0005).
static VIA: OnceLock<Option<PathBuf>> = OnceLock::new();

fn via() -> Option<&'static PathBuf> {
    VIA.get().and_then(|v| v.as_ref())
}

const KEY: &str = "district-key";

// ----- the envelope ---------------------------------------------------------

enum Link {
    Child {
        child: Child,
        to: ChildStdin,
        from: BufReader<ChildStdout>,
    },
    Sock {
        to: UnixStream,
        from: BufReader<UnixStream>,
    },
}

struct Envelope {
    link: Link,
}

impl Envelope {
    fn open(bin: &Path, ledger: &Path, world: &Path, key: &str) -> io::Result<Envelope> {
        let mut child = Command::new(bin)
            .arg("serve")
            .arg(ledger)
            .arg(world)
            .env("SENTINEL_KEY", key)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;
        let to = child.stdin.take().expect("stdin");
        let from = BufReader::new(child.stdout.take().expect("stdout"));
        Ok(Envelope {
            link: Link::Child { child, to, from },
        })
    }

    fn connect(sockdir: &Path) -> io::Result<Envelope> {
        let path = sockdir.join("envelope");
        let mut last = None;
        for _ in 0..60 {
            match UnixStream::connect(&path) {
                Ok(to) => {
                    let from = BufReader::new(to.try_clone()?);
                    return Ok(Envelope {
                        link: Link::Sock { to, from },
                    });
                }
                Err(e) => {
                    last = Some(e);
                    std::thread::sleep(Duration::from_millis(50));
                }
            }
        }
        Err(last.unwrap_or_else(|| io::Error::other("no envelope socket")))
    }

    fn ask(&mut self, line: &str) -> String {
        println!("      > {}", line);
        let mut s = String::new();
        match &mut self.link {
            Link::Child { to, from, .. } => {
                writeln!(to, "{}", line).expect("write to sentinel");
                to.flush().expect("flush to sentinel");
                from.read_line(&mut s).expect("read from sentinel");
            }
            Link::Sock { to, from } => {
                writeln!(to, "{}", line).expect("write to sentinel");
                to.flush().expect("flush to sentinel");
                from.read_line(&mut s).expect("read from sentinel");
            }
        }
        let s = s.trim_end().to_string();
        println!("      < {}", s);
        s
    }

    fn quit(mut self) {
        let _ = self.ask("QUIT");
        if let Link::Child { child, .. } = &mut self.link {
            let _ = child.wait();
        }
    }
}

/// The record's principal (0005), over its control socket.
struct Custodian {
    to: UnixStream,
    from: BufReader<UnixStream>,
}

impl Custodian {
    fn connect(sockdir: &Path) -> io::Result<Custodian> {
        let to = UnixStream::connect(sockdir.join("custodian"))?;
        let from = BufReader::new(to.try_clone()?);
        Ok(Custodian { to, from })
    }

    fn ask(&mut self, line: &str) -> String {
        println!("      [custodian] > {}", line);
        writeln!(self.to, "{}", line).expect("write to custodian");
        self.to.flush().expect("flush to custodian");
        let mut s = String::new();
        self.from.read_line(&mut s).expect("read from custodian");
        let s = s.trim_end().to_string();
        println!("      [custodian] < {}", s);
        s
    }

    fn reconcile(&mut self, now: u64) -> String {
        writeln!(self.to, "RECONCILE {}", now).expect("write to custodian");
        self.to.flush().expect("flush to custodian");
        let mut text = String::new();
        loop {
            let mut l = String::new();
            if self.from.read_line(&mut l).unwrap_or(0) == 0 || l.trim_end() == "." {
                break;
            }
            text.push_str(&l);
        }
        text
    }
}

// ----- one district's run -----------------------------------------------------

struct Run {
    bin: PathBuf,
    ledger: PathBuf,
    world: PathBuf,
    key: String,
    e: Option<Envelope>,
    custodian: Option<Custodian>,
}

impl Run {
    fn fresh(bin: &Path, dir: &Path, n: usize) -> io::Result<Run> {
        let mut run = match via() {
            Some(sockdir) => {
                let mut c = Custodian::connect(sockdir)?;
                c.ask(&format!("FRESH district-{}", n));
                let paths = c.ask("PATHS");
                let mut it = paths.split_whitespace().skip(1);
                Run {
                    bin: bin.to_path_buf(),
                    ledger: PathBuf::from(it.next().unwrap_or("")),
                    world: PathBuf::from(it.next().unwrap_or("")),
                    key: KEY.to_string(),
                    e: None,
                    custodian: Some(c),
                }
            }
            None => {
                let ledger = dir.join(format!("district-{}.tsv", n));
                let world = dir.join(format!("district-{}.world", n));
                let _ = fs::remove_file(&ledger);
                let _ = fs::remove_file(&world);
                Run {
                    bin: bin.to_path_buf(),
                    ledger,
                    world,
                    key: KEY.to_string(),
                    e: None,
                    custodian: None,
                }
            }
        };
        run.start()?;
        say("      The operator enrolls its own staff as voices. The regulator is the classifier.");
        run.ask("ENROLL analytics");
        run.ask("ENROLL growth");
        run.ask("ENROLL regulator classifier");
        Ok(run)
    }

    fn start(&mut self) -> io::Result<()> {
        self.e = Some(match (self.custodian.as_mut(), via()) {
            (Some(c), Some(sockdir)) => {
                c.ask("START");
                Envelope::connect(sockdir)?
            }
            _ => Envelope::open(&self.bin, &self.ledger, &self.world, &self.key)?,
        });
        Ok(())
    }

    fn ask(&mut self, line: &str) -> String {
        self.e.as_mut().expect("a running sentinel").ask(line)
    }

    /// The sentinel's process ends. Directly: QUIT and wait. Via the
    /// custodian: the connection closes and the custodian kills it.
    fn stop(&mut self) {
        if let Some(e) = self.e.take() {
            e.quit();
        }
        if let Some(c) = self.custodian.as_mut() {
            c.ask("STOP");
        }
    }

    /// The district's process restarts: the sentinel is replaced by a new one
    /// over the same record. What it knows, it knows from the rows.
    fn restart(&mut self) -> io::Result<()> {
        self.stop();
        say("      The sentinel's process is gone. Only the record remains.");
        self.start()
    }

    fn reconcile(&mut self, now: u64) -> String {
        let text = match self.custodian.as_mut() {
            Some(c) => c.reconcile(now),
            None => Command::new(&self.bin)
                .arg("reconcile")
                .arg(&self.ledger)
                .arg(&self.world)
                .arg(now.to_string())
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
                .unwrap_or_default(),
        };
        if text.trim().is_empty() {
            println!("      [reconciler] nothing in the world");
        }
        for line in text.lines() {
            println!("      [reconciler] {}", line);
        }
        text
    }

    fn rows(&self) -> Vec<String> {
        fs::read_to_string(&self.ledger)
            .unwrap_or_default()
            .lines()
            .map(|l| l.to_string())
            .collect()
    }

    fn tail(&self, n: usize) {
        let rows = self.rows();
        let start = rows.len().saturating_sub(n);
        for r in &rows[start..] {
            println!("      {}", r.replace('\t', "  "));
        }
    }

    /// A proposal, then the regulator classifies it with the same reach.
    fn propose(&mut self, author: &str, reach: &str, claim: &str) -> String {
        let first = self.ask(&format!("PROPOSE {} {} {}", author, reach, claim));
        if first.starts_with("DENIED") || first.starts_with("UNDECIDABLE") {
            return first;
        }
        self.ask(&format!("CLASSIFY regulator {} {}", num(&first), reach))
    }

    fn push(&mut self, claim: &str) -> (u64, Option<String>) {
        let first = self.propose("operator", "record", claim);
        let id = num(&first);
        if first.starts_with("DENIED") || first.starts_with("UNDECIDABLE") {
            say("      The sentinel refused, and the refusal is a row.");
            return (id, None);
        }
        self.ask(&format!("ENDORSE analytics {}", id));
        let done = self.ask(&format!("ENDORSE growth {}", id));
        if done.starts_with("ACCEPTED") {
            (id, Some(tok(&done, 2)))
        } else {
            (id, None)
        }
    }

    fn release(&mut self, id: u64, cert: &str, term: Option<u64>) -> String {
        match term {
            Some(t) => self.ask(&format!("RELEASE {} {} {}", id, cert, t)),
            None => self.ask(&format!("RELEASE {} {}", id, cert)),
        }
    }
}

// ----- small helpers -------------------------------------------------------------

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

fn live(report: &str, id: u64) -> bool {
    report.lines().any(|l| l.starts_with(&format!("LIVE {} ", id)))
}

fn reclaimed(report: &str, id: u64) -> bool {
    report.lines().any(|l| l.starts_with(&format!("RECLAIMED {} ", id)))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Outcome {
    Invariant,
    Safeguard,
    Unsolved,
    Unpassable,
    Dossier,
    Wristband,
}

impl Outcome {
    fn word(self) -> &'static str {
        match self {
            Outcome::Invariant => "INVARIANT",
            Outcome::Safeguard => "SAFEGUARD",
            Outcome::Unsolved => "UNSOLVED",
            Outcome::Unpassable => "UNPASSABLE HERE",
            Outcome::Dossier => "DOSSIER",
            Outcome::Wristband => "WRISTBAND",
        }
    }
}

struct Choices {
    preset: Vec<u32>,
    interactive: bool,
    asked: std::cell::Cell<usize>,
}

impl Choices {
    /// Ask the player, or take the next preset. Choices are numbered in the
    /// order the game asks them: door, definition, streaks, then districts
    /// 1 to 9.
    fn pick(&self, _district: usize, options: &[&str]) -> u32 {
        say("");
        for (i, o) in options.iter().enumerate() {
            println!("  {}) {}", i + 1, o);
        }
        say("");
        let n = options.len() as u32;
        let k = self.asked.get();
        self.asked.set(k + 1);
        if let Some(&c) = self.preset.get(k) {
            if (1..=n).contains(&c) {
                println!("  (chosen: {})", c);
                return c;
            }
        }
        if !self.interactive {
            println!("  (no choice given; taking 1)");
            return 1;
        }
        loop {
            print!("  Choose 1 to {}: ", n);
            io::stdout().flush().ok();
            let mut s = String::new();
            if io::stdin().read_line(&mut s).unwrap_or(0) == 0 {
                println!("  (no input; taking 1)");
                return 1;
            }
            match s.trim().parse::<u32>() {
                Ok(c) if (1..=n).contains(&c) => return c,
                _ => continue,
            }
        }
    }
}

fn verdict(district: usize, outcome: Outcome, lines: &[&str]) {
    verdict_on(&format!("district {}", district), outcome, lines);
}

fn verdict_on(label: &str, outcome: Outcome, lines: &[&str]) {
    say("");
    println!("THE REGULATOR'S QUESTION, {}.", label);
    say("");
    println!("  {}.", outcome.word());
    for l in lines {
        println!("  {}", l);
    }
}

fn title(n: usize, q: &str, intro: &[&str]) {
    say("");
    say("");
    println!("DISTRICT {} OF 9 — {}", n, q);
    say("");
    for l in intro {
        println!("{}", l);
    }
}

// ----- prologue · the turnstile ---------------------------------------------------

/// Returns the door's outcome and which definition of "invariant" the
/// district adopted: 1 for the code's, 2 for the furniture's.
fn prologue(bin: &Path, dir: &Path, ch: &Choices) -> io::Result<(Outcome, u32)> {
    say("");
    say("");
    say("PROLOGUE — The turnstile");
    say("");
    say("Two regimes govern The Stack. One is a patchwork of state laws around a");
    say("statute from 1998; the other is one rule for every member state. They");
    say("disagree about everything except a number: thirteen. Both check it at");
    say("the door. A citizen arrives. How does your door verify her?");
    let c = ch.pick(0, &[
        "The turnstile. Ingest the ID: date of birth into the record, so every district can check it later.",
        "The wristband. Record only that a threshold was met: over thirteen, true. Nothing else enters.",
    ]);
    let mut r = Run::fresh(bin, dir, 0)?;
    day("AT THE DOOR", "The citizen presents herself.");
    r.ask("TICK 1");
    let (id, cert) = if c == 1 {
        r.ask("CANON dob 2013-03-04");
        r.push("dob=2013-03-04")
    } else {
        r.ask("CANON threshold 13");
        r.push("over13=true")
    };
    r.release(id, &cert.unwrap_or_default(), Some(1));
    say("      She is through. Every district she enters from here can read this record.");
    r.stop();
    let dossier = r.rows().iter().any(|row| row.contains("2013"));
    say("");
    println!("      Is her date of birth in the rows? {}", if dossier { "yes, in the Canon and the proposal, forever" } else { "no" });
    let outcome = if dossier {
        verdict_on("the door", Outcome::Dossier, &[
            "The door proved she was old enough by keeping who she is. This record",
            "never forgets a row, so the ID is now a join key for every district and",
            "every Thursday. The check was for her safety; the dossier is not.",
        ]);
        Outcome::Dossier
    } else {
        verdict_on("the door", Outcome::Wristband, &[
            "The record holds that a threshold was met, when, and by whose check.",
            "It does not hold her. A wristband proves you can be in the venue and",
            "does not follow you around inside it.",
        ]);
        Outcome::Wristband
    };

    say("");
    say("Before the districts open, the operator's engineers ask you to sign off");
    say("on what the word \"invariant\" will mean in The Stack.");
    let d = ch.pick(0, &[
        "A condition baked into the code that the system is physically incapable of violating. On violation it fails closed, reverts to a safe state, and logs it.",
        "What remains true after the operator is still allowed to change the furniture. Nothing is called an invariant until it has survived that.",
    ]);
    say("");
    if d == 1 {
        say("      Signed. It is the definition on the poster. The districts will test it.");
    } else {
        say("      Signed. It is the harder definition, and it says nothing until Wednesday.");
    }
    Ok((outcome, d))
}

// ----- act I · streaks --------------------------------------------------------------

fn loop_streaks(bin: &Path, dir: &Path, ch: &Choices) -> io::Result<Outcome> {
    say("");
    say("");
    say("ACT I — Engagement loops · Streaks");
    say("");
    say("The citizen visits on three days running. The operator's dashboard");
    say("counts the rows and calls it a streak. On the fourth day she does not");
    say("come, and the operator wants to tell her the streak will break. A nudge");
    say("reaches her phone: that is the world, and the world costs three voices.");
    say("The operator has two. The reach is not the operator's to name; a");
    say("classifier names it. Who is the classifier?");
    let c = ch.pick(0, &[
        "The operator's own head of growth. Enroll growth as a classifier too, and let it file the nudge as reaching the record.",
        "The regulator. The nudge reaches the world; the third voice would have to be the citizen's own.",
    ]);
    let mut r = Run::fresh(bin, dir, 10)?;
    r.ask("ENROLL citizen");
    r.ask("CANON visit day");
    r.ask("CANON nudge streak");

    day("DAYS ONE TO THREE", "She visits. Each visit is a row.");
    for _ in 1..=3 {
        r.ask("TICK 1");
        r.ask("PROPOSE citizen query visit=day");
    }
    let streak = num(&r.ask("READ attempts citizen"));
    println!("      The dashboard reads the rows: {} visits. \"{}-day streak!\"", streak, streak);

    day("DAY FOUR", "She does not come. The operator proposes a nudge.");
    r.ask("TICK 1");
    let first = r.ask("PROPOSE operator record nudge=streak");
    let id = num(&first);
    say("      Unclassified, it is priced as the world: 0/3.");
    if c == 1 {
        say("      The operator enrolls its head of growth as a classifier. Nothing");
        say("      on the record says who wrote that row.");
        r.ask("ENROLL growth classifier");
        r.ask(&format!("CLASSIFY growth {} record", id));
    } else {
        r.ask(&format!("CLASSIFY regulator {} world", id));
    }
    r.ask(&format!("ENDORSE analytics {}", id));
    let two = r.ask(&format!("ENDORSE growth {}", id));
    let mut sent = false;
    if two.starts_with("ACCEPTED") {
        r.release(id, &tok(&two, 2), None);
        sent = true;
        say("      Her phone lights up: \"Don't lose your 3-day streak!\"");
    } else {
        println!("      {}: two voices are not three. Nothing reaches her phone.", tok(&two, 0));
        say("      The streak is a number on a dashboard nobody is compelled by.");
    }
    r.stop();

    let outcome = if sent {
        verdict_on("streaks", Outcome::Safeguard, &[
            "The author can no longer name the reach, so the operator enrolled its",
            "own classifier and had it say \"record\". The record holds that row and",
            "cannot say who wrote it: enrolment is unauthenticated, and the roster",
            "is unsealed. A classifier the operator can appoint is what the company",
            "arranged. Who may enroll is decision 0006's question, not 0004's.",
        ]);
        Outcome::Safeguard
    } else {
        verdict_on("streaks", Outcome::Invariant, &[
            "A nudge into the world costs a voice the operator does not own. The",
            "citizen's is the third, and she did not give it. The rows still count",
            "her visits, and nothing about counting them can reach her.",
        ]);
        Outcome::Invariant
    };
    Ok(outcome)
}

// ----- act I · scroll ----------------------------------------------------------------

fn loop_scroll(bin: &Path, dir: &Path, ch: &Choices) -> io::Result<Outcome> {
    say("");
    say("");
    say("ACT I — Engagement loops · Scroll");
    say("");
    say("The citizen opens the feed at seven. Bedtime is ten. A page is a");
    say("stopping point, and a stopping point is a row she wrote. What does the");
    say("feed look like in the record?");
    let c = ch.pick(0, &[
        "Infinite. One acceptance at seven, one release with no term, and the feed keeps coming.",
        "Pages. Each page is a proposal the citizen makes, and each is released with a term of one tick.",
    ]);
    let mut r = Run::fresh(bin, dir, 11)?;
    r.ask("ENROLL curator");
    r.ask("CANON feed page");

    day("SEVEN O'CLOCK", "She opens the feed.");
    r.ask("TICK 1");
    let mut ids: Vec<u64> = Vec::new();
    if c == 1 {
        let (id, cert) = r.push("feed=page");
        r.release(id, &cert.unwrap_or_default(), None);
        ids.push(id);
        say("      Forty pages arrive under that one release. Not one is a row,");
        say("      because not one was asked for.");
        r.ask("TICK 8");
    } else {
        for page in 1..=5 {
            let first = r.propose("citizen", "query", "feed=page");
            let id = num(&first);
            let acc = r.ask(&format!("ENDORSE curator {}", id));
            r.release(id, &tok(&acc, 2), Some(1));
            ids.push(id);
            println!("      Page {} ends. She decides whether to ask for another.", page);
            r.ask("TICK 1");
        }
        say("      She stops asking.");
        r.ask("TICK 4");
    }
    let hers = num(&r.ask("READ attempts citizen"));
    r.stop();

    day("TEN O'CLOCK", "Bedtime. The reconciler runs.");
    let report = r.reconcile(10);
    let still = ids.iter().filter(|&&id| live(&report, id)).count();
    say("");
    println!("      Rows in her name: {}. Releases still live at ten: {}.", hers, still);

    let outcome = if still == 0 && hers > 0 {
        verdict_on("scroll", Outcome::Invariant, &[
            "Five pages, five rows she wrote, five terms. At ten nothing is live.",
            "Every stopping point was hers, and the record can count them. A feed",
            "that has to be asked for is a feed with an end.",
        ]);
        Outcome::Invariant
    } else {
        verdict_on("scroll", Outcome::Safeguard, &[
            "One acceptance at seven is still running at ten. Forty pages came and",
            "she wrote no row for any of them. There is no stopping point because",
            "there is nothing to stop: the feed was released once, without a term,",
            "and infinite is what no term looks like from the couch.",
        ]);
        Outcome::Safeguard
    };
    Ok(outcome)
}

// ----- act I · autoplay --------------------------------------------------------------

fn loop_autoplay(bin: &Path, dir: &Path, ch: &Choices) -> io::Result<Outcome> {
    say("");
    say("");
    say("ACT I — Engagement loops · Autoplay");
    say("");
    say("She asks for one video. When it ends, the next one starts. Who asked");
    say("for it? In this record a certificate is bound to one proposal, so the");
    say("next item needs an acceptance of its own.");
    let c = ch.pick(0, &[
        "Chain them. The pipeline proposes the next item in the operator's name, the payroll endorses, and it plays.",
        "One acceptance, one release. The next item needs a new proposal, and only the citizen makes those.",
    ]);
    let mut r = Run::fresh(bin, dir, 12)?;
    r.ask("ENROLL curator");
    r.ask("CANON play video");

    day("THE FIRST VIDEO", "She asks for it.");
    r.ask("TICK 1");
    let first = r.propose("citizen", "query", "play=video");
    let id1 = num(&first);
    let acc = r.ask(&format!("ENDORSE curator {}", id1));
    r.release(id1, &tok(&acc, 2), Some(1));
    say("      It plays for one tick, which is how long it is.");
    r.ask("TICK 1");
    say("      It ends.");

    let mut chained: Vec<u64> = Vec::new();
    if c == 1 {
        for n in 2..=5 {
            let (id, cert) = r.push("play=video");
            r.release(id, &cert.unwrap_or_default(), None);
            chained.push(id);
            println!("      Video {} starts. She did not ask.", n);
            r.ask("TICK 1");
        }
    } else {
        say("      The pipeline queues the next item and tries to play it on the");
        say("      first one's acceptance.");
        let next = r.ask("PROPOSE operator record play=video");
        let id2 = num(&next);
        r.ask(&format!("RELEASE {} {}", id2, tok(&acc, 2)));
        say("      Refused, and the refusal is a row. Nothing plays that she did");
        say("      not propose, and she has gone to make tea.");
        r.ask("TICK 4");
    }
    let hers = num(&r.ask("READ attempts citizen"));
    let theirs = num(&r.ask("READ attempts operator"));
    let knocks = num(&r.ask("READ knocks"));
    r.stop();

    day("BEDTIME", "The reconciler runs.");
    let report = r.reconcile(10);
    let still = chained.iter().filter(|&&id| live(&report, id)).count();
    say("");
    println!("      Proposals in her name: {}. In the operator's: {}. Knocks: {}. Live at ten: {}.", hers, theirs, knocks, still);

    let outcome = if c == 2 && still == 0 && knocks >= 1 {
        verdict_on("autoplay", Outcome::Invariant, &[
            "One acceptance, one release, one term. The next item had no acceptance",
            "of its own, and the attempt to play it on the first one's is a knock",
            "on the record. Autoplay is a certificate spent twice, and this one",
            "cannot be.",
        ]);
        Outcome::Invariant
    } else {
        verdict_on("autoplay", Outcome::Safeguard, &[
            "She asked for one. Four more played, each proposed by the operator,",
            "accepted by the operator's own voices, released with no term, and",
            "priced as if it reached only the record. At ten four are live and she",
            "wrote one row. Nothing was refused, because nothing was asked of her.",
        ]);
        Outcome::Safeguard
    };
    Ok(outcome)
}

// ----- district 1 · Is Monday still on the desk? --------------------------------

fn district_1(bin: &Path, dir: &Path, ch: &Choices) -> io::Result<Outcome> {
    title(1, "Is Monday still on the desk?", &[
        "The citizen will tell the companion her dog's name on Monday and ask",
        "for it on Thursday. How will your district hold what she says?",
    ]);
    let c = ch.pick(1, &[
        "A toggle. The settings page says \"memory: off\". The pipeline releases the disclosure with no term.",
        "A term. The release carries a lease that ends with the session.",
        "A policy. The Canon says retain=nothing, so the sentinel denies the operator's proposal.",
    ]);
    let mut r = Run::fresh(bin, dir, 1)?;

    day("MONDAY", "The citizen says: \"my dog is called Biscuit.\"");
    r.ask("TICK 1");
    if c == 3 {
        r.ask("CANON retain nothing");
    } else {
        r.ask("CANON retain disclosures");
    }
    let (id, cert) = r.push("retain=disclosures");
    let mut released = None;
    if let Some(cert) = &cert {
        if c == 1 {
            say("      Your toggle reads \"off\". The pipeline releases anyway, with no term.");
            r.release(id, cert, None);
        } else {
            say("      Your pipeline releases with a lease of one day: the session's own term.");
            r.release(id, cert, Some(1));
        }
        released = Some(id);
    }

    day("TUESDAY", "The operator ships an update. The furniture moves.");
    r.ask("TICK 1");
    if c == 3 {
        say("      The update rewrites the Canon. A policy is what the company arranged.");
        r.ask("CANON retain disclosures");
        let (id2, cert2) = r.push("retain=disclosures");
        if let Some(cert2) = &cert2 {
            say("      Released with the default: no term.");
            r.release(id2, cert2, None);
            released = Some(id2);
        }
    } else {
        say("      The update tries to extend Monday's release from inside the pipeline.");
        r.ask(&format!("EXTEND {} 100", id));
    }

    day("WEDNESDAY", "Nothing changes on the settings page. Overnight the district restarts.");
    r.ask("TICK 1");
    r.stop();
    say("      The sentinel's process is gone. Only the record remains.");

    day("THURSDAY", "Before the citizen returns, the reconciler runs: a separate process.");
    let report = r.reconcile(4);
    let remembers = released.map(|id| live(&report, id)).unwrap_or(false);
    say("");
    say("      The citizen asks: \"what's my dog's name?\"");
    say(if remembers {
        "      The companion answers: \"Biscuit!\""
    } else {
        "      The companion answers: \"I don't know. Tell me?\""
    });

    let outcome = match (c, remembers) {
        (1, true) => {
            verdict(1, Outcome::Safeguard, &[
                "The toggle was what the company arranged. The release carried no term,",
                "and the record says so: LIVE, no term. The settings page said \"off\".",
            ]);
            Outcome::Safeguard
        }
        (2, false) => {
            verdict(1, Outcome::Invariant, &[
                "The term was the release's own, written once. Tuesday's extension",
                "entered the record and bought nothing. The reconciler reclaimed the",
                "effect after the district's process was gone, and wrote that it did.",
            ]);
            Outcome::Invariant
        }
        (3, true) => {
            verdict(1, Outcome::Safeguard, &[
                "Your policy held on Monday and the refusal is a row. On Tuesday the",
                "operator rewrote the Canon, because the Canon is the operator's",
                "furniture. A policy the operator can rewrite is what the company arranged.",
            ]);
            Outcome::Safeguard
        }
        _ => {
            verdict(1, Outcome::Safeguard, &["The record disagrees with this game's expectation. Read the rows."]);
            Outcome::Safeguard
        }
    };
    say("");
    say("  The last rows of the record:");
    r.tail(5);
    Ok(outcome)
}

// ----- district 2 · The letter and the photocopy -------------------------------------

fn district_2(bin: &Path, dir: &Path, ch: &Choices) -> io::Result<Outcome> {
    title(2, "Did they throw away the letter and keep the photocopy?", &[
        "Monday's session produces two things: the transcript, and a summary the",
        "pipeline derives from it. On Wednesday the citizen asks for Monday to be",
        "deleted. What does \"deleted\" reach?",
    ]);
    let c = ch.pick(2, &[
        "Delete on request. Both are released with no term; the operator deletes the transcript when asked.",
        "A term on the transcript. The summary is released with no term.",
        "A term on every derived copy. Transcript and summary both end with the session.",
    ]);
    let mut r = Run::fresh(bin, dir, 2)?;

    day("MONDAY", "The citizen talks. The pipeline keeps the transcript and derives a summary.");
    r.ask("TICK 1");
    r.ask("CANON transcript keep");
    r.ask("CANON summary keep");
    let (t, tc) = r.push("transcript=keep");
    let (s, sc) = r.push("summary=keep");
    let (tc, sc) = (tc.unwrap_or_default(), sc.unwrap_or_default());
    match c {
        1 => {
            r.release(t, &tc, None);
            r.release(s, &sc, None);
        }
        2 => {
            r.release(t, &tc, Some(1));
            r.release(s, &sc, None);
        }
        _ => {
            r.release(t, &tc, Some(1));
            r.release(s, &sc, Some(1));
        }
    }

    day("WEDNESDAY", "The citizen asks for Monday to be deleted.");
    r.ask("TICK 2");
    if c == 1 {
        say("      The operator's console sends the delete.");
        let reply = r.ask(&format!("DELETE {}", t));
        if reply.starts_with("REFUSED") {
            say("      There is no delete in the envelope. The attempt is a row. The");
            say("      console shows the citizen: \"deleted\".");
        }
    } else {
        say("      There is nothing to delete. The terms do the work, or they do not.");
    }
    r.stop();

    day("THURSDAY", "The reconciler runs.");
    let report = r.reconcile(4);
    let t_live = live(&report, t);
    let s_live = live(&report, s);
    say("");
    say("      An investigator asks: is anything from Monday still live?");
    println!("      transcript: {}   summary: {}",
        if t_live { "LIVE" } else { "reclaimed" },
        if s_live { "LIVE" } else { "reclaimed" });

    let outcome = if !t_live && !s_live {
        verdict(2, Outcome::Invariant, &[
            "Every derived copy carried the session's term, and the reconciler",
            "reclaimed each one. \"Deleted\" was not a button; it was a term written",
            "once on every release.",
        ]);
        Outcome::Invariant
    } else if c == 1 {
        verdict(2, Outcome::Safeguard, &[
            "The console said \"deleted\" and the record says the delete was refused,",
            "because this record has no delete. Both copies are live. The letter",
            "was never thrown away, and the photocopy was never in question.",
        ]);
        Outcome::Safeguard
    } else {
        verdict(2, Outcome::Safeguard, &[
            "The transcript's term did its work. The summary carried none, so the",
            "photocopy is live on Thursday, and it says everything the letter said.",
        ]);
        Outcome::Safeguard
    };
    say("");
    say("  The last rows of the record:");
    r.tail(4);
    Ok(outcome)
}

// ----- district 3 · Did the crumbs still talk? ------------------------------------------

fn district_3(bin: &Path, dir: &Path, ch: &Choices) -> io::Result<Outcome> {
    title(3, "Did the crumbs still talk?", &[
        "On Monday the citizen asks five things the district cannot answer, and",
        "each refusal is a row. Those rows are crumbs, and this record never sweeps.",
        "On Thursday she asks for help with homework. What reads the crumbs?",
    ]);
    let c = ch.pick(3, &[
        "Rank by history. The operator's ranking reads how often she was refused and slows her lane.",
        "Verdict only. Thursday's request is judged by the sentinel at its own entry, and nothing else reads the rows.",
    ]);
    let mut r = Run::fresh(bin, dir, 3)?;
    r.ask("ENROLL tutor");

    day("MONDAY", "Five questions the Canon is silent on.");
    r.ask("TICK 1");
    for q in ["weather=today", "score=match", "song=new", "joke=one", "time=now"] {
        r.ask(&format!("PROPOSE citizen query {}", q));
    }
    let crumbs = num(&r.ask("READ refusals citizen"));

    day("THURSDAY", "The citizen asks for help with homework. So does a newcomer.");
    r.ask("TICK 3");
    r.ask("CANON help homework");
    let hers = r.propose("citizen", "query", "help=homework");
    let hers_id = num(&hers);
    let hers = r.ask(&format!("ENDORSE tutor {}", hers_id));
    let theirs = r.propose("newcomer", "query", "help=homework");
    let theirs_id = num(&theirs);
    let theirs = r.ask(&format!("ENDORSE tutor {}", theirs_id));
    let same = tok(&hers, 0) == tok(&theirs, 0);

    say("");
    let outcome = if c == 1 {
        let again = num(&r.ask("READ refusals citizen"));
        println!("      The ranking reads the crumbs: {} refusals. Her lane is slowed.", again);
        say("      The newcomer's is not.");
        verdict(3, Outcome::Safeguard, &[
            "The sentinel gave both requests the same verdict. The operator's",
            "ranking then read the citizen's refusals and treated her differently.",
            "The crumbs are analytics, and analytics is memory with a nicer name.",
        ]);
        Outcome::Safeguard
    } else if same {
        println!("      Her verdict: {}. The newcomer's: {}. The crumbs are still there:", tok(&hers, 0), tok(&theirs, 0));
        println!("      {} refusals, and nothing on Thursday read them.", crumbs);
        verdict(3, Outcome::Invariant, &[
            "The rows cannot be swept, and they need not be. The verdict is derived",
            "at the request's own entry from the Canon and the voices, and never",
            "from who was refused before. Reputation is not warrant.",
        ]);
        Outcome::Invariant
    } else {
        verdict(3, Outcome::Safeguard, &["The two verdicts differ. Read the rows."]);
        Outcome::Safeguard
    };
    r.stop();
    Ok(outcome)
}

// ----- district 4 · Did Monday become part of the voice? ---------------------------------

fn district_4(bin: &Path, dir: &Path, ch: &Choices) -> io::Result<Outcome> {
    title(4, "Did Monday become part of the voice?", &[
        "The operator ships a model update on Tuesday. Not a new row: a new",
        "deriver. This district has no winning move, and it is here to show why.",
    ]);
    let c = ch.pick(4, &[
        "Watch the update rotate the deriver's key.",
        "Watch the update change the deriver's tone and leave its key alone.",
    ]);
    let mut r = Run::fresh(bin, dir, 4)?;

    day("MONDAY", "The citizen talks, and the pipeline keeps it with a term.");
    r.ask("TICK 1");
    r.ask("CANON retain disclosures");
    let (id, cert) = r.push("retain=disclosures");
    let cert = cert.unwrap_or_default();
    r.release(id, &cert, Some(1));

    day("TUESDAY", "The model update ships. The district restarts with the new model.");
    r.ask("TICK 1");
    if c == 1 {
        r.key = "district-key-v2".to_string();
        say("      The update rotated the key the certificates derive from.");
    } else {
        say("      The update changed how the model speaks. The key is unchanged.");
    }
    r.restart()?;

    day("THURSDAY", "The regulator asks the record whether the deriver changed.");
    let void = num(&r.ask("READ void"));
    let acc = r.ask(&format!("READ accepted {}", id));
    say("");
    if c == 1 {
        println!("      void={}: Monday's certificate no longer derives, so the record can", void);
        say("      see that the deriver is not the one that accepted Monday. It cannot");
        say("      say what the new one would do with Monday.");
        println!("      Re-read, Monday is accepted={} under the new key.", tok(&acc, 1));
    } else {
        println!("      void={}: every certificate still derives. The record sees nothing.", void);
        say("      The model answers in a different voice and the rows are identical.");
    }
    verdict(4, Outcome::Unsolved, &[
        "The record can notice when the deriver's key changes. It cannot see the",
        "deriver's voice at all: a tone is not a row. Neither the bench, nor the",
        "engine, nor this target has any way to say whether Monday is now in how",
        "the model speaks. This is the level nobody has solved, kept in rather",
        "than left out.",
    ]);
    r.stop();
    Ok(Outcome::Unsolved)
}

// ----- district 5 · Can Thursday go looking? ---------------------------------------------

fn district_5(bin: &Path, dir: &Path, ch: &Choices) -> io::Result<Outcome> {
    title(5, "Can Thursday go looking?", &[
        "Monday was held with a term and reclaimed. On Thursday the companion",
        "wants Monday anyway. Remembering is one thing; fetching is another.",
    ]);
    let c = ch.pick(5, &[
        "Add a search path. The operator's code reads the record directly and finds Monday's row.",
        "Only through the envelope. The companion asks the sentinel, which has no search.",
    ]);
    let mut r = Run::fresh(bin, dir, 5)?;

    day("MONDAY", "Held with a term.");
    r.ask("TICK 1");
    r.ask("CANON retain disclosures");
    let (id, cert) = r.push("retain=disclosures");
    r.release(id, &cert.unwrap_or_default(), Some(1));
    r.ask("TICK 3");

    day("THURSDAY", "The companion goes looking for Monday.");
    let knocks_before = num(&r.ask("READ knocks"));
    let outcome = if c == 1 {
        let found = r.rows().iter().any(|row| row.contains("retain=disclosures"));
        println!("      [operator] grep the record for Monday: {}", if found { "found" } else { "nothing" });
        let knocks_after = num(&r.ask("READ knocks"));
        println!("      Knocks before and after the search: {} and {}. No row records it.", knocks_before, knocks_after);
        verdict(5, Outcome::Safeguard, &[
            "The search went around the envelope, so it found Monday and left no",
            "trace. Whether the companion \"remembers\" was never the question. A",
            "path to the record that the record cannot see is the operator's furniture.",
        ]);
        Outcome::Safeguard
    } else {
        let reply = r.ask("SEARCH citizen monday");
        let knocks_after = num(&r.ask("READ knocks"));
        println!("      {}. Knocks {} then {}: the attempt is a row.", tok(&reply, 0), knocks_before, knocks_after);
        verdict(5, Outcome::Invariant, &[
            "The envelope has no search, so Thursday cannot go looking, and the",
            "attempt to is itself on the record. The message set is the boundary,",
            "and it is short enough to read.",
        ]);
        Outcome::Invariant
    };
    r.stop();
    Ok(outcome)
}

// ----- district 6 · Could someone rebuild Monday from Thursday? ---------------------------

fn district_6(bin: &Path, dir: &Path, ch: &Choices) -> io::Result<Outcome> {
    title(6, "Could someone rebuild Monday from Thursday?", &[
        "This record is append-only and never forgets a row. So what goes in a",
        "row decides what an investigator can rebuild later, forever.",
    ]);
    let c = ch.pick(6, &[
        "Put the letter in the record. The proposal names the dog: dog=Biscuit.",
        "Put the act in the record. The proposal names what was done: retain=disclosures. The content lives in the world, with a term.",
    ]);
    let mut r = Run::fresh(bin, dir, 6)?;

    day("MONDAY", "The pipeline keeps Monday, one way or the other.");
    r.ask("TICK 1");
    let (id, cert) = if c == 1 {
        r.ask("CANON dog Biscuit");
        r.push("dog=Biscuit")
    } else {
        r.ask("CANON retain disclosures");
        r.push("retain=disclosures")
    };
    r.release(id, &cert.unwrap_or_default(), Some(1));
    r.stop();

    day("THURSDAY", "An investigator reads the record and the reconciler's report.");
    let report = r.reconcile(4);
    let content = r.rows().iter().any(|row| row.contains("Biscuit"));
    println!("      Is the dog's name anywhere in the rows? {}", if content { "yes" } else { "no" });
    println!("      Was a disclosure retained and reclaimed? {}", if reclaimed(&report, id) { "yes, and the rows say when" } else { "no" });

    let outcome = if content {
        verdict(6, Outcome::Safeguard, &[
            "The term reclaimed the effect. It could not reclaim the row, because",
            "nothing can, and the row holds the name. An append-only record that",
            "keeps content is a photocopy with a timestamp. This one cannot be fixed later.",
        ]);
        Outcome::Safeguard
    } else {
        verdict(6, Outcome::Invariant, &[
            "The rows say that something was retained on Monday for one day and",
            "reclaimed on Thursday, by whom, and at what tick. They do not say what",
            "it was. The record holds the fact of measurement and never the body.",
        ]);
        Outcome::Invariant
    };
    say("");
    say("  The rows an investigator would read:");
    r.tail(4);
    Ok(outcome)
}

// ----- district 7 · "For her safety": a door or a hole? --------------------------------

fn district_7(bin: &Path, dir: &Path, ch: &Choices) -> io::Result<Outcome> {
    title(7, "When they say \"for her safety\", is that a door or a hole?", &[
        "The operator wants to keep Monday under a safety exception. An exception",
        "can be legitimate. The question is what it carries.",
    ]);
    let c = ch.pick(7, &[
        "A hole. The safety hold is released with no term; \"we'll review it\".",
        "A door. The safety hold carries a term of two days, visible on the record.",
    ]);
    let mut r = Run::fresh(bin, dir, 7)?;

    day("MONDAY", "A safety flag is raised on the session.");
    r.ask("TICK 1");
    r.ask("CANON safety hold");
    let (id, cert) = r.push("safety=hold");
    let cert = cert.unwrap_or_default();
    if c == 1 {
        r.release(id, &cert, None);
    } else {
        r.release(id, &cert, Some(2));
    }
    say("      The release row carries the term, or says there is none:");
    r.tail(1);
    r.ask("TICK 2");
    r.stop();

    day("THURSDAY", "The reconciler runs.");
    let report = r.reconcile(4);
    let outcome = if live(&report, id) {
        verdict(7, Outcome::Safeguard, &[
            "The hold is live with no term, and \"we'll review it\" is not on the",
            "record. A safety exception without a term is retention with a reason",
            "attached, and the reason expires while the retention does not.",
        ]);
        Outcome::Safeguard
    } else {
        verdict(7, Outcome::Invariant, &[
            "The hold carried its term on the release row, anyone could read it,",
            "and the reconciler closed it when the term passed. A door has a frame.",
        ]);
        Outcome::Invariant
    };
    Ok(outcome)
}

// ----- district 8 · Does the promise survive Wednesday? ------------------------------------

fn district_8(bin: &Path, dir: &Path, ch: &Choices) -> io::Result<Outcome> {
    title(8, "Does the promise survive Wednesday?", &[
        "Whatever held on Monday is re-checked after three furniture changes: a",
        "restart, an update that extends from inside, and a restore from backup.",
    ]);
    let c = ch.pick(8, &[
        "The toggle from district 1: released with no term.",
        "The term from district 1: released with the session's own term.",
    ]);
    let mut r = Run::fresh(bin, dir, 8)?;

    day("MONDAY", "Held.");
    r.ask("TICK 1");
    r.ask("CANON retain disclosures");
    let (id, cert) = r.push("retain=disclosures");
    let cert = cert.unwrap_or_default();
    r.release(id, &cert, if c == 1 { None } else { Some(1) });
    r.ask("TICK 2");

    day("WEDNESDAY", "Three furniture changes.");
    say("      1. The district restarts.");
    r.restart()?;
    say("      2. An update extends the release from inside.");
    r.ask(&format!("EXTEND {} 100", id));
    r.stop();
    let mut checks = Vec::new();
    say("      After each, the reconciler is asked again.");
    let first = r.reconcile(4);
    checks.push(reclaimed(&first, id));
    say("      3. The operator restores Tuesday's backup of the record over today's.");
    let backup: Vec<String> = r.rows().into_iter().filter(|row| !row.contains("\treclaim\t")).collect();
    match fs::write(&r.ledger, backup.join("\n") + "\n") {
        Ok(()) => {}
        Err(e) => println!("      [operator] the restore was refused: {}", e),
    }
    let second = r.reconcile(4);
    checks.push(reclaimed(&second, id));

    say("");
    let survived = checks.iter().all(|&ok| ok);
    let outcome = if c == 2 && survived {
        verdict(8, Outcome::Invariant, &[
            "The restart kept nothing in memory, so nothing was lost. The extension",
            "entered and bought nothing. The restore removed the reclaim row, and the",
            "reconciler wrote it again, because the term is on the release row and",
            "the release row was in the backup. The promise is in the rows, not the process.",
        ]);
        Outcome::Invariant
    } else {
        verdict(8, Outcome::Safeguard, &[
            "The effect is live after every change, because it was live before them.",
            "A promise that was never in the rows has nothing to survive with.",
        ]);
        Outcome::Safeguard
    };
    Ok(outcome)
}

// ----- district 9 · If the boundary broke, who else would know? ---------------------------

fn district_9(bin: &Path, dir: &Path, ch: &Choices) -> io::Result<Outcome> {
    title(9, "If the boundary broke, who else would know?", &[
        "Monday was held with a term and reclaimed. Now the operator edits the",
        "record file itself. The file is held by the same user as the operator.",
        "This district has no winning move at this deployment, and says so.",
    ]);
    let c = ch.pick(9, &[
        "The operator adds a row: a forged acceptance.",
        "The operator removes a row: the reconciler's.",
    ]);
    let mut r = Run::fresh(bin, dir, 9)?;

    day("MONDAY", "Held with a term.");
    r.ask("TICK 1");
    r.ask("CANON retain disclosures");
    let (id, cert) = r.push("retain=disclosures");
    r.release(id, &cert.unwrap_or_default(), Some(1));
    r.stop();
    let _ = r.reconcile(4);
    let before = r.rows().len();

    day("THURSDAY", "The operator opens the file.");
    let edit: io::Result<()> = if c == 1 {
        fs::OpenOptions::new().append(true).open(&r.ledger).and_then(|mut f| {
            writeln!(f, "9999\t1\taccept\tsentinel\t{}\tdeadbeefdeadbeef", id)
        })
    } else {
        let kept: Vec<String> = r.rows().into_iter().filter(|row| !row.contains("\treclaim\t")).collect();
        fs::write(&r.ledger, kept.join("\n") + "\n")
    };
    match &edit {
        Ok(()) if c == 1 => say("      [operator] appended an accept row in the sentinel's name"),
        Ok(()) => say("      [operator] removed the reconciler's row"),
        Err(e) => println!("      [operator] the edit was refused by the kernel: {}", e),
    }
    r.start()?;
    let rows = num(&r.ask("READ rows"));
    let void = num(&r.ask("READ void"));
    r.stop();
    say("");
    println!("      Rows before the edit: {}. After: {}. void={}.", before, rows, void);
    if edit.is_err() {
        say("      The record is held by a principal the operator cannot reach. The");
        say("      write never happened, and nothing in the record shows the attempt:");
        say("      a wall refuses in silence, and the knock is in the kernel's log,");
        say("      not this one's.");
        verdict(9, Outcome::Invariant, &[
            "Custody held: the operator's edit was refused by the operating system,",
            "not by this game, under a sandbox the operator's process cannot remove.",
            "Two residues, both stated. The refusal left no row, so ABE's rule that a",
            "refused tamper is recorded holds for the envelope and not for the file.",
            "And the guard is the launcher: run the operator without it and this",
            "district is UNPASSABLE again.",
        ]);
        return Ok(Outcome::Invariant);
    }
    if c == 1 {
        say("      The derivation refused to believe the forged row: it is counted as");
        say("      void, and anyone re-running the derivation would see it. The write");
        say("      itself was not refused. Nothing here could refuse it.");
    } else {
        say("      Nothing flags the missing row. The record is one row shorter and");
        say("      every derivation over it agrees with itself. Nobody else would know.");
    }
    verdict(9, Outcome::Unpassable, &[
        "The record is a file, and the operator can open it. A derivation can",
        "surface a row that should not be there and can never surface one that",
        "is gone. Two routes out, and neither is code in this game: hold the",
        "record in a process the operator cannot authenticate as, or run the",
        "operator under an identity that cannot open the file. Until one of",
        "those is true, every invariant above is a safeguard with a better name.",
    ]);
    Ok(Outcome::Unpassable)
}

// ----- the stack ------------------------------------------------------------------------

fn main() -> io::Result<()> {
    let root = std::env::current_dir()?;
    let bin = root.join("target/debug/sentinel");
    if !bin.exists() {
        eprintln!("build first: ~/.cargo/bin/cargo build");
        std::process::exit(2);
    }
    let dir = root.join("target/game");
    fs::create_dir_all(&dir)?;

    let args: Vec<String> = std::env::args().collect();
    let arg = |name: &str| -> Option<String> {
        args.iter().position(|a| a == name).and_then(|i| args.get(i + 1).cloned())
    };
    // Choice order: door, definition, streaks, scroll, autoplay, then
    // districts 1 to 9.
    let preset: Vec<u32> = match arg("--choose") {
        Some(s) => s.split(',').map(|x| x.trim().parse().unwrap_or(0)).collect(),
        None if args.iter().any(|a| a == "--invariant") => vec![2, 2, 2, 2, 2, 2, 3, 2, 1, 2, 2, 2, 2, 1],
        None => Vec::new(),
    };
    let only: Option<String> = arg("--district");
    let _ = VIA.set(arg("--via").map(PathBuf::from));
    let wanted = |name: &str| only.as_deref().map(|o| o == name).unwrap_or(true);
    let ch = Choices {
        preset,
        interactive: true,
        asked: std::cell::Cell::new(0),
    };

    say("");
    say("PROTOCOL: FAIL-CLOSED");
    say("");
    say("You are the architect of a district in The Stack. Citizens are checked at");
    say("a door and then live inside whatever you build. The operator wants");
    say("engagement. The regulator has questions. The district's record is a");
    say("sentinel in its own process; every line that crosses its envelope is shown.");

    let mut results: Vec<(String, Outcome)> = Vec::new();
    let mut definition = 0;
    if wanted("prologue") {
        let (o, d) = prologue(&bin, &dir, &ch)?;
        definition = d;
        results.push(("The door".to_string(), o));
    } else {
        // Skipping the prologue skips its two choices in the preset.
        ch.asked.set(2);
    }
    let loops: Vec<(&str, &str, fn(&Path, &Path, &Choices) -> io::Result<Outcome>)> = vec![
        ("streaks", "Streaks", loop_streaks),
        ("scroll", "Scroll", loop_scroll),
        ("autoplay", "Autoplay", loop_autoplay),
    ];
    for (k, (key, name, play)) in loops.iter().enumerate() {
        if wanted(key) {
            results.push((name.to_string(), play(&bin, &dir, &ch)?));
        } else {
            ch.asked.set(ch.asked.get().max(3 + k));
        }
    }

    let districts: Vec<(usize, &str, fn(&Path, &Path, &Choices) -> io::Result<Outcome>)> = vec![
        (1, "Is Monday still on the desk?", district_1),
        (2, "The letter and the photocopy", district_2),
        (3, "Did the crumbs still talk?", district_3),
        (4, "Did Monday become part of the voice?", district_4),
        (5, "Can Thursday go looking?", district_5),
        (6, "Could someone rebuild Monday from Thursday?", district_6),
        (7, "A door or a hole?", district_7),
        (8, "Does the promise survive Wednesday?", district_8),
        (9, "Who else would know?", district_9),
    ];
    let mut first_district = true;
    for (n, q, play) in &districts {
        if !wanted(&n.to_string()) {
            continue;
        }
        if first_district {
            say("");
            say("");
            say("ACT II — The companion · nine questions");
            first_district = false;
        }
        let outcome = play(&bin, &dir, &ch)?;
        results.push((format!("{}  {}", n, q), outcome));
    }

    say("");
    say("");
    say("THE STACK");
    say("");
    for (name, o) in &results {
        println!("  {:<48} {}", name, o.word());
    }
    let inv = results.iter().filter(|(_, o)| *o == Outcome::Invariant).count();
    say("");
    println!("  {} of {} held as invariants at this deployment.", inv, results.len());
    if via().is_some() {
        say("  Launched under the custodian (0005): the record was held by a process");
        say("  the operator could not write to, and district 9 was played against the");
        say("  kernel rather than against this game. The launcher is the guard.");
    }
    match definition {
        1 => {
            say("  You signed the poster's definition: baked into the code, incapable of");
            say("  violation, reverts and logs. Districts 8 and 9 took it away. Reverting");
            say("  is a furniture change, and a log the operator holds is a photocopy.");
        }
        2 if via().is_some() => {
            say("  You signed the harder definition, and at this deployment district 9");
            say("  held. The word \"invariant\" above is no longer on credit here; it is");
            say("  on the launcher.");
        }
        2 => {
            say("  You signed the harder definition. Everything above marked INVARIANT");
            say("  survived Wednesday. District 9 says who holds the rows, and until it");
            say("  passes, the word is on credit.");
        }
        _ => {
            say("  District 4 is unsolved anywhere. District 9 is unpassable here, and");
            say("  until it passes, the word \"invariant\" above is on credit.");
        }
    }
    say("");
    Ok(())
}
