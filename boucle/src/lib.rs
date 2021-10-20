pub mod buffers;
pub mod cpal_helpers;
pub mod ops;
pub mod op_sequence;
pub mod patterns;
pub mod piano_control;
mod tests;

use std::convert::TryInto;

use log::*;

pub use op_sequence::OpSequence;
pub use piano_control::PianoControl;

// This is the sample format used inside the audio engine.
pub type Sample = f32;

pub type SamplePosition = usize;
pub type SampleOffset = isize;

pub struct Config {
    pub sample_rate: u32,
    pub beats_to_samples: f32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            sample_rate: 44100,
            beats_to_samples: 44100.0    /* Assumes 1 beat = 1 second at 44.1KHz */
        }
    }
}

pub struct Boucle {
    pub controller: PianoControl,
    pub sample_rate: u32,
}

impl Boucle {
    pub fn new(config: &Config) -> Boucle {
        return Boucle {
            controller: PianoControl::new(config.sample_rate, config.beats_to_samples),
            sample_rate: config.sample_rate,
        }
    }

    pub fn next_sample(self: &Boucle, loop_buffer: &[Sample], op_sequence: &OpSequence, play_clock: SamplePosition) -> Sample {
        let loop_length = loop_buffer.len();
        let mut transformed_clock: SampleOffset = play_clock.try_into().unwrap();

        for entry in op_sequence {
            if op_sequence::op_active(entry, play_clock) {
                let transform = entry.op.get_transform(play_clock, entry.start, loop_length);
                transformed_clock += transform;
            }
        }

        let loop_position;
        if transformed_clock < 0 {
            debug!("transforming {}", transformed_clock.saturating_abs() as usize%loop_length);
            loop_position = (loop_length - ((transformed_clock.saturating_abs() as SamplePosition) % loop_length)) % loop_length;
        } else {
            loop_position = (transformed_clock as SamplePosition) % loop_length;
        }

        return loop_buffer[loop_position];
    }

    pub fn process_buffer(self: &Boucle,
                          loop_buffer: &[Sample],
                          play_clock: SamplePosition,
                          out_buffer_length: SamplePosition,
                          ops: &OpSequence,
                          write_sample: &mut dyn FnMut(Sample)) {
        let loop_length = loop_buffer.len();
        info!("Buffer is {:#?} samples long, playing at {:?} for {:#?}", loop_length, play_clock, out_buffer_length);

        for sample in 0..out_buffer_length {
            let s = self.next_sample(&loop_buffer, ops, play_clock + sample);
            write_sample(s);
        }
    }
}
