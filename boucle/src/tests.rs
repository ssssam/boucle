#[cfg(test)]
mod tests {
    use crate::BeatFraction;
    use crate::Boucle;
    use crate::Config;
    use crate::Operation;
    use crate::Sample;
    use crate::op_sequence;
    use crate::OpSequence;

    const TEST_CONFIG: Config = Config {
        sample_rate: 44100,
        // Map 1:1 beats to samples.
        beat_fraction_to_samples: 1.0 / 16.0,
    };

    fn make_buffer(data: &[i16]) -> Vec<Sample> {
        data.iter().map(|s| Sample::from(*s)).collect()
    }

    #[test]
    fn basic_reverse() {
        let boucle: Boucle = Boucle::new(&TEST_CONFIG);
        let input = make_buffer(&[1,2,3,4, 5,6,7,8]);

        let ops: OpSequence = vec!(
            op_sequence::Entry { start: 3, duration: Some(10), operation: Operation::Reverse },
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
            op_sequence::Entry { start: 4, duration: Some(4), operation: Operation::Jump { offset: BeatFraction::from(-4.0) } },
            op_sequence::Entry { start: 12, duration: Some(4), operation: Operation::Jump { offset: BeatFraction::from(8.0) } },
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
            op_sequence::Entry { start: 0, duration: Some(20), operation: Operation::Repeat { loop_size: BeatFraction::from(8.0) } },
        );
        let expected_output = make_buffer(&[1,2,3,4, 5,6,7,8, 1,2,3,4, 5,6,7,8, 1,2,3,4, 21,22,23,24]);

        let mut output: Vec<Sample> = Vec::new();
        boucle.process_buffer(&input, 0, input.len(), &ops, &mut |s| output.push(s));
        assert_eq!(output, expected_output);
    }
}
