use crate::Sample;
use crate::PositionInSamples;
use crate::PositionInBlocks;

use std::fmt;
use std::num;

pub trait Op: fmt::Debug {
    // Identity transforms.
    fn transform_position(self: &Self,
                          _block_start: &mut PositionInSamples,
                          _block_end: &mut PositionInSamples,
                          _buffer_end: PositionInSamples) {}
    fn transform_block(self: &Self, _block: &mut[Sample]) {}
}

#[derive(Debug)]
pub struct ReverseOp { }

#[derive(Debug)]
pub struct JumpOp {
    pub offset: i32,
}

#[derive(Debug)]
pub struct LoopInLoopOp {
    pub loop_size: u32,
}

#[derive(Debug)]
pub struct SpeedRampOp {
    start_speed: f32,
    end_speed: f32,
}

impl Op for ReverseOp {
    fn transform_position(self: &Self,
                          block_start: &mut PositionInSamples,
                          block_end: &mut PositionInSamples,
                          _buffer_end: PositionInSamples) {
        // Play backwards from block_start.
        let block_length = *block_end - *block_start;
        *block_end = *block_start;
        *block_start = *block_end - block_length;
        println!("reverse-op: Position now ({},{})", *block_start, *block_end);
    }

    fn transform_block(self: &Self, block: &mut[Sample]) {
        block.reverse();
        println!("reverse-op: Reverse block");
    }
}

impl Op for JumpOp { }
impl Op for LoopInLoopOp { }
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

pub fn new_from_string(line: &str) -> Result<(PositionInBlocks, PositionInBlocks, Box<dyn Op>), ParseError> {
    let parts: Vec<&str> = line.split_ascii_whitespace().collect();

    let start = parts[1].parse::<PositionInBlocks>()?;
    let duration = parts[2].parse::<PositionInBlocks>()?;

    match parts[0] {
        "reverse" => {
          Ok((start, duration, Box::new(ReverseOp {})))
        },
        "jump" => {
          let offset = parts[3].parse::<i32>()?;
          Ok((start, duration, Box::new(JumpOp { offset: offset })))
        },
        "loop_in_loop" => {
          let loop_size = parts[3].parse::<u32>()?;
          Ok((start, duration, Box::new(LoopInLoopOp { loop_size: loop_size })))
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
