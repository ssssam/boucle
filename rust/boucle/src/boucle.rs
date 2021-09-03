use crate::ops::*;

pub type Sample = i32;

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

    pub fn process_block(self: &Boucle, buffer: &[Sample], ops: &OpSequence, position: usize) -> Vec<Sample> {
        println!("Processing block at position {}", position);

        // Identity
        //let block = &buffer[position..position+FRAMES_PER_BLOCK];

        // Jump
        // let block = &buffer[position+offset..position+offset+FRAMES_PER_BLOCK];

        // Reverse op
        let reverse_position = buffer.len() - position;

        let mut block = vec![1; self.config.frames_per_block];
        block.copy_from_slice(&buffer[reverse_position-self.config.frames_per_block..reverse_position]);
        block.reverse();

        return block;
    }

    pub fn process_buffer(self: &Boucle, buffer: &[Sample], ops: &OpSequence, write_sample: &mut dyn FnMut(Sample)) {
        let buffer_size = buffer.len();
        println!("Buffer is {} samples long, {} frames per block", buffer_size, self.config.frames_per_block);

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
