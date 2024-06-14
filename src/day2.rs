use crate::intcode;
use crate::utils;

///Day 2 solution
pub fn day2() -> (usize, usize) {
    let initial_state: Vec<usize> = utils::parse_input_by_sep("input/day2.txt", ',');
    let mut program = intcode::Program::new(&initial_state);
    program.set_inputs(12, 2);
    program.run();
    let part1 = program.output();
    let mut noun = 0;
    let mut verb = 0;

    // assume noun and verb are pretty small
    'outer: for ii in 0..100 {
        for jj in 0..100 {
            program.initialize(&initial_state);
            program.set_inputs(ii, jj);
            program.run();
            let output = program.output();
            if output == 19690720 {
                noun = ii;
                verb = jj;
                break 'outer;
            }
        }
    }

    (part1, 100 * noun + verb)
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
