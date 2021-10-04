use crate::Sample;
use crate::SamplePosition;
use crate::SampleOffset;

use std::convert::TryFrom;
use std::fmt;
use std::num;

pub trait Op: fmt::Debug {
    // Return a +/- delta that will be applied to `play_clock` to represent this operation.
    fn get_transform(self: &Self,
                     _play_clock: SamplePosition,
                     _op_start: SamplePosition,
                     _loop_length: SamplePosition) -> isize {
        // Identity transform
        return 0;
    }
}

#[derive(Debug)]
pub struct ReverseOp { }

#[derive(Debug)]
pub struct JumpOp {
    pub offset: SampleOffset,
}

#[derive(Debug)]
pub struct RepeatOp {
    pub loop_size: SamplePosition,
}

#[derive(Debug)]
pub struct SpeedRampOp {
    start_speed: f32,
    end_speed: f32,
}

impl Op for JumpOp {
    fn get_transform(self: &Self,
                          play_clock: SamplePosition,
                          op_start: SamplePosition,
                          loop_length: SamplePosition) -> SampleOffset {
        return self.offset;
    }
}

impl Op for ReverseOp {
    fn get_transform(self: &Self,
                     play_clock: SamplePosition,
                     op_start: SamplePosition,
                     loop_length: SamplePosition) -> SampleOffset {
        let op_active_time = play_clock - op_start;
        let transform = -(op_active_time as SampleOffset) * 2;
        println!("reverse-op({}): clock {}, active time = {}, transform {}", op_start, play_clock, op_active_time, transform);
        return transform;
    }
}

impl Op for RepeatOp {
    fn get_transform(self: &Self,
                     play_clock: SamplePosition,
                     op_start: SamplePosition,
                     loop_length: SamplePosition) -> SampleOffset {
        // Samples since operation started
        let delta = play_clock - op_start;
        // Times the inner loop has repeated
        let cycle_count = ((delta as f64) / (self.loop_size as f64)).floor() as SampleOffset;
        // Offset within current inner loop
        let inner_loop_size = self.loop_size as SampleOffset;
        let mut offset = 0;
        if cycle_count > 0 {
            offset = (cycle_count) * inner_loop_size;
        }
        let transform: SampleOffset = -offset as SampleOffset;
        println!("repeat-op: delta {}, inner loop size {}: cycle count {}, offset {}, tf {}",
                 delta, self.loop_size, cycle_count, offset, transform);
        return transform;
    }
}

impl Op for SpeedRampOp { }

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

pub fn new_from_string(line: &str) -> Result<(SamplePosition, SamplePosition, Box<dyn Op + Send>), ParseError> {
    let parts: Vec<&str> = line.split_ascii_whitespace().collect();

    let start = parts[1].parse::<SamplePosition>()?;
    let duration = parts[2].parse::<SamplePosition>()?;

    match parts[0] {
        "reverse" => {
          Ok((start, duration, Box::new(ReverseOp {})))
        },
        "jump" => {
          let offset = parts[3].parse::<SampleOffset>()?;
          Ok((start, duration, Box::new(JumpOp { offset: offset })))
        },
        "repeat" => {
          let loop_size = parts[3].parse::<SamplePosition>()?;
          Ok((start, duration, Box::new(RepeatOp { loop_size: loop_size })))
        },
        "speed-ramp" => {
          let start_speed = parts[3].parse::<f32>()?;
          let end_speed = parts[4].parse::<f32>()?;

          Ok((start, duration, Box::new(SpeedRampOp {
              start_speed: start_speed,
              end_speed: end_speed
          })))
        },
        _ => {
          Err(ParseError { message: format!("unknown operation '{}'", parts[0]) })
        }
    }
}
