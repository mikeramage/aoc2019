use crate::intcode;
use crate::utils;

///Day 5 solution
pub fn day5() -> (usize, usize) {
    let initial_state: Vec<isize> = utils::parse_input_by_sep("input/day5.txt", ',');
    let mut program = intcode::Program::new(&initial_state);
    program.add_input(1);
    program.run();
    let part1 = *program.outputs().last().unwrap();
    program.initialize(&initial_state);
    program.add_input(5);
    program.run();
    let part2 = *program.outputs().last().unwrap();

    (part1 as usize, part2 as usize)
}
