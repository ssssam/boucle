#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::path::PathBuf;

    use crate::app_config::AppConfig;
    use crate::cmd_batch::run_batch;

    fn get_test_data_path(filename: &str) -> String {
        let mut path = PathBuf::from(file!());
        path.pop(); path.pop(); path.pop();
        path.push("test_data");
        path.push(filename);
        let abs_path = path.canonicalize().unwrap();
        return abs_path
            .to_str().unwrap()
            .to_string();
    }

    fn get_test_output_path(filename: &str) -> String {
        let mut path = std::env::temp_dir();
        path.push("boucle-tests");
        std::fs::create_dir_all(path.to_str().unwrap()).unwrap();
        path.push(filename);
        return path
            .to_str().unwrap()
            .to_string();
    }

    #[test]
    fn test_batch_i16() {
        let app_config = AppConfig::new(44100, 2.0);
        let ops_path = get_test_data_path("ops.test");
        let input_path = get_test_data_path("chirp.i16.wav");
        let output_path = get_test_output_path("out.i16.wav");
        run_batch(&app_config, &input_path, &output_path, &ops_path);

        assert!(Path::new(&output_path).exists(),
            "Output {} does not exist", output_path);
    }

    #[test]
    fn test_batch_f32() {
        let app_config = AppConfig::new(44100, 2.0);
        let ops_path = get_test_data_path("ops.test");
        let input_path = get_test_data_path("chirp.f32.wav");
        let output_path = get_test_output_path("out.f32.wav");
        run_batch(&app_config, &input_path, &output_path, &ops_path);

        assert!(Path::new(&output_path).exists());
    }
}
