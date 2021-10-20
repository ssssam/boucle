// Helpers to integrate Boucle core with CPAL audio library.

use cpal::traits::{DeviceTrait, HostTrait};
use log::*;

use crate::Config;

pub fn get_audio_config(lib_config: &Config, device: &cpal::Device) -> cpal::SupportedStreamConfig {
    let mut supported_configs_range = device.supported_output_configs()
        .expect("error while querying configs");
    let supported_config = supported_configs_range.next()
        .expect("no supported config")
        .with_sample_rate(cpal::SampleRate(lib_config.sample_rate));
    info!("audio config: {:?}", supported_config);
    return supported_config;
}

