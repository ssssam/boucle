#[cfg(test)]
mod tests {
    use crate::Boucle;
    use crate::Config;
    use crate::Sample;
    use crate::ops;
    use crate::op_sequence;
    use crate::OpSequence;

    const TEST_CONFIG: Config = Config { frames_per_block: 4 };

    #[test]
    fn basic_reverse() {
        let boucle: Boucle = Boucle::new(TEST_CONFIG);
        let input = vec!(1,2,3,4, 5,6,7,8, 9,10,11,12, 13,14,15,16);

        let ops: OpSequence = vec!(
            op_sequence::Entry { start: 1, duration: 2, op: Box::new(ops::ReverseOp {}) },
        );
        let expected_output = vec!(1,2,3,4, 4,3,2,1, 8,7,6,5, 13,14,15,16);

        let mut output: Vec<Sample> = Vec::new();
        boucle.process_buffer(&input, &ops, &mut |s| output.push(s));
        assert_eq!(output, expected_output);
    }

    #[test]
    fn basic_jump() {
        let boucle: Boucle = Boucle::new(TEST_CONFIG);
        let input = vec!(1,2,3,4, 5,6,7,8, 9,10,11,12, 13,14,15,16);

        let ops: OpSequence = vec!(
            op_sequence::Entry { start: 1, duration: 1, op: Box::new(ops::JumpOp { offset: -1}) },
            op_sequence::Entry { start: 3, duration: 1, op: Box::new(ops::JumpOp { offset: 2}) },
        );
        let expected_output = vec!(1,2,3,4, 1,2,3,4, 9,10,11,12, 5,6,7,8);

        let mut output: Vec<Sample> = Vec::new();
        boucle.process_buffer(&input, &ops, &mut |s| output.push(s));
        assert_eq!(output, expected_output);
    }

    #[test]
    fn basic_repeat() {
        let boucle: Boucle = Boucle::new(TEST_CONFIG);
        let input = vec!(1,2,3,4, 5,6,7,8, 9,10,11,12, 13,14,15,16, 17,18,19,20, 21,22,23,24);

        let ops: OpSequence = vec!(
            op_sequence::Entry { start: 0, duration: 5, op: Box::new(ops::RepeatOp { loop_size: 2}) },
        );
        let expected_output = vec!(1,2,3,4, 5,6,7,8, 1,2,3,4, 5,6,7,8, 1,2,3,4, 21,22,23,24);

        let mut output: Vec<Sample> = Vec::new();
        boucle.process_buffer(&input, &ops, &mut |s| output.push(s));
        assert_eq!(output, expected_output);
    }
}
