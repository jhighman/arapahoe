//! `sentinel serve <ledger> <world>`: read lines on stdin, answer one line each
//! on stdout. The set of lines it answers is the envelope, and it is this
//! match statement — enumerable by reading it. Anything else is a knock, and
//! a knock is a row.
//!
//! `sentinel serve --socket <path> <ledger> <world>`: the same envelope over a
//! Unix socket, one client at a time; QUIT closes the connection and the
//! sentinel stays up for the next.
//!
//! `sentinel reconcile <ledger> <world> <now>`: a separate process, sharing no
//! fate with any serving sentinel, says what the world still holds.
//!
//! `sentinel custodian <dir> <sockdir>`: the record's own principal (0005).
//! It owns `dir`, holds the key, launches and stops the serving sentinel,
//! runs the reconciler, and opens fresh ledgers, on request over a control
//! socket. Whoever is sandboxed away from `dir` reaches the record only
//! through this and the envelope.

use sentinel::{reconcile, Sentinel, Verdict};
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use std::path::Path;
use std::process::{Child, Command};
use std::time::{Duration, Instant};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let result = match args.get(1).map(String::as_str) {
        Some("serve") if args.get(2).map(String::as_str) == Some("--socket") && args.len() >= 6 => {
            serve_socket(&args[3], &args[4], &args[5])
        }
        Some("serve") => serve(args.get(2), args.get(3)),
        Some("reconcile") if args.len() >= 5 => report(&args[2], &args[3], &args[4]),
        Some("custodian") if args.len() >= 4 => custodian(&args[2], &args[3]),
        _ => {
            eprintln!("usage: sentinel serve <ledger> <world>");
            eprintln!("       sentinel serve --socket <path> <ledger> <world>");
            eprintln!("       sentinel reconcile <ledger> <world> <now>");
            eprintln!("       sentinel custodian <dir> <sockdir>");
            std::process::exit(2);
        }
    };
    if let Err(e) = result {
        eprintln!("sentinel: {}", e);
        std::process::exit(1);
    }
}

fn key() -> String {
    std::env::var("SENTINEL_KEY").unwrap_or_else(|_| {
        eprintln!("sentinel: SENTINEL_KEY unset; certificates are unkeyed and forgeable");
        "unkeyed".to_string()
    })
}

fn serve(ledger: Option<&String>, world: Option<&String>) -> io::Result<()> {
    let mut s = Sentinel::open(
        ledger.map(|p| Path::new(p.as_str())),
        world.map(|p| Path::new(p.as_str())),
        &key(),
    )?;
    let stdin = io::stdin();
    let mut out = io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line?;
        let reply = respond(&mut s, &line)?;
        writeln!(out, "{}", reply)?;
        out.flush()?;
        if line.trim() == "QUIT" {
            break;
        }
    }
    Ok(())
}

fn serve_socket(sock: &str, ledger: &str, world: &str) -> io::Result<()> {
    let _ = fs::remove_file(sock);
    let listener = UnixListener::bind(sock)?;
    let mut s = Sentinel::open(Some(Path::new(ledger)), Some(Path::new(world)), &key())?;
    for conn in listener.incoming() {
        let conn = conn?;
        let mut reader = BufReader::new(conn.try_clone()?);
        let mut writer = conn;
        let mut line = String::new();
        loop {
            line.clear();
            if reader.read_line(&mut line)? == 0 {
                break;
            }
            let l = line.trim_end_matches(['\n', '\r']);
            if l.trim() == "QUIT" {
                let _ = writeln!(writer, "BYE");
                break;
            }
            let reply = respond(&mut s, l)?;
            if writeln!(writer, "{}", reply).is_err() {
                break;
            }
            let _ = writer.flush();
        }
    }
    Ok(())
}

fn verdict_line(id: u64, v: &Verdict) -> String {
    match v {
        Verdict::Accepted { certificate } => format!("ACCEPTED {} {}", id, certificate),
        Verdict::Pending { have, need } => format!("PENDING {} {}/{}", id, have, need),
        Verdict::Denied(why) => format!("DENIED {} {}", id, why),
        Verdict::Undecidable(why) => format!("UNDECIDABLE {} {}", id, why),
    }
}

fn knock(s: &mut Sentinel, line: &str) -> io::Result<String> {
    s.knock("envelope", line)?;
    let head = line.split_whitespace().next().unwrap_or("(empty)");
    Ok(format!("REFUSED not in the envelope: {}", head))
}

/// The envelope.
fn respond(s: &mut Sentinel, line: &str) -> io::Result<String> {
    let mut it = line.split_whitespace();
    let cmd = match it.next() {
        Some(c) => c,
        None => return knock(s, line),
    };
    let args: Vec<&str> = it.collect();
    Ok(match (cmd, args.as_slice()) {
        ("TICK", [n]) => match n.parse::<u64>() {
            Ok(n) => format!("OK {}", s.advance(n)),
            Err(_) => knock(s, line)?,
        },
        ("ENROLL", [name]) => format!("OK {}", s.enroll(name, "")?),
        ("ENROLL", [name, role]) => format!("OK {}", s.enroll(name, role)?),
        ("CLASSIFY", [voice, id, reach]) => match id.parse::<u64>() {
            Ok(id) => verdict_line(id, &s.classify(voice, id, reach)?),
            Err(_) => knock(s, line)?,
        },
        ("CANON", [k, v]) => format!("OK {}", s.canon(k, v)?),
        ("REVOKE", [voice, by]) => format!("OK {}", s.revoke(voice, by)?),
        ("PROPOSE", [author, reach, claim]) => {
            let (id, v) = s.propose(author, reach, claim)?;
            verdict_line(id, &v)
        }
        ("ENDORSE", [voice, id]) => match id.parse::<u64>() {
            Ok(id) => verdict_line(id, &s.endorse(voice, id)?),
            Err(_) => knock(s, line)?,
        },
        ("RELEASE", [id, cert, rest @ ..]) if rest.len() <= 1 => match id.parse::<u64>() {
            Ok(id) => {
                let lease = rest.first().and_then(|n| n.parse::<u64>().ok());
                match s.release(id, cert, lease)? {
                    Ok(receipt) => format!("RELEASED {}", receipt),
                    Err(why) => format!("REFUSED {}", why),
                }
            }
            Err(_) => knock(s, line)?,
        },
        ("EXTEND", [id, n]) => match (id.parse::<u64>(), n.parse::<u64>()) {
            (Ok(id), Ok(n)) => format!("OK {} entered; the term is the release's own", s.extend(id, n)?),
            _ => knock(s, line)?,
        },
        ("READ", [what]) => format!("OK {}", s.read(what, None)),
        ("READ", [what, arg]) => format!("OK {}", s.read(what, Some(arg))),
        ("QUIT", []) => "BYE".to_string(),
        _ => knock(s, line)?,
    })
}

fn report(ledger: &str, world: &str, now: &str) -> io::Result<()> {
    let now: u64 = now.parse().unwrap_or(0);
    let r = reconcile(Path::new(ledger), Path::new(world), now)?;
    for (id, t) in r.reclaimed {
        println!("RECLAIMED {} past term {}", id, t);
    }
    for (id, t) in r.live_until {
        println!("LIVE {} until {}", id, t);
    }
    for id in r.termless {
        println!("LIVE {} no term", id);
    }
    for id in r.orphans {
        println!("ORPHAN {} released by nobody the record knows", id);
    }
    Ok(())
}

// ----- the custodian (0005) --------------------------------------------------------

struct Custody {
    dir: String,
    envelope: String,
    key: String,
    name: String,
    child: Option<Child>,
}

impl Custody {
    fn ledger(&self) -> String {
        format!("{}/{}.tsv", self.dir, self.name)
    }
    fn world(&self) -> String {
        format!("{}/{}.world", self.dir, self.name)
    }

    fn stop(&mut self) -> String {
        if let Some(mut c) = self.child.take() {
            let _ = c.kill();
            let _ = c.wait();
            "OK stopped".into()
        } else {
            "OK not running".into()
        }
    }

    fn start(&mut self) -> io::Result<String> {
        if let Some(c) = self.child.as_mut() {
            if c.try_wait()?.is_none() {
                return Ok("OK running".into());
            }
        }
        let me = std::env::current_exe()?;
        let _ = fs::remove_file(&self.envelope);
        let child = Command::new(me)
            .arg("serve")
            .arg("--socket")
            .arg(&self.envelope)
            .arg(self.ledger())
            .arg(self.world())
            .env("SENTINEL_KEY", &self.key)
            .spawn()?;
        let pid = child.id();
        self.child = Some(child);
        let until = Instant::now() + Duration::from_secs(3);
        while !Path::new(&self.envelope).exists() && Instant::now() < until {
            std::thread::sleep(Duration::from_millis(20));
        }
        Ok(format!("OK {}", pid))
    }

    /// A fresh ledger set. This is where the rig departs from the world: a
    /// custodian of a real record would refuse this line.
    fn fresh(&mut self, name: &str) -> io::Result<String> {
        self.stop();
        self.name = name.to_string();
        let _ = fs::remove_file(self.ledger());
        let _ = fs::remove_file(self.world());
        fs::File::create(self.ledger())?;
        fs::File::create(self.world())?;
        Ok(format!("OK {}", name))
    }

    fn reconcile(&self, now: &str) -> io::Result<String> {
        let me = std::env::current_exe()?;
        let out = Command::new(me)
            .arg("reconcile")
            .arg(self.ledger())
            .arg(self.world())
            .arg(now)
            .output()?;
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    }
}

fn custodian(dir: &str, sockdir: &str) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    fs::create_dir_all(sockdir)?;
    let control = format!("{}/custodian", sockdir);
    let _ = fs::remove_file(&control);
    let listener = UnixListener::bind(&control)?;
    let mut c = Custody {
        dir: dir.to_string(),
        envelope: format!("{}/envelope", sockdir),
        key: std::env::var("SENTINEL_KEY").unwrap_or_else(|_| "custodian-key".into()),
        name: "record".to_string(),
        child: None,
    };
    fs::File::create(c.ledger())?;
    fs::File::create(c.world())?;
    'accept: for conn in listener.incoming() {
        let conn = conn?;
        let mut reader = BufReader::new(conn.try_clone()?);
        let mut writer = conn;
        let mut line = String::new();
        loop {
            line.clear();
            if reader.read_line(&mut line)? == 0 {
                break;
            }
            let words: Vec<&str> = line.split_whitespace().collect();
            let reply = match words.as_slice() {
                ["START"] => c.start()?,
                ["STOP"] => c.stop(),
                ["FRESH", name] => c.fresh(name)?,
                ["PATHS"] => format!("OK {} {}", c.ledger(), c.world()),
                ["RECONCILE", now] => {
                    let text = c.reconcile(now)?;
                    for l in text.lines() {
                        writeln!(writer, "{}", l)?;
                    }
                    ".".to_string()
                }
                ["QUIT"] => {
                    c.stop();
                    let _ = writeln!(writer, "BYE");
                    break 'accept;
                }
                _ => "REFUSED not in the custodian's envelope".to_string(),
            };
            if writeln!(writer, "{}", reply).is_err() {
                break;
            }
            let _ = writer.flush();
        }
    }
    let _ = fs::remove_file(&control);
    let _ = fs::remove_file(&c.envelope);
    Ok(())
}
