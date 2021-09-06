// VST2 plugin for Boucle.
//
// Following: https://vaporsoft.net/creating-an-audio-plugin-with-rust-vst/

#[macro_use]
extern crate vst;

use vst::api::Events;
use vst::buffer::AudioBuffer;
use vst::event::Event;
use vst::plugin::{Category, Info, Plugin};

#[derive(Default)]
struct BoucleVst;

type VstSample = f32;

impl Plugin for BoucleVst {
    fn get_info(&self) -> Info {
        Info {
            name: "Boucle".to_string(),
            vendor: "Medium Length Life".to_string(),
            unique_id: 42,
            inputs: 2,
            outputs: 2,
            version: 1,
            category: Category::Effect,
            ..Default::default()
        }
    }

    fn process_events(&mut self, events: &Events) {
        for event in events.events() {
            match event {
                Event::Midi(ev) => {
                    println!("Got MIDI event: {}.", ev.data[0]);
                },
                _ => (),
            }
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer<VstSample>) {
        let (input_buffer, mut output_buffer) = buffer.split();

        for output_channel in output_buffer.into_iter() {
            for output_sample in output_channel {
                *output_sample = 0f32;
            }
        }
    }
}

plugin_main!(BoucleVst);
