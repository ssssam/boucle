#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::run_batch;

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

    #[test]
    fn test_batch_i16() {
        let ops_path = get_test_data_path("ops.test");
        let input_path = get_test_data_path("chirp.i16.wav");
        run_batch(&input_path, "/tmp/foo.wav", &ops_path);
    }

    #[test]
    fn test_batch_f32() {
        let ops_path = get_test_data_path("ops.test");
        let input_path = get_test_data_path("chirp.f32.wav");
        run_batch(&input_path, "/tmp/foo.wav", &ops_path);
    }
}
