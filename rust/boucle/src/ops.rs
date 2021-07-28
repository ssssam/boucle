enum OpType {
    None = 0,
    Reverse = 1,
    AbsoluteJump = 2,
    RelativeJump = 3,
    LoopInLoop = 4,
    SpeedRamp = 5,
}

pub trait Op: std::fmt::Debug {
}

#[derive(Debug)]
pub struct OpSpan {
    start: u32,
    duration: u32,
}

pub struct ReverseOp {
    span: OpSpan,
}

pub struct AbsoluteJumpOp {
    span: OpSpan,
    absolute_position: u32,
}

pub struct RelativeJumpOp {
    span: OpSpan,
    relative_position: u32,
}

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

impl Op for SpeedRampOp {
}

pub fn new_from_string(line: &str) -> Box<dyn Op> {
    return Box::new(SpeedRampOp { span: OpSpan { start: 1, duration: 2 }, start_speed: 0.5, end_speed: 1.0 });
}
