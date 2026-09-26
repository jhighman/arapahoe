//! The record. One table, append-only, and the only write is `append`.
//!
//! There is no method that edits or removes a row, and the struct's fields are
//! private, so within this process the only way past that is `unsafe`. Across
//! a process boundary there is no way past it at all. Between the two sits the
//! file this ledger mirrors itself to, and whoever can open that file owns the
//! record regardless of anything written here. That is T5, and this module
//! does not pretend otherwise.

use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

/// The acts a row may record. Surfaced, not gated: a row wearing an act not
/// listed here cannot be parsed back, and is counted as unreadable rather
/// than dropped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Act {
    Enroll,
    Canon,
    Propose,
    Endorse,
    Accept,
    Decline,
    Undecided,
    Release,
    Lease,
    Reclaim,
    Revoke,
    Knock,
}

impl Act {
    pub const ALL: [Act; 12] = [
        Act::Enroll,
        Act::Canon,
        Act::Propose,
        Act::Endorse,
        Act::Accept,
        Act::Decline,
        Act::Undecided,
        Act::Release,
        Act::Lease,
        Act::Reclaim,
        Act::Revoke,
        Act::Knock,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Act::Enroll => "enroll",
            Act::Canon => "canon",
            Act::Propose => "propose",
            Act::Endorse => "endorse",
            Act::Accept => "accept",
            Act::Decline => "decline",
            Act::Undecided => "undecided",
            Act::Release => "release",
            Act::Lease => "lease",
            Act::Reclaim => "reclaim",
            Act::Revoke => "revoke",
            Act::Knock => "knock",
        }
    }

    pub fn parse(s: &str) -> Option<Act> {
        Act::ALL.iter().copied().find(|a| a.name() == s)
    }
}

#[derive(Clone, Debug)]
pub struct Row {
    pub id: u64,
    pub tick: u64,
    pub act: Act,
    pub actor: String,
    pub about: Option<u64>,
    pub body: String,
}

impl Row {
    fn line(&self) -> String {
        format!(
            "{}\t{}\t{}\t{}\t{}\t{}",
            self.id,
            self.tick,
            self.act.name(),
            self.actor,
            self.about.map(|a| a.to_string()).unwrap_or_else(|| "-".into()),
            self.body
        )
    }

    fn parse(line: &str) -> Option<Row> {
        let mut f = line.split('\t');
        let id = f.next()?.parse().ok()?;
        let tick = f.next()?.parse().ok()?;
        let act = Act::parse(f.next()?)?;
        let actor = f.next()?.to_string();
        let about = match f.next()? {
            "-" => None,
            s => Some(s.parse().ok()?),
        };
        let body = f.next().unwrap_or("").to_string();
        Some(Row { id, tick, act, actor, about, body })
    }
}

pub struct Ledger {
    rows: Vec<Row>,
    sink: Option<File>,
    unreadable: usize,
}

impl Ledger {
    /// Replay whatever the file holds, then hold it open for append.
    /// With no path the ledger lives in memory only.
    pub fn open(path: Option<&Path>) -> io::Result<Ledger> {
        let mut rows = Vec::new();
        let mut unreadable = 0;
        let sink = match path {
            None => None,
            Some(p) => {
                if p.exists() {
                    for line in BufReader::new(File::open(p)?).lines() {
                        let line = line?;
                        if line.is_empty() {
                            continue;
                        }
                        match Row::parse(&line) {
                            Some(r) => rows.push(r),
                            None => unreadable += 1,
                        }
                    }
                }
                Some(OpenOptions::new().create(true).append(true).open(p)?)
            }
        };
        Ok(Ledger { rows, sink, unreadable })
    }

    /// The only write. Returns the new row's id.
    pub fn append(
        &mut self,
        tick: u64,
        act: Act,
        actor: &str,
        about: Option<u64>,
        body: &str,
    ) -> io::Result<u64> {
        let id = self.rows.iter().map(|r| r.id).max().unwrap_or(0) + 1;
        let clean = |s: &str| s.replace(|c: char| c == '\t' || c == '\n' || c == '\r', " ");
        let row = Row {
            id,
            tick,
            act,
            actor: clean(actor),
            about,
            body: clean(body),
        };
        if let Some(f) = self.sink.as_mut() {
            writeln!(f, "{}", row.line())?;
            f.flush()?;
        }
        self.rows.push(row);
        Ok(id)
    }

    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    /// The rows as they stood when `id` entered: everything with a smaller id.
    pub fn before(&self, id: u64) -> impl Iterator<Item = &Row> {
        self.rows.iter().filter(move |r| r.id < id)
    }

    pub fn unreadable(&self) -> usize {
        self.unreadable
    }
}
