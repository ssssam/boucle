pub mod ops;
pub mod op_sequence;
pub mod patterns;
mod tests;

pub use op_sequence::OpSequence;

// This is the sample format used inside the audio engine.
pub type Sample = i32;

pub type Buffer = Vec<Sample>;

pub type PositionInSamples = usize;
pub type PositionInBlocks = usize;
pub type OffsetInBlocks = i32;

pub struct Config {
    pub frames_per_block: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            frames_per_block: 16
        }
    }
}

pub struct Boucle {
    pub config: Config
}

impl Boucle {
    pub fn new(config: Config) -> Boucle {
        return Boucle { config: config }
    }

    pub fn process_block(self: &Boucle, buffer: &[Sample], op_sequence: &OpSequence, position: PositionInSamples) -> Vec<Sample> {
        println!("Processing block at position {}", position);

        let mut block_start = position;
        let mut block_end = position + self.config.frames_per_block;

        let block_position: PositionInBlocks = position / self.config.frames_per_block;

        for entry in op_sequence {
            if op_sequence::op_in_block(entry, block_position) {
                entry.op.transform_position(&mut block_start, &mut block_end,
                                            entry.start * self.config.frames_per_block,
                                            buffer.len())
            }
        }

        let block_length = block_end - block_start;
        let mut block: Vec<Sample> = vec![0; block_length];
        println!("  copy {}..{}", block_start, block_end);
        block.copy_from_slice(&buffer[block_start..block_end]);

        for entry in op_sequence {
            if op_sequence::op_in_block(entry, block_position) {
                entry.op.transform_block(&mut block)
            }
        }

        return block;
    }

    pub fn process_buffer(self: &Boucle, buffer: &[Sample], ops: &OpSequence, write_sample: &mut dyn FnMut(Sample)) {
        let buffer_size = buffer.len();
        println!("Buffer is {} samples long, {} frames per block", buffer_size, self.config.frames_per_block);

        if buffer_size % self.config.frames_per_block != 0 {
            panic!("Buffer size must be a multiple of block size")
        }

        let mut position = 0;
        while position < buffer_size {
            let block = self.process_block(&buffer, ops, position);
            position += self.config.frames_per_block;

            for s in block {
                write_sample(s);
            }
        }
    }
}
