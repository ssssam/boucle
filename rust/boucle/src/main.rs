mod ops;

use hound;

use std::env;
use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Write;

/* IDEA:
 *
 * On commandline, you can pass .wav file and a list of operations
 * like this:
 *
 * TYPE:start(ms):duration(ms):params
 * e.g.
 */

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

fn read_ops(file: &mut dyn Read) -> Result<String, io::Error> {
    let mut text = String::new();
    file.read_to_string(&mut text)?;
    for line in text.lines() {
        let op: Box<dyn ops::Op> = ops::new_from_string(line).expect("Failed to parse line");
        println!("{} = {:?}", line, op);
    }
    return Ok(text);
}

fn process(samples: Vec<i32>) {
    let frames_per_block = 16;
    println!("Buffer is {} samples long", samples.len());
    let blocks = samples.chunks(frames_per_block);
    // FIXME: not handling frames yet, so this will only work for mono samples
    blocks.for_each(|block| {
        println!("Took {} samples", block.len());
    });
}

fn main() {
    println!("Boucle looper");

    let args: Vec<String> = env::args().collect();

    let (mut operations, audio_in, audio_out) = parse_args(&args).expect("Failed to open args");

    let ops = read_ops(&mut operations).expect("Failed to read ops");
    println!("ops: {}", ops);

    println!("Reading input...");
    let mut reader = hound::WavReader::new(io::BufReader::new(audio_in)).unwrap(); //expect("Failed to read input");

    let buffer: Vec<i32> = reader.samples::<i32>().map(|s| s.unwrap()).collect();
    process(buffer);
}
