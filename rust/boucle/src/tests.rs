use crate::boucle::*;
use crate::ops::*;

#[cfg(test)]

const TEST_CONFIG: Config = Config { frames_per_block: 4 };

#[test]
fn basic_reverse() {
    let mut boucle: Boucle = Boucle::new(TEST_CONFIG);
    let input = vec!(1,2,3,4, 5,6,7,8, 9,10,11,12, 13,14,15,16);
    let expected_output = vec!(1,2,3,4, 12,11,10,9, 8,7,6,5, 13,14,15,16);
    let ops: Vec<Box<dyn Op>> = vec!(
        Box::new(ReverseOp { span: OpSpan { start: 1, duration: 2 } }),
    );
    let mut output: Vec<Sample> = Vec::new();
    boucle.process_buffer(&input, &ops, &mut |s| output.push(s));
    assert_eq!(output, expected_output);
}
