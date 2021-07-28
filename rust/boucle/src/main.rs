mod ops;

use std::env;
use std::fs::File;
use std::io;
use std::io::Read;

/* IDEA:
 *
 * On commandline, you can pass .wav file and a list of operations
 * like this:
 *
 * TYPE:start(ms):duration(ms):params
 * e.g.
 */

fn parse_args(args: &[String]) -> (&str, &str, &str) {
    let operations_file = args.get(1).expect("No operations file given");
    let audio_in = match args.get(2) {
        None => "stdin",
        Some(name) => name,
    };
    let audio_out = match args.get(2) {
        None => "stdout",
        Some(name) => name,
    };
    return (operations_file, audio_in, audio_out);
}

fn read_ops_from_file(filename: &str) -> Result<String, io::Error> {
    let mut file: File = File::open(filename)?;
    let mut text = String::new();
    file.read_to_string(&mut text)?;
    for line in text.lines() {
        let op: Box<dyn ops::Op> = ops::new_from_string(line);
        println!("{} = {:?}", line, op);
    }
    return Ok(text);
}

fn main() {
    println!("Boucle looper");

    let args: Vec<String> = env::args().collect();

    let (operations_file, audio_in, audio_out) = parse_args(&args);

    println!("operations_file: {}", operations_file);
    println!("audio_in: {}", audio_in);
    println!("audio_out: {}", audio_out);

    let ops = read_ops_from_file(operations_file).expect("Failed to read ops");
    println!("ops: {}", ops);
}
