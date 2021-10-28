use dasp::{Sample};
use hound;
use log::*;

use boucle;
use boucle::buffers::{Buffer, LoopBuffers};
use crate::app_error::*;

pub fn input_wav_to_buffer(audio_in_path: &str, buffers: &mut LoopBuffers) -> Result<(), AppError> {
    let reader = hound::WavReader::open(audio_in_path)?;
    let spec = reader.spec();

    if spec.channels != 1 {
        panic!("Input WAV file must be mono (got {} channels", spec.channels);
    }

    info!("Read input {}: {:?}", audio_in_path, spec);
    let wav_buffer: Buffer = match spec.sample_format {
        hound::SampleFormat::Int => {
            let samples = reader
                .into_samples()
                .filter_map(Result::ok);
            samples.map(|s: i32| s.to_sample::<boucle::Sample>()).collect()
        },
        hound::SampleFormat::Float => {
            let samples = reader
                .into_samples()
                .filter_map(Result::ok);
            samples.map(|s: f32| s.to_sample::<boucle::Sample>()).collect()
        },
    };

    for i in 0..buffers.input_a.len() {
        // Sin wave
        //buffer[i] = f32::sin((i as f32) / 10.0) * 0.2;
        if i < wav_buffer.len() {
            buffers.input_a[i] = wav_buffer[i];
            buffers.input_b[i] = wav_buffer[i];
        } else {
            buffers.input_a[i] = 0.0;
            buffers.input_b[i] = 0.0;
        }
    };

    return Ok(());
}
