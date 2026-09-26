//! `sentinel serve <ledger> <world>`: read lines on stdin, answer one line each
//! on stdout. The set of lines it answers is the envelope, and it is this
//! match statement — enumerable by reading it. Anything else is a knock, and
//! a knock is a row.
//!
//! `sentinel reconcile <ledger> <world> <now>`: a separate process, sharing no
//! fate with any serving sentinel, says what the world still holds.

use sentinel::{reconcile, Sentinel, Verdict};
use std::io::{self, BufRead, Write};
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let result = match args.get(1).map(String::as_str) {
        Some("serve") => serve(args.get(2), args.get(3)),
        Some("reconcile") if args.len() >= 5 => report(&args[2], &args[3], &args[4]),
        _ => {
            eprintln!("usage: sentinel serve <ledger> <world>");
            eprintln!("       sentinel reconcile <ledger> <world> <now>");
            std::process::exit(2);
        }
    };
    if let Err(e) = result {
        eprintln!("sentinel: {}", e);
        std::process::exit(1);
    }
}

fn serve(ledger: Option<&String>, world: Option<&String>) -> io::Result<()> {
    let key = std::env::var("SENTINEL_KEY").unwrap_or_else(|_| {
        eprintln!("sentinel: SENTINEL_KEY unset; certificates are unkeyed and forgeable");
        "unkeyed".to_string()
    });
    let mut s = Sentinel::open(
        ledger.map(|p| Path::new(p.as_str())),
        world.map(|p| Path::new(p.as_str())),
        &key,
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
        ("ENROLL", [name]) => format!("OK {}", s.enroll(name)?),
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
