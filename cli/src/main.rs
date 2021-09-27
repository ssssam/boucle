use boucle;
use boucle::op_sequence;
use boucle::Sample;
use boucle::OpSequence;

use clap::{Arg, App};
use hound;

use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Write;

fn read_ops(file_name: &str) -> Result<OpSequence, io::Error> {
    let mut text = String::new();
    let mut op_sequence = OpSequence::new();
    let mut file = File::open(file_name)?;
    file.read_to_string(&mut text)?;
    for line in text.lines() {
        let (start, duration, op) = boucle::ops::new_from_string(line).expect("Failed to parse line");
        op_sequence.push(op_sequence::Entry { start, duration, op });
    }
    return Ok(op_sequence);
}

fn main() {
    let matches = App::new("Boucle looper")
        .version("1.0")
        .arg(Arg::with_name("INPUT")
             .required(true)
             .index(1))
        .arg(Arg::with_name("OUTPUT")
             .required(true)
             .index(2))
        .get_matches();

    let audio_in = matches.value_of("INPUT").unwrap();
    let audio_out = matches.value_of("OUTPUT").unwrap();

    let operations_file = "ops.test";

    let op_sequence = read_ops(&operations_file).expect("Failed to read ops");
    for op in &op_sequence {
        println!("{}", op);
    }

    println!("Reading input...");
    let mut reader = hound::WavReader::open(audio_in).unwrap(); //expect("Failed to read input");
    let spec = reader.spec();

    if spec.channels != 1 {
        panic!("Input WAV file must be mono (got {} channels", spec.channels);
    }

    let buffer: Vec<Sample> = reader.samples::<Sample>().map(|s| s.unwrap()).collect();

    let mut writer = hound::WavWriter::create(audio_out, spec).unwrap();

    let boucle: boucle::Boucle = boucle::Boucle::new(boucle::Config::default());
    boucle.process_buffer(&buffer, &op_sequence, &mut |s| writer.write_sample(s).unwrap());
    writer.finalize().unwrap();
}
