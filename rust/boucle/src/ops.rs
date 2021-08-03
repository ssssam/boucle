use std::fmt;
use std::num;

pub trait Op: fmt::Debug {
}

#[derive(Debug)]
pub struct OpSpan {
    start: u32,
    duration: u32,
}

#[derive(Debug)]
pub struct ReverseOp {
    span: OpSpan,
}

#[derive(Debug)]
pub struct AbsoluteJumpOp {
    span: OpSpan,
    absolute_position: u32,
}

#[derive(Debug)]
pub struct RelativeJumpOp {
    span: OpSpan,
    relative_position: u32,
}

#[derive(Debug)]
pub struct LoopInLoopOp {
    span: OpSpan,
    loop_size: u32,
}

#[derive(Debug)]
pub struct SpeedRampOp {
    span: OpSpan,
    start_speed: f32,
    end_speed: f32,
}

impl Op for ReverseOp { }
impl Op for AbsoluteJumpOp { }
impl Op for RelativeJumpOp { }
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

pub fn new_from_string(line: &str) -> Result<Box<dyn Op>, ParseError>  {
    let parts: Vec<&str> = line.split_ascii_whitespace().collect();

    let start = parts[1].parse::<u32>()?;
    let duration = parts[2].parse::<u32>()?;
    let span = OpSpan { start: start, duration: duration };

    match parts[0] {
        "reverse" => {
          Ok(Box::new(ReverseOp { span: span }))
        },
        "absolute-jump" => {
          let absolute_position = parts[3].parse::<u32>()?;
          Ok(Box::new(AbsoluteJumpOp { span: span, absolute_position: absolute_position }))
        },
        "relative-jump" => {
          let relative_position = parts[3].parse::<u32>()?;
          Ok(Box::new(RelativeJumpOp { span: span, relative_position: relative_position }))
        },
        "loop_in_loop" => {
          let loop_size = parts[3].parse::<u32>()?;
          Ok(Box::new(LoopInLoopOp { span: span, loop_size: loop_size }))
        },
        "speed-ramp" => {
          let start_speed = parts[3].parse::<f32>()?;
          let end_speed = parts[4].parse::<f32>()?;

          Ok(Box::new(SpeedRampOp {
              span: span,
              start_speed: start_speed,
              end_speed: end_speed
          }))
        },
        _ => {
          Err(ParseError { message: format!("unknown operation '{}'", parts[0]) })
        }
    }
}
