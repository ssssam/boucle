use crate::boucle::*;
use crate::ops::*;
use crate::op_sequence::*;

#[cfg(test)]

const TEST_CONFIG: Config = Config { frames_per_block: 4 };

#[test]
fn basic_reverse() {
    let mut boucle: Boucle = Boucle::new(TEST_CONFIG);
    let input = vec!(1,2,3,4, 5,6,7,8, 9,10,11,12, 13,14,15,16);
    let expected_output = vec!(1,2,3,4, 4,3,2,1, 8,7,6,5, 13,14,15,16);
    let ops: OpSequenceVec = vec!(
        Entry { start: 1, duration: 2, op: Box::new(ReverseOp {}) },
    );
    let mut output: Vec<Sample> = Vec::new();
    boucle.process_buffer(&input, &ops, &mut |s| output.push(s));
    assert_eq!(output, expected_output);
}
