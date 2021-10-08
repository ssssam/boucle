pub mod ops;
pub mod op_sequence;
pub mod patterns;
pub mod piano_control;
mod tests;

use std::convert::TryInto;
use std::time::{Duration, Instant};

use log::*;

pub use op_sequence::OpSequence;
pub use piano_control::PianoControl;

// This is the sample format used inside the audio engine.
pub type Sample = f32;

pub type Buffer = Vec<Sample>;

pub type SamplePosition = usize;
pub type SampleOffset = isize;

pub struct Config {
    pub sample_rate: u64,
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
    pub sample_rate: u64,
    pub start_time: Instant,
}

fn duration_as_samples(duration: Duration, sample_rate: u64) -> SamplePosition {
    return ((duration.as_nanos() * (sample_rate as u128)) / 1000000000) as usize;
}

impl Boucle {
    pub fn new(config: &Config) -> Boucle {
        return Boucle {
            controller: PianoControl::new(config.beats_to_samples),
            sample_rate: config.sample_rate,
            start_time: Instant::now(),
        }
    }

    pub fn set_start_time(self: &mut Self, start_time: Instant) {
        self.start_time = start_time;
    }

    pub fn next_sample(self: &Boucle, loop_buffer: &[Sample], op_sequence: &OpSequence, play_clock: Instant) -> Sample {
        let loop_length = loop_buffer.len();
        let play_clock_samples: SamplePosition = duration_as_samples(play_clock - self.start_time, self.sample_rate);
        let mut transformed_clock: SampleOffset = play_clock_samples.try_into().unwrap();

        for entry in op_sequence {
            if op_sequence::op_active(entry, self.start_time, play_clock) {
                let op_start_samples: SamplePosition = duration_as_samples(entry.start - self.start_time, self.sample_rate);
                let transform = entry.op.get_transform(play_clock_samples, op_start_samples, loop_length);
                transformed_clock += transform;
            }
        }

        let mut loop_position = 0;
        if transformed_clock < 0 {
            debug!("transforming {}", transformed_clock.saturating_abs() as usize%loop_length);
            loop_position = (loop_length - ((transformed_clock.saturating_abs() as SamplePosition) % loop_length)) % loop_length;
        } else {
            loop_position = (transformed_clock as SamplePosition) % loop_length;
        }

        debug!("start time {:?}, clock time {:?}, transformed from {:?} to {:?}, loop pos {:?}", self.start_time, play_clock, play_clock_samples, transformed_clock, loop_position);
        return loop_buffer[loop_position];
    }

    pub fn process_buffer(self: &Boucle,
                          loop_buffer: &[Sample],
                          play_clock: Instant,
                          out_buffer_length: SamplePosition,
                          ops: &OpSequence,
                          write_sample: &mut dyn FnMut(Sample)) {
        let loop_length = loop_buffer.len();
        info!("Buffer is {:#?} samples long, playing at {:?} for {:#?}", loop_length, play_clock, out_buffer_length);

        for sample in 0..out_buffer_length {
            let sample_time: Duration = Duration::from_nanos((sample as u64) * 1000000000 / self.sample_rate);
            //debug!("sample {}: time {:?}", sample, sample_time);
            let s = self.next_sample(&loop_buffer, ops, play_clock + sample_time);
            write_sample(s);
        }
    }
}
