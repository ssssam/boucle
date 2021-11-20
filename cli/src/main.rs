mod app_config;
mod app_error;
mod cmd_batch;
mod cmd_list_ports;
mod cmd_live;
mod tests;
mod wav;

use clap::{Arg, App};
use log::*;

use crate::app_config::AppConfig;

fn parse_f32_option(string: Option<&str>) -> Option<f32> {
    return match string {
        Some(text) => Some(text.parse::<f32>().unwrap()),
        None => None
    };
}

fn calculate_loop_time(seconds: Option<f32>, beats: Option<f32>, bpm: Option<f32>) -> Result<f32, String> {
    if let Some(value) = seconds {
        return Ok(value);
    } else if let Some(value) = beats {
        if let Some(multiplier) = bpm {
            let loop_seconds: f32 = value * (60.0 / multiplier);
            info!("Loop length: {} * (60.0 / {}) = {}", value, multiplier, loop_seconds);
            return Ok(loop_seconds);
        } else {
            return Err("Loop size in beats requires a BPM".to_string());
        }
    } else {
        return Err("Must specify loop size in either seconds or beats".to_string());
    };
}

fn main() {
    env_logger::init();

    let app_m = App::new("Boucle looper")
        .version("1.0")
        .subcommand(App::new("live")
            .arg(Arg::with_name("input-file")
                 .long("input-file")
                 .short("f")
                 .help("Read loop buffer from FILE")
                 .takes_value(true)
                 .value_name("FILE"))
            .arg(Arg::with_name("input-device")
                 .long("input-device")
                 .short("i")
                 .help("Record audio from device")
                 .takes_value(true)
                 .value_name("NAME"))
            .arg(Arg::with_name("output-device")
                 .long("output-device")
                 .short("o")
                 .help("Play audio to device")
                 .takes_value(true)
                 .value_name("NAME"))
            .arg(Arg::with_name("midi-port")
                 .long("midi-port")
                 .short("p")
                 .help("MIDI port to read from")
                 .takes_value(true)
                 .value_name("PORT"))
            .arg(Arg::with_name("bpm")
                 .long("bpm")
                 .help("Beats per minute")
                 .takes_value(true)
                 .value_name("BPM"))
            .arg(Arg::with_name("loop-time-seconds")
                 .long("loop-time-seconds")
                 .short("s")
                 .help("Loop length, in seconds")
                 .takes_value(true)
                 .value_name("SECONDS"))
            .arg(Arg::with_name("loop-time-beats")
                 .long("loop-time-beats")
                 .short("b")
                 .help("Loop length, in beats (requires `--bpm`)")
                 .takes_value(true)
                 .value_name("BEATS")))
        .subcommand(App::new("batch")
            .arg(Arg::with_name("INPUT")
                 .required(true)
                 .index(1))
            .arg(Arg::with_name("OUTPUT")
                 .required(true)
                 .index(2))
            .arg(Arg::with_name("bpm")
                 .long("bpm")
                 .help("Beats per minute")
                 .takes_value(true)
                 .value_name("BPM"))
            .arg(Arg::with_name("loop-time-seconds")
                 .long("loop-time-seconds")
                 .short("s")
                 .help("Loop length, in seconds")
                 .takes_value(true)
                 .value_name("SECONDS"))
            .arg(Arg::with_name("loop-time-beats")
                 .long("loop-time-beats")
                 .short("b")
                 .help("Loop length, in beats (requires `--bpm`)")
                 .takes_value(true)
                 .value_name("BEATS")))
        .subcommand(App::new("list-ports"))
        .get_matches();


    const SAMPLE_RATE: u32 = 44100;
    match app_m.subcommand() {
        ("batch", Some(sub_m)) => {
            let loop_time_seconds: Option<f32> = parse_f32_option(sub_m.value_of("loop-time-seconds"));
            let loop_time_beats: Option<f32> = parse_f32_option(sub_m.value_of("loop-time-beats"));
            let bpm: Option<f32> = parse_f32_option(sub_m.value_of("bpm"));
            let loop_time = match calculate_loop_time(loop_time_seconds, loop_time_beats, bpm) {
                Ok(value) => value,
                Err(string) => panic!("{}", string),
            };

            let app_config = AppConfig::new(SAMPLE_RATE, loop_time);
            let audio_in = sub_m.value_of("INPUT").unwrap();
            let audio_out = sub_m.value_of("OUTPUT").unwrap();
            let operations_file = "ops.test";
            cmd_batch::run_batch(&app_config, audio_in, audio_out, operations_file);
        },
        ("live", Some(sub_m)) => {
            let loop_time_seconds: Option<f32> = parse_f32_option(sub_m.value_of("loop-time-seconds"));
            let loop_time_beats: Option<f32> = parse_f32_option(sub_m.value_of("loop-time-beats"));
            let bpm: Option<f32> = parse_f32_option(sub_m.value_of("bpm"));
            let loop_time = match calculate_loop_time(loop_time_seconds, loop_time_beats, bpm) {
                Ok(value) => value,
                Err(string) => panic!("{}", string),
            };

            let app_config = AppConfig::new(SAMPLE_RATE, loop_time);
            let midi_port: i32 = sub_m.value_of("midi-port").unwrap_or("0").
                                    parse::<i32>().unwrap();
            let input_file = sub_m.value_of("input-file");
            let input_device_name = sub_m.value_of("input-device");
            let output_device_name = sub_m.value_of("output-device");
            cmd_live::run_live(&app_config, midi_port, input_file, input_device_name, output_device_name, loop_time, bpm.unwrap_or(60.0)).unwrap();
        },
        ("list-ports", Some(_)) => {
            cmd_list_ports::run_list_ports().unwrap();
        },
        _ => {
            println!("{}", app_m.usage());
            println!();
            println!("Run with `--help` to see subcommands.")
        }
    }
}
