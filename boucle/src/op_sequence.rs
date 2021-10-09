use crate::ops::Op;

use std::fmt;
use std::time::Duration;
use std::time::Instant;

#[derive(Clone)]
#[derive(Debug)]
pub struct Entry {
    pub start: Instant,
    pub duration: Option<Duration>,
    pub op: Box<dyn Op + Send>,
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let end = match self.duration {
            Some(duration) => format!("{:#?}", duration),
            None => format!("∞"),
        };
        return write!(f, "({:#?}->{}): {:?}", self.start, end, self.op);
    }
}

pub type OpSequence = Vec<Entry>;

pub fn op_active(entry: &Entry, loop_start: Instant, clock: Instant) -> bool {
    let started = clock >= entry.start;
    let finished = match entry.duration {
        Some(duration) => clock >= (entry.start.checked_add(duration).unwrap()),
        None => false,
    };
    return started && !finished;
}
