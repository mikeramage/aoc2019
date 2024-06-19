use crate::intcode;
use crate::utils;

///Day 2 solution
pub fn day5() -> (usize, usize) {
    let initial_state: Vec<i32> = utils::parse_input_by_sep("input/day5.txt", ',');
    let mut program = intcode::Program::new(&initial_state);
    program.set_input(1);
    program.run();
    let part1 = *program.outputs().last().unwrap();
    program.initialize(&initial_state);
    program.set_input(5);
    program.run();
    let part2 = *program.outputs().last().unwrap();

    (part1 as usize, part2 as usize)
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
