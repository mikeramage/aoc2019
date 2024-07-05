use crate::intcode;
use crate::utils;

///Day 9 solution
pub fn day9() -> (usize, usize) {
    let initial_state: Vec<isize> = utils::parse_input_by_sep("input/day9.txt", ',');
    let mut program = intcode::Program::new(&initial_state);
    program.add_input(1);
    program.run();
    let part1 = *program.outputs().last().unwrap();
    program.initialize(&initial_state);
    program.add_input(2);
    program.run();
    let part2 = *program.outputs().last().unwrap();


    (part1 as usize, part2 as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_day9() {
        let mut program = intcode::Program::new(&vec![
            109, 1, 204, -1, 1001, 100, 1, 100, 1008, 100, 16, 101, 1006, 101, 0, 99,
        ]);
        program.run();
        assert_eq!(
            vec![109, 1, 204, -1, 1001, 100, 1, 100, 1008, 100, 16, 101, 1006, 101, 0, 99],
            *program.outputs()
        );

        program = intcode::Program::new(&vec![1102, 34915192, 34915192, 7, 4, 7, 99, 0]);
        program.run();
        assert_eq!(16, program.outputs().last().unwrap().to_string().len());

        program = intcode::Program::new(&vec![104, 1125899906842624, 99]);
        program.run();
        assert_eq!(1125899906842624, *program.outputs().last().unwrap());
    }
}
