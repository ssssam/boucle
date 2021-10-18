use std::io;
use std::io::Read;
use std::fs::File;

use dasp::{Sample};
use log::*;

use boucle::op_sequence;
use boucle::OpSequence;

use crate::app_config::AppConfig;
use crate::buffers::{create_buffers, input_wav_to_buffer};

fn read_ops(sample_rate: u32, file_name: &str) -> Result<OpSequence, io::Error> {
    let mut text = String::new();
    let mut op_sequence = OpSequence::new();
    let mut file = File::open(file_name)?;
    file.read_to_string(&mut text)?;
    for line in text.lines() {
        let (start_seconds, duration_seconds, op) = boucle::ops::new_from_string(line).expect("Failed to parse line");
        op_sequence.push(op_sequence::Entry {
            start: (start_seconds * sample_rate as f64) as usize,
            duration: Some((duration_seconds * sample_rate as f64) as usize),
            op
        });
    }
    return Ok(op_sequence);
}

pub fn run_batch(config: &AppConfig, audio_in_path: &str, audio_out: &str, operations_file: &str) {
    let op_sequence = read_ops(config.sample_rate, &operations_file).expect("Failed to read ops");
    for op in &op_sequence {
        debug!("{}", op);
    }

    let buffer_size_samples: usize = (config.loop_time * config.sample_rate as f32)
        .floor() as usize;

    let mut buffers = create_buffers(buffer_size_samples);

    input_wav_to_buffer(audio_in_path, &mut buffers).expect("Failed to read input");

    let out_spec = hound::WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int
    };
    let mut writer = hound::WavWriter::create(audio_out, out_spec).unwrap();

    let boucle: boucle::Boucle = boucle::Boucle::new(&boucle::Config::default());
    boucle.process_buffer(&buffers.input_a, 0, buffers.input_a.len(), &op_sequence, &mut |s| {
        let s_i16 = s.to_sample::<i16>();
        writer.write_sample(s_i16).unwrap();
    });
    writer.finalize().unwrap();
}

