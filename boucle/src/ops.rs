use crate::Sample;
use crate::SamplePosition;
use crate::SampleOffset;

use std::convert::TryFrom;
use std::fmt;
use std::num;

pub trait Op: fmt::Debug {
    // Identity transforms.
    fn transform_position(self: &Self,
                          _position: &mut SamplePosition,
                          _op_start: SamplePosition,
                          _op_end: SamplePosition,
                          _buffer_end: SamplePosition) {}
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

impl Op for ReverseOp {
    fn transform_position(self: &Self,
                          position: &mut SamplePosition,
                          op_start: SamplePosition,
                          op_end: SamplePosition,
                          _buffer_end: SamplePosition) {
        // Mirror position within operation boundary.
        assert!(op_start <= *position);
        assert!(op_end >= *position);
        *position = op_end - (*position - op_start);
        println!("reverse-op({}-{}): Position now {} )", op_start, op_end, *position);
    }
}

// From https://stackoverflow.com/questions/54035728/how-to-add-a-negative-i32-number-to-an-usize-variable/54035801
fn add(u: usize, i: i32) -> usize {
    if i.is_negative() {
        u - i.wrapping_abs() as u32 as usize
    } else {
        u + i as usize
    }
}

impl Op for JumpOp {
    fn transform_position(self: &Self,
                          position: &mut SamplePosition,
                          op_start: SamplePosition,
                          op_end: SamplePosition,
                          buffer_end: SamplePosition) {
        *position = add(*position, self.offset) % buffer_end;
        println!("jump-op({},{}): Position now {}", op_start, op_end, *position);
    }
}

impl Op for RepeatOp {
    fn transform_position(self: &Self,
                          position: &mut SamplePosition,
                          op_start: SamplePosition,
                          op_end: SamplePosition,
                          _buffer_end: SamplePosition) {
        let samples_since_loop_started = *position - op_start;
        let offset = if samples_since_loop_started >= self.loop_size {
            samples_since_loop_started - (samples_since_loop_started % self.loop_size)
        } else {
            0
        };
        println!("repeat-op: {} samples since loop started, loop size {}: go back {}",
                 samples_since_loop_started, self.loop_size, offset);

        *position = *position - offset;
        println!("repeat-op({},{}): Position now {}", op_start, op_end, *position);
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

pub fn new_from_string(line: &str) -> Result<(SamplePosition, SamplePosition, Box<dyn Op>), ParseError> {
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
