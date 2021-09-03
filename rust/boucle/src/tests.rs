use crate::boucle::*;

#[cfg(test)]
#[test]
fn basic() {
    let mut boucle: Boucle = Boucle::new(Config::default());
    let input = vec!(1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16);
    let expected_output = vec!(16,15,14,13,12,11,10,9,8,7,6,5,4,3,2,1);
    let result = boucle.process_block(&input, 0);
    assert_eq!(result, expected_output);
}
