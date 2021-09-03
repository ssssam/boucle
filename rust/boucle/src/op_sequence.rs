use crate::boucle::PositionInBlocks;
use crate::ops::Op;

#[derive(Debug)]
pub struct Entry {
    pub start: PositionInBlocks,
    pub duration: PositionInBlocks,
    pub op: Box<dyn Op>,
}

pub type OpSequence = [Entry];
pub type OpSequenceVec = Vec<Entry>;

pub fn op_in_block(entry: &Entry, block_position: PositionInBlocks) -> bool {
    let result = block_position >= entry.start && block_position < (entry.start + entry.duration);
    println!("op_in_block: entry ({},{}) block {}: {}", entry.start, entry.duration, block_position, result);
    return result;
}
