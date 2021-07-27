enum OpType {
    None = 0,
    Reverse = 1,
    AbsoluteJump = 2,
    RelativeJump = 3,
    LoopInLoop = 4,
    SpeedRamp = 5,
}

struct OpSpan {
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
    loop_size: u32,
}

pub struct SpeedRampOp {
    start_speed: f32,
    end_speed: f32,
}
