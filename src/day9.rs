use crate::intcode;
use crate::utils;

///Day 9 solution
pub fn day9() -> (usize, usize) {
    let initial_state: Vec<isize> = utils::parse_input_by_sep("input/day9.txt", ',');
    let mut program = intcode::Program::new(&initial_state);

    (0, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_input_by_sep() {
        assert_eq!(
            vec!(2, 3, 4, 5, 6),
            utils::parse_input_by_sep("input/day2_test1.txt", ',')
        );
        assert_eq!(
            vec!("bob", "charlie", "daniela", "edward", "fiona", "gary", "helen", "ian", "jane"),
            utils::parse_input_by_sep::<String>("input/day2_test2.txt", '-')
        );
    }
}
