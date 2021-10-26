use crate::BeatFraction;
use crate::SamplePosition;
use crate::SampleOffset;

use std::fmt;
use std::num;

use log::*;


#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
pub enum Operation {
    NoOp,
    Reverse,
    Repeat { loop_size: BeatFraction },
    Jump { offset: BeatFraction },
    SpeedRamp { start_speed: f32, end_speed: f32 },
}

// Return a +/- delta that will be applied to `play_clock` to represent given operation.
pub fn get_transform(op: Operation,
                     beat_fraction_to_samples: f32,
                     play_clock: SamplePosition,
                     op_start: SamplePosition,
                     _loop_length: SamplePosition) -> isize {
    match op {
        Operation::NoOp => 0,

        Operation::Jump { offset } => offset.as_sample_offset(beat_fraction_to_samples),

        Operation::Reverse => {
            let op_active_time = play_clock - op_start;
            let transform = -(op_active_time as SampleOffset) * 2;
            debug!("reverse-op({}): clock {}, active time = {}, transform {}", op_start, play_clock, op_active_time, transform);
            transform
        },

        Operation::Repeat { loop_size } => {
            // Samples since operation started
            let delta = play_clock - op_start;
            // Times the inner loop has repeated
            let cycle_count: usize = delta / loop_size.as_sample_position(beat_fraction_to_samples);
            // Offset within current inner loop
            let inner_loop_size = loop_size.as_sample_position(beat_fraction_to_samples);
            let mut offset: SampleOffset = 0;
            if cycle_count > 0 {
                offset = (cycle_count * inner_loop_size) as SampleOffset;
            }
            let transform: SampleOffset = -offset as SampleOffset;
            debug!("repeat-op: delta {}, inner loop size {}: cycle count {}, offset {}, tf {}",
                   delta, loop_size, cycle_count, offset, transform);
            transform
        },

        // Not implemented
        Operation::SpeedRamp { .. } => 0,
    }
}

#[derive(Debug)]
pub struct ParseError {
    message: String
}

impl fmt::Display for ParseError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Parse error: {}", self.message)
  }
}

impl From<num::ParseFloatError> for ParseError {
  fn from(error: num::ParseFloatError) -> Self {
    ParseError { message: error.to_string() }
  }
}

impl From<num::ParseIntError> for ParseError {
  fn from(error: num::ParseIntError) -> Self {
    ParseError { message: error.to_string() }
  }
}

pub fn new_from_string(line: &str) -> Result<(f64, f64, Operation), ParseError> {
    let parts: Vec<&str> = line.split_ascii_whitespace().collect();

    let start = parts[1].parse::<f64>()?;
    let duration = parts[2].parse::<f64>()?;

    match parts[0] {
        "reverse" => {
          Ok((start, duration, Operation::Reverse))
        },
        "jump" => {
          let offset = parts[3].parse::<f32>()?;
          Ok((start, duration, Operation::Jump {
              offset: BeatFraction::from(offset)
          }))
        },
        "repeat" => {
          let loop_size = parts[3].parse::<f32>()?;
          Ok((start, duration, Operation::Repeat {
              loop_size: BeatFraction::from(loop_size)
          }))
        },
        "speed-ramp" => {
          let start_speed = parts[3].parse::<f32>()?;
          let end_speed = parts[4].parse::<f32>()?;

          Ok((start, duration, Operation::SpeedRamp {
              start_speed, end_speed
          }))
        },
        _ => {
          Err(ParseError { message: format!("unknown operation '{}'", parts[0]) })
        }
    }
}
