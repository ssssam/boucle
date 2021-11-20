#[cfg(test)]
mod event_recorder {
    use std::time::{Duration, Instant};

    use crate::BeatFraction;
    use crate::event::StateChange;
    use crate::EventRecorder;
    use crate::Operation;
    use crate::SamplePosition;

    const TEST_SAMPLE_RATE: u32 = 44100;
    const ONE_SECOND: SamplePosition = TEST_SAMPLE_RATE as SamplePosition;

    fn recorder_with_one_event(sync_point: Instant) -> EventRecorder {
        let mut recorder = EventRecorder::new(TEST_SAMPLE_RATE);
        recorder.set_event_sync_point(sync_point, 0);

        recorder.record_event(sync_point, StateChange::On, Operation::Reverse);
        recorder.record_event(sync_point + Duration::from_secs(1), StateChange::Off, Operation::Reverse);
        return recorder;
    }

    fn recorder_with_two_event_combo(sync_point: Instant) -> EventRecorder {
        let mut recorder = EventRecorder::new(TEST_SAMPLE_RATE);
        recorder.set_event_sync_point(sync_point, 0);

        let op_1 = Operation::Jump { offset: BeatFraction::from(1.0) };
        let op_2 = Operation::Jump { offset: BeatFraction::from(2.0) };
        recorder.record_event(sync_point, StateChange::On, op_1);
        recorder.record_event(sync_point + Duration::from_secs(1), StateChange::On, op_2);
        recorder.record_event(sync_point + Duration::from_secs(2), StateChange::Off, op_1);
        recorder.record_event(sync_point + Duration::from_secs(3), StateChange::Off, op_2);
        return recorder;
    }

    #[test]
    fn one_event_one_second() {
        let instant = Instant::now();
        let mut recorder = recorder_with_one_event(instant);

        let ops_one_second = recorder.ops_for_period(0, ONE_SECOND);
        assert_eq!(ops_one_second.len(), 1);
        assert_eq!(ops_one_second[0].start, 0);
        // The op didn't end in the period, so no duration here.
        assert_eq!(ops_one_second[0].duration, None);
        assert_eq!(ops_one_second[0].operation, Operation::Reverse);
    }

    #[test]
    fn one_event_two_seconds() {
        let instant = Instant::now();
        let mut recorder = recorder_with_one_event(instant);

        let ops_two_seconds = recorder.ops_for_period(0, ONE_SECOND * 2);
        assert_eq!(ops_two_seconds.len(), 1);
        assert_eq!(ops_two_seconds[0].start, 0);
        assert_eq!(ops_two_seconds[0].duration, Some(ONE_SECOND));
        assert_eq!(ops_two_seconds[0].operation, Operation::Reverse);
    }

    #[test]
    fn one_event_half_second() {
        let instant = Instant::now();
        let mut recorder = recorder_with_one_event(instant);
        // Query ops *after* the period began.
        let ops_half_second = recorder.ops_for_period(ONE_SECOND / 2, ONE_SECOND);
        assert_eq!(ops_half_second.len(), 1);
        assert_eq!(ops_half_second[0].start, 0);
        assert_eq!(ops_half_second[0].duration, Some(ONE_SECOND / 2));
        assert_eq!(ops_half_second[0].operation, Operation::Reverse);
    }

    #[test]
    fn two_event_combo() {
        env_logger::init();
        let instant = Instant::now();
        let mut recorder = recorder_with_two_event_combo(instant);

        let op_1 = Operation::Jump { offset: BeatFraction::from(1.0) };
        let op_2 = Operation::Jump { offset: BeatFraction::from(2.0) };

        let ops_first = recorder.ops_for_period(0, ONE_SECOND);
        assert_eq!(ops_first.len(), 1);
        assert_eq!(ops_first[0].start, 0);
        assert_eq!(ops_first[0].duration, None);
        assert_eq!(ops_first[0].operation, op_1);

        let ops_second = recorder.ops_for_period(ONE_SECOND, ONE_SECOND * 2);
        // Both ops are active, they are same type so we should get a single
        // op that represents the combo.
        assert_eq!(ops_second.len(), 2);
        assert_eq!(ops_second[0].start, 0);
        assert_eq!(ops_second[0].duration, Some(ONE_SECOND));
        assert_eq!(ops_second[0].operation, op_1);
        assert_eq!(ops_second[1].start, ONE_SECOND);
        assert_eq!(ops_second[1].duration, None);
        assert_eq!(ops_second[1].operation, op_2);

        let ops_third = recorder.ops_for_period(ONE_SECOND * 2, ONE_SECOND * 3);
        assert_eq!(ops_third.len(), 1);
        assert_eq!(ops_third[0].start, ONE_SECOND);
        assert_eq!(ops_third[0].duration, Some(ONE_SECOND));
        assert_eq!(ops_third[0].operation, op_2);
    }
}

#[cfg(test)]
mod operations {
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
        let input = make_buffer(&[1,2,3,4, 5,6,7,8]);
        let boucle: Boucle = Boucle::new(&TEST_CONFIG, input.len());

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
        let input = make_buffer(&[1,2,3,4, 5,6,7,8, 9,10,11,12, 13,14,15,16]);
        let boucle: Boucle = Boucle::new(&TEST_CONFIG, input.len());

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
        let input = make_buffer(&[1,2,3,4, 5,6,7,8, 9,10,11,12, 13,14,15,16, 17,18,19,20, 21,22,23,24]);
        let boucle: Boucle = Boucle::new(&TEST_CONFIG, input.len());

        let ops: OpSequence = vec!(
            op_sequence::Entry { start: 0, duration: Some(20), operation: Operation::Repeat { loop_size: BeatFraction::from(8.0) } },
        );
        let expected_output = make_buffer(&[1,2,3,4, 5,6,7,8, 1,2,3,4, 5,6,7,8, 1,2,3,4, 21,22,23,24]);

        let mut output: Vec<Sample> = Vec::new();
        boucle.process_buffer(&input, 0, input.len(), &ops, &mut |s| output.push(s));
        assert_eq!(output, expected_output);
    }
}
