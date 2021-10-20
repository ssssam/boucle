use dasp;

pub type Buffer = Vec<crate::Sample>;

#[derive(PartialEq)]
pub enum InputBuffer { A, B }

pub struct LoopBuffers {
    pub input_a: Buffer,
    pub input_b: Buffer,
    pub current_input: InputBuffer,
    pub current_output: InputBuffer,
    pub record_pos: crate::SamplePosition,
    pub play_clock: crate::SamplePosition,
}

pub fn create_buffers(buffer_size_samples: usize) -> LoopBuffers {
    let this = LoopBuffers {
        input_a: vec!(0.0; buffer_size_samples),
        input_b: vec!(0.0; buffer_size_samples),
        current_input: InputBuffer::B,
        current_output: InputBuffer::A,
        record_pos: 0,
        play_clock: 0,
    };
    return this;
}
