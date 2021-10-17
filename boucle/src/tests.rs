#[cfg(test)]
mod tests {
    use crate::Boucle;
    use crate::Config;
    use crate::Sample;
    use crate::ops;
    use crate::op_sequence;
    use crate::OpSequence;

    const TEST_CONFIG: Config = Config { sample_rate: 44100, beats_to_samples: 100.0 };

    fn make_buffer(data: &[i16]) -> Vec<Sample> {
        data.iter().map(|s| Sample::from(*s)).collect()
    }

    #[test]
    fn basic_reverse() {
        let boucle: Boucle = Boucle::new(&TEST_CONFIG);
        let input = make_buffer(&[1,2,3,4, 5,6,7,8]);

        let ops: OpSequence = vec!(
            op_sequence::Entry { start: 3, duration: Some(10), op: Box::new(ops::ReverseOp {}) },
        );
        let expected_output = make_buffer(&[1,2,3,4,3,2,1,8,7,6,5,4,3,6,7,8]);

        let mut output: Vec<Sample> = Vec::new();
        boucle.process_buffer(&input, 0, input.len() * 2, &ops, &mut |s| output.push(s));
        assert_eq!(output, expected_output);
    }

    #[test]
    fn basic_jump() {
        let boucle: Boucle = Boucle::new(&TEST_CONFIG);
        let input = make_buffer(&[1,2,3,4, 5,6,7,8, 9,10,11,12, 13,14,15,16]);

        let ops: OpSequence = vec!(
            op_sequence::Entry { start: 4, duration: Some(4), op: Box::new(ops::JumpOp { offset: -4}) },
            op_sequence::Entry { start: 12, duration: Some(4), op: Box::new(ops::JumpOp { offset: 8}) },
        );
        let expected_output = make_buffer(&[1,2,3,4, 1,2,3,4, 9,10,11,12, 5,6,7,8]);

        let mut output: Vec<Sample> = Vec::new();
        boucle.process_buffer(&input, 0, input.len(), &ops, &mut |s| output.push(s));
        assert_eq!(output, expected_output);
    }

    #[test]
    fn basic_repeat() {
        let boucle: Boucle = Boucle::new(&TEST_CONFIG);
        let input = make_buffer(&[1,2,3,4, 5,6,7,8, 9,10,11,12, 13,14,15,16, 17,18,19,20, 21,22,23,24]);

        let ops: OpSequence = vec!(
            op_sequence::Entry { start: 0, duration: Some(20), op: Box::new(ops::RepeatOp { loop_size: 8}) },
        );
        let expected_output = make_buffer(&[1,2,3,4, 5,6,7,8, 1,2,3,4, 5,6,7,8, 1,2,3,4, 21,22,23,24]);

        let mut output: Vec<Sample> = Vec::new();
        boucle.process_buffer(&input, 0, input.len(), &ops, &mut |s| output.push(s));
        assert_eq!(output, expected_output);
    }
}
