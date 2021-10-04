use crate::SamplePosition;
use crate::ops::Op;

use std::fmt;

#[derive(Debug)]
pub struct Entry {
    pub start: SamplePosition,
    pub duration: Option<SamplePosition>,
    pub op: Box<dyn Op>,
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let end = match self.duration {
            Some(duration) => format!("{}", duration),
            None => format!("∞"),
        };
        return write!(f, "({}->{}): {:?}", self.start, end, self.op);
    }
}

pub type OpSequence = Vec<Entry>;

pub fn op_active(entry: &Entry, position: SamplePosition) -> bool {
    let started = position >= entry.start;
    let finished = match entry.duration {
        Some(duration) => position >= (entry.start + duration),
        None => false,
    };
    return started && !finished;
}
