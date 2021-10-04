pub mod ops;
pub mod op_sequence;
pub mod patterns;
mod tests;

pub use op_sequence::OpSequence;

// This is the sample format used inside the audio engine.
pub type Sample = f32;

pub type Buffer = Vec<Sample>;

pub type SamplePosition = usize;
pub type SampleOffset = i32;

pub struct Config {
    // FIXME: this is a placeholder right now, it has no effect.
    pub sample_rate: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            sample_rate: 44100
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

    pub fn next_sample(self: &Boucle, loop_buffer: &[Sample], op_sequence: &OpSequence, play_position: SamplePosition) -> Sample {
        let mut position = play_position;

        for entry in op_sequence {
            if op_sequence::op_active(entry, position) {
                entry.op.transform_position(&mut position, entry.start, entry.start + entry.duration, loop_buffer.len())
            }
        }

        return loop_buffer[position];
    }

    pub fn process_buffer(self: &Boucle,
                          loop_buffer: &[Sample],
                          play_start: SamplePosition,
                          play_end: SamplePosition,
                          ops: &OpSequence,
                          write_sample: &mut dyn FnMut(Sample)) {
        let buffer_size = loop_buffer.len();
        println!("Buffer is {} samples long, playing {} to {}", buffer_size, play_start, play_end);

        for position in play_start..play_end {
            let s = self.next_sample(&loop_buffer, ops, position % buffer_size);
            write_sample(s);
        }
    }
}
