pub mod buffers;
pub mod control_surface;
pub mod cpal_helpers;
pub mod event;
pub mod event_recorder;
pub mod ops;
pub mod op_sequence;
pub mod patterns;
pub mod units;
mod tests;

use std::convert::TryInto;

use log::*;

pub use control_surface::midi::MidiControlSurface;
pub use event_recorder::EventRecorder;
pub use ops::Operation;
pub use op_sequence::OpSequence;
pub use units::BeatFraction;
pub use units::Sample;
pub use units::SampleOffset;
pub use units::SamplePosition;

pub struct Config {
    pub sample_rate: u32,
    pub beat_fraction_to_samples: f32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            sample_rate: 44100,
            beat_fraction_to_samples: 44100.0 / 16.0   /* Assumes 1 beat = 1 second at 44.1KHz */
        }
    }
}

pub struct Boucle {
    pub event_recorder: EventRecorder,
    pub sample_rate: u32,
    pub beat_fraction_to_samples: f32,
    pub loop_length: SamplePosition,
}

impl Boucle {
    pub fn new(config: &Config, loop_length: SamplePosition) -> Boucle {
        return Boucle {
            event_recorder: EventRecorder::new(config.sample_rate),
            sample_rate: config.sample_rate,
            beat_fraction_to_samples: config.beat_fraction_to_samples,
            loop_length: loop_length,
        }
    }

    // When increasing loop length, old recordings may play from the buffer.
    // It's up to caller to erase these if desired before updating loop length.
    pub fn set_loop_length(self: &mut Self, loop_length: SamplePosition) {
        self.loop_length = loop_length;
    }

    pub fn loop_length(self: &Boucle) -> SamplePosition {
        return self.loop_length;
    }

    pub fn next_sample(self: &Boucle, loop_buffer: &[Sample], op_sequence: &OpSequence, play_clock: SamplePosition) -> Sample {
        let loop_length = self.loop_length();
        let mut transformed_clock: SampleOffset = play_clock.try_into().unwrap();

        for entry in op_sequence {
            if op_sequence::op_active(entry, play_clock) {
                let transform = ops::get_transform(
                    entry.operation,
                    self.beat_fraction_to_samples,
                    play_clock,
                    entry.start,
                    loop_length
                );
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
        let loop_length = self.loop_length();
        info!("Buffer is {:#?} samples long, loop is {:#?} playing at {:?} for {:#?}",
              loop_buffer.len(), loop_length, play_clock, out_buffer_length);

        for sample in 0..out_buffer_length {
            let s = self.next_sample(&loop_buffer, ops, play_clock + sample);
            write_sample(s);
        }
    }
}
