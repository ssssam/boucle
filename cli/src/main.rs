use boucle;
use boucle::op_sequence;
use boucle::Sample;
use boucle::OpSequence;

use hound;

use std::env;
use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Write;

fn parse_args(args: &[String]) -> Result<(Box<dyn Read>, Box<dyn Read>, Box<dyn Write>), io::Error> {
    let operations = Box::new(File::open(args.get(1).expect("No operations file given"))?);

    let audio_in: Result<Box<dyn Read>, io::Error> = args.get(2).map_or(
        Ok(Box::new(io::stdin())),
        |name| Ok(Box::new(File::open(name)?)));

    let audio_out: Result<Box<dyn Write>, io::Error> = args.get(3).map_or(
        Ok(Box::new(io::stdout())),
        |name| Ok(Box::new(File::create(name)?)));

    if audio_in.is_err() {
        return Err(audio_in.err().unwrap());
    }

    if audio_out.is_err() {
        return Err(audio_out.err().unwrap());
    }

    Ok((operations, audio_in.unwrap(), audio_out.unwrap()))
}

fn read_ops(file: &mut dyn Read) -> Result<OpSequence, io::Error> {
    let mut text = String::new();
    let mut op_sequence = OpSequence::new();
    file.read_to_string(&mut text)?;
    for line in text.lines() {
        let (start, duration, op) = boucle::ops::new_from_string(line).expect("Failed to parse line");
        op_sequence.push(op_sequence::Entry { start, duration, op });
    }
    return Ok(op_sequence);
}

fn main() {
    println!("Boucle looper");

    let args: Vec<String> = env::args().collect();

    let (mut operations_file, audio_in, _audio_out) = parse_args(&args).expect("Failed to open args");

    let op_sequence = read_ops(&mut operations_file).expect("Failed to read ops");
    for op in &op_sequence {
        println!("{}", op);
    }

    println!("Reading input...");
    let mut reader = hound::WavReader::new(io::BufReader::new(audio_in)).unwrap(); //expect("Failed to read input");
    let spec = reader.spec();

    if spec.channels != 1 {
        panic!("Input WAV file must be mono (got {} channels", spec.channels);
    }

    let buffer: Vec<Sample> = reader.samples::<Sample>().map(|s| s.unwrap()).collect();

    let mut writer = hound::WavWriter::create("output.wav", spec).unwrap();

    let boucle: boucle::Boucle = boucle::Boucle::new(boucle::Config::default());
    boucle.process_buffer(&buffer, &op_sequence, &mut |s| writer.write_sample(s).unwrap());
    writer.finalize().unwrap();
}
