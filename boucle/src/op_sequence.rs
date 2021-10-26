use crate::SamplePosition;
use crate::ops::Operation;

use std::fmt;

#[derive(Clone)]
#[derive(Debug)]
pub struct Entry {
    pub start: SamplePosition,
    pub duration: Option<SamplePosition>,
    pub operation: Operation,
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let end = match self.duration {
            Some(duration) => format!("{:#?}", duration),
            None => format!("∞"),
        };
        return write!(f, "({:#?}->{}): {:?}", self.start, end, self.operation);
    }
}

pub type OpSequence = Vec<Entry>;

pub fn op_active(entry: &Entry, clock: SamplePosition) -> bool {
    let started = clock >= entry.start;
    let finished = match entry.duration {
        Some(duration) => clock >= (entry.start.checked_add(duration).unwrap()),
        None => false,
    };
    return started && !finished;
}
