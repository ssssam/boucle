pub struct AppConfig {
    pub sample_rate: u32,
    pub loop_time: f32,
}

impl AppConfig {
    pub fn new(sample_rate: u32, loop_time: f32) -> Self {
        AppConfig { sample_rate, loop_time }
    }
}
