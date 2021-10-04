use crate::SamplePosition;
use crate::ops::Op;

use std::fmt;

#[derive(Debug)]
pub struct Entry {
    pub start: SamplePosition,
    pub duration: SamplePosition,
    pub op: Box<dyn Op>,
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}->{}): {:?}", self.start, self.start + self.duration, self.op)
    }
}

pub type OpSequence = Vec<Entry>;

pub fn op_active(entry: &Entry, position: SamplePosition) -> bool {
    let result = position >= entry.start && position < (entry.start + entry.duration);
    println!("op_active: entry ({},{}) position {}: {}", entry.start, entry.duration, position, result);
    return result;
}
