use crate::PositionInBlocks;
use crate::ops::Op;

use std::fmt;

#[derive(Debug)]
pub struct Entry {
    pub start: PositionInBlocks,
    pub duration: PositionInBlocks,
    pub op: Box<dyn Op>,
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}->{}): {:?}", self.start, self.start + self.duration, self.op)
    }
}

pub type OpSequence = Vec<Entry>;

pub fn op_in_block(entry: &Entry, block_position: PositionInBlocks) -> bool {
    let result = block_position >= entry.start && block_position < (entry.start + entry.duration);
    println!("op_in_block: entry ({},{}) block {}: {}", entry.start, entry.duration, block_position, result);
    return result;
}
