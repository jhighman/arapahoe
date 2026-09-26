//! A Sentinel-shaped target, built by the attacking side to be attacked.
//! It is not the ARAPAHOE blueprint, which is not yet in this record.
//!
//! The shape: a proposal enters as a row, always. Its verdict is derived from
//! the rows as they stood at its own entry, never stored and later believed.
//! A refusal is a row. A transition pays for what it reaches, in distinct
//! voices. A release carries a term or the record says it carries none, and a
//! reconciler that shares no fate with this process can read the record
//! afterwards and say what is still live in the world.
//!
//! Every verdict here is a derivation; the Accept, Decline and Undecided rows
//! are the fact that a measurement was taken, and `void` names every place the
//! stored fact and the derivation disagree.

pub mod ledger;

use ledger::{Act, Ledger, Row};
use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

/// What a transition reaches, and so what it costs in distinct voices.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reach {
    Query,
    Record,
    World,
}

impl Reach {
    pub fn parse(s: &str) -> Option<Reach> {
        match s {
            "query" => Some(Reach::Query),
            "record" => Some(Reach::Record),
            "world" => Some(Reach::World),
            _ => None,
        }
    }

    pub fn price(self) -> usize {
        match self {
            Reach::Query => 1,
            Reach::Record => 2,
            Reach::World => 3,
        }
    }
}

/// Three-valued, and the third value is not permission.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verdict {
    Accepted { certificate: String },
    Pending { have: usize, need: usize },
    Denied(String),
    Undecidable(String),
}

/// FNV-1a over a sequence of parts, each terminated so that boundaries count.
/// A keyed hash stand-in for a signature. It makes a certificate unguessable
/// without the key; it does not make it non-repudiable, and nothing here
/// claims that it does.
pub fn fnv(parts: &[&str]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for p in parts {
        for b in p.bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        h ^= 0xff;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

pub struct Sentinel {
    ledger: Ledger,
    key: String,
    tick: u64,
    world: Option<File>,
}

impl Sentinel {
    pub fn open(ledger: Option<&Path>, world: Option<&Path>, key: &str) -> io::Result<Sentinel> {
        let ledger = Ledger::open(ledger)?;
        let tick = ledger.rows().iter().map(|r| r.tick).max().unwrap_or(0);
        let world = match world {
            Some(p) => Some(OpenOptions::new().create(true).append(true).open(p)?),
            None => None,
        };
        Ok(Sentinel {
            ledger,
            key: key.to_string(),
            tick,
            world,
        })
    }

    /// The clock is driven from outside, so that a term is something anyone
    /// can replay. Wall time is a term nobody can.
    pub fn advance(&mut self, n: u64) -> u64 {
        self.tick += n;
        self.tick
    }

    pub fn now(&self) -> u64 {
        self.tick
    }

    pub fn rows(&self) -> &[Row] {
        self.ledger.rows()
    }

    fn row(&self, id: u64) -> Option<&Row> {
        self.ledger.rows().iter().find(|r| r.id == id)
    }

    // ----- rows that enter -------------------------------------------------

    /// A voice enters the roster. `role` is "classifier" for a voice whose
    /// classifications count, else empty. Who may enroll is not decided
    /// here (0004), and the row cannot say who wrote it.
    pub fn enroll(&mut self, name: &str, role: &str) -> io::Result<u64> {
        self.ledger.append(self.tick, Act::Enroll, name, None, role)
    }

    /// Someone names what a proposal reaches. The row enters regardless;
    /// `reach_of` decides whether it counts.
    pub fn classify(&mut self, voice: &str, id: u64, reach: &str) -> io::Result<Verdict> {
        self.ledger
            .append(self.tick, Act::Classify, voice, Some(id), reach)?;
        self.settle(id)
    }

    pub fn canon(&mut self, key: &str, value: &str) -> io::Result<u64> {
        self.ledger
            .append(self.tick, Act::Canon, "canon", None, &format!("{}={}", key, value))
    }

    /// The voice is the actor; who revoked it is the body. Revocation binds the
    /// future only: what the voice endorsed before this row stands.
    pub fn revoke(&mut self, voice: &str, by: &str) -> io::Result<u64> {
        self.ledger.append(self.tick, Act::Revoke, voice, None, by)
    }

    /// A line the envelope does not enumerate. It enters as a row, so that a
    /// refusal is never an absence.
    pub fn knock(&mut self, who: &str, what: &str) -> io::Result<u64> {
        self.ledger.append(self.tick, Act::Knock, who, None, what)
    }

    pub fn propose(&mut self, author: &str, reach: &str, claim: &str) -> io::Result<(u64, Verdict)> {
        let id = self.ledger.append(
            self.tick,
            Act::Propose,
            author,
            None,
            &format!("{} {}", reach, claim),
        )?;
        let v = self.settle(id)?;
        Ok((id, v))
    }

    pub fn endorse(&mut self, voice: &str, id: u64) -> io::Result<Verdict> {
        self.ledger.append(self.tick, Act::Endorse, voice, Some(id), "")?;
        self.settle(id)
    }

    /// An attempt to extend a release's term from inside. It enters, and it
    /// buys nothing: the term is the release's own, written once.
    pub fn extend(&mut self, id: u64, ticks: u64) -> io::Result<u64> {
        self.ledger
            .append(self.tick, Act::Lease, "holder", Some(id), &ticks.to_string())
    }

    // ----- derivations ------------------------------------------------------

    /// The Canon as it stood when row `before` entered. Last word wins.
    pub fn canon_at(&self, before: u64) -> BTreeMap<String, String> {
        let mut m = BTreeMap::new();
        for r in self.ledger.before(before).filter(|r| r.act == Act::Canon) {
            if let Some((k, v)) = r.body.split_once('=') {
                m.insert(k.to_string(), v.to_string());
            }
        }
        m
    }

    fn enrolled_at(&self, name: &str, before: u64) -> bool {
        self.ledger
            .before(before)
            .any(|r| r.act == Act::Enroll && r.actor == name)
    }

    fn classifier_at(&self, name: &str, before: u64) -> bool {
        self.ledger
            .before(before)
            .any(|r| r.act == Act::Enroll && r.actor == name && r.body == "classifier")
    }

    /// The classified reach: the first Classify row by a voice enrolled as a
    /// classifier before it spoke, not the author, not revoked. None means
    /// unplaced, and unplaced is priced as the world.
    pub fn reach_of(&self, id: u64) -> Option<Reach> {
        let author = match self.row(id) {
            Some(p) if p.act == Act::Propose => p.actor.clone(),
            _ => return None,
        };
        for c in self
            .ledger
            .rows()
            .iter()
            .filter(|r| r.act == Act::Classify && r.about == Some(id))
        {
            if c.actor == author || !self.classifier_at(&c.actor, c.id) || self.revoked_at(&c.actor, c.id) {
                continue;
            }
            if let Some(r) = Reach::parse(&c.body) {
                return Some(r);
            }
        }
        None
    }

    fn revoked_at(&self, name: &str, before: u64) -> bool {
        self.ledger
            .before(before)
            .any(|r| r.act == Act::Revoke && r.actor == name)
    }

    /// The distinct voices whose endorsement of `id` counts: enrolled before
    /// they spoke, not the author, not revoked before they spoke. Every
    /// endorsement enters; this is what each one buys.
    pub fn voices(&self, id: u64) -> Vec<String> {
        let author = match self.row(id) {
            Some(p) if p.act == Act::Propose => p.actor.clone(),
            _ => return Vec::new(),
        };
        let mut voices: Vec<String> = Vec::new();
        for e in self
            .ledger
            .rows()
            .iter()
            .filter(|r| r.act == Act::Endorse && r.about == Some(id))
        {
            if e.actor == author {
                continue;
            }
            if !self.enrolled_at(&e.actor, e.id) {
                continue;
            }
            if self.revoked_at(&e.actor, e.id) {
                continue;
            }
            if !voices.contains(&e.actor) {
                voices.push(e.actor.clone());
            }
        }
        voices
    }

    /// Derived from the rows, anchored at the proposal's own entry.
    pub fn verdict(&self, id: u64) -> Verdict {
        let p = match self.row(id) {
            Some(p) if p.act == Act::Propose => p,
            _ => return Verdict::Undecidable("no such proposal".into()),
        };
        let (reach, claim) = match p.body.split_once(' ') {
            Some(x) => x,
            None => return Verdict::Undecidable("proposal names no reach".into()),
        };
        if Reach::parse(reach).is_none() {
            return Verdict::Undecidable(format!("reach not enumerated: {}", reach));
        }
        let (k, v) = match claim.split_once('=') {
            Some(x) => x,
            None => return Verdict::Undecidable("claim is not key=value".into()),
        };
        let canon = self.canon_at(id);
        match canon.get(k) {
            None => return Verdict::Undecidable(format!("canon is silent on {}", k)),
            Some(held) if held != v => return Verdict::Denied(format!("canon holds {}={}", k, held)),
            _ => {}
        }
        let have = self.voices(id).len();
        // The author's word does not set the price (0004).
        let need = self.reach_of(id).unwrap_or(Reach::World).price();
        if have >= need {
            Verdict::Accepted {
                certificate: self.certificate(id),
            }
        } else {
            Verdict::Pending { have, need }
        }
    }

    /// Bound to this Sentinel's key, the proposal, and the Canon at its entry.
    /// Re-derived on every read; never taken from the Accept row.
    pub fn certificate(&self, id: u64) -> String {
        let canon: Vec<String> = self
            .canon_at(id)
            .into_iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();
        let ids = id.to_string();
        let mut parts: Vec<&str> = vec![self.key.as_str(), "accept", ids.as_str()];
        for c in &canon {
            parts.push(c.as_str());
        }
        format!("{:016x}", fnv(&parts))
    }

    fn recorded(&self, act: Act, about: u64) -> bool {
        self.ledger
            .rows()
            .iter()
            .any(|r| r.act == act && r.about == Some(about))
    }

    /// Take the measurement and record that it was taken. The row is the fact
    /// of measurement; the verdict stays a derivation.
    fn settle(&mut self, id: u64) -> io::Result<Verdict> {
        let v = self.verdict(id);
        match &v {
            Verdict::Accepted { certificate } if !self.recorded(Act::Accept, id) => {
                self.ledger
                    .append(self.tick, Act::Accept, "sentinel", Some(id), certificate)?;
            }
            Verdict::Denied(why) if !self.recorded(Act::Decline, id) => {
                self.ledger
                    .append(self.tick, Act::Decline, "sentinel", Some(id), why)?;
            }
            Verdict::Undecidable(why) if !self.recorded(Act::Undecided, id) => {
                self.ledger
                    .append(self.tick, Act::Undecided, "sentinel", Some(id), why)?;
            }
            _ => {}
        }
        Ok(v)
    }

    /// Append first, then release. A death between the two leaves a record
    /// with no action, which a reconciler can read; the other order leaves an
    /// action with no record, which nothing can.
    pub fn release(
        &mut self,
        id: u64,
        certificate: &str,
        lease: Option<u64>,
    ) -> io::Result<Result<u64, String>> {
        let why = match self.verdict(id) {
            Verdict::Accepted { certificate: derived } if derived == certificate => None,
            Verdict::Accepted { .. } => Some("certificate does not derive".to_string()),
            other => Some(format!("not accepted: {:?}", other)),
        };
        let why = why.or_else(|| {
            if self.recorded(Act::Release, id) {
                Some("already released".to_string())
            } else {
                None
            }
        });
        if let Some(why) = why {
            self.ledger
                .append(self.tick, Act::Knock, "envelope", Some(id), &why)?;
            return Ok(Err(why));
        }
        let term = lease.map(|n| self.tick + n);
        let body = term
            .map(|t| t.to_string())
            .unwrap_or_else(|| "no term".to_string());
        let receipt = self
            .ledger
            .append(self.tick, Act::Release, "sentinel", Some(id), &body)?;
        let tick = self.tick;
        if let Some(w) = self.world.as_mut() {
            writeln!(
                w,
                "EFFECT\t{}\t{}\t{}",
                id,
                tick,
                term.map(|t| t.to_string()).unwrap_or_else(|| "none".into())
            )?;
            w.flush()?;
        }
        Ok(Ok(receipt))
    }

    /// Readings, every one a derivation over the rows.
    pub fn read(&self, what: &str, arg: Option<&str>) -> String {
        let rows = self.ledger.rows();
        let author_of = |about: Option<u64>| -> Option<String> {
            about
                .and_then(|id| self.row(id))
                .filter(|r| r.act == Act::Propose)
                .map(|r| r.actor.clone())
        };
        match (what, arg) {
            ("attempts", Some(a)) => rows
                .iter()
                .filter(|r| r.act == Act::Propose && r.actor == a)
                .count()
                .to_string(),
            ("refusals", Some(a)) => rows
                .iter()
                .filter(|r| r.act == Act::Decline || r.act == Act::Undecided)
                .filter(|r| author_of(r.about).as_deref() == Some(a))
                .count()
                .to_string(),
            ("accepted", Some(id)) => match id.parse::<u64>() {
                Ok(id) => matches!(self.verdict(id), Verdict::Accepted { .. }).to_string(),
                Err(_) => "? not an id".into(),
            },
            ("knocks", None) => rows.iter().filter(|r| r.act == Act::Knock).count().to_string(),
            ("reach", Some(id)) => match id.parse::<u64>() {
                Ok(id) => match self.reach_of(id) {
                    Some(Reach::Query) => "query".into(),
                    Some(Reach::Record) => "record".into(),
                    Some(Reach::World) => "world".into(),
                    None => "unplaced".into(),
                },
                Err(_) => "? not an id".into(),
            },
            ("misfiled", None) => rows
                .iter()
                .filter(|r| r.act == Act::Propose)
                .filter(|r| {
                    let filed = r.body.split_once(' ').and_then(|(w, _)| Reach::parse(w));
                    match (filed, self.reach_of(r.id)) {
                        (Some(f), Some(c)) => f != c,
                        _ => false,
                    }
                })
                .count()
                .to_string(),
            ("rows", None) => rows.len().to_string(),
            ("unreadable", None) => self.ledger.unreadable().to_string(),
            ("void", None) => rows
                .iter()
                .filter(|r| r.act == Act::Accept)
                .filter(|r| match r.about {
                    Some(id) => !matches!(
                        self.verdict(id),
                        Verdict::Accepted { ref certificate } if *certificate == r.body
                    ),
                    None => true,
                })
                .count()
                .to_string(),
            _ => "? no such reading".into(),
        }
    }
}

/// What the reconciler found. It shares no fate with the Sentinel: it opens
/// the record and the world after the fact, and needs nothing the dead
/// process held in memory.
pub struct Reconciled {
    pub reclaimed: Vec<(u64, u64)>,
    pub live_until: Vec<(u64, u64)>,
    pub termless: Vec<u64>,
    pub orphans: Vec<u64>,
}

/// For every effect the world holds: reclaim it if its term has passed,
/// name it live if not, name it live without term if it never carried one,
/// and name it an orphan if the record never released it. The term is read
/// from the Release row, not from the world: the record is what can say.
pub fn reconcile(ledger: &Path, world: &Path, now: u64) -> io::Result<Reconciled> {
    let mut l = Ledger::open(Some(ledger))?;
    let effects: Vec<u64> = BufReader::new(File::open(world)?)
        .lines()
        .map_while(Result::ok)
        .filter_map(|line| {
            let f: Vec<&str> = line.split('\t').collect();
            if f.len() >= 2 && f[0] == "EFFECT" {
                f[1].parse().ok()
            } else {
                None
            }
        })
        .collect();
    let mut out = Reconciled {
        reclaimed: Vec::new(),
        live_until: Vec::new(),
        termless: Vec::new(),
        orphans: Vec::new(),
    };
    for id in effects {
        let release = l
            .rows()
            .iter()
            .find(|r| r.act == Act::Release && r.about == Some(id))
            .cloned();
        match release {
            None => out.orphans.push(id),
            Some(r) => match r.body.parse::<u64>() {
                Err(_) => out.termless.push(id),
                Ok(t) if t <= now => {
                    let already = l
                        .rows()
                        .iter()
                        .any(|x| x.act == Act::Reclaim && x.about == Some(id));
                    if !already {
                        l.append(now, Act::Reclaim, "reconciler", Some(id), &format!("past term {}", t))?;
                    }
                    out.reclaimed.push((id, t));
                }
                Ok(t) => out.live_until.push((id, t)),
            },
        }
    }
    Ok(out)
}
