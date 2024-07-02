use crate::intcode;
use crate::utils;
use itertools;
use itertools::Itertools;

///Day 7 solution
pub fn day7() -> (usize, usize) {
    let initial_state: Vec<i32> = utils::parse_input_by_sep("input/day7.txt", ',');
    let mut program = intcode::Program::new(&initial_state);
    let mut current_input = 0;

    let part1 = (0..5).permutations(5).map(|x| {
        current_input = 0;
        for phase in x {
            program.initialize(&initial_state);
            program.set_inputs(vec!(current_input, phase));
            program.run();
            current_input = *program.outputs().last().unwrap();
        }
        current_input
    }).max().unwrap();

    let part2 = 0;

    (part1 as usize, part2 as usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_amplifiers() {
        assert_eq!(43210, get_output_for_listing_and_phases(&vec![3,15,3,16,1002,16,10,16,1,16,15,15,4,15,99,0,0], &vec![4, 3, 2, 1, 0]));
        assert_eq!(54321, get_output_for_listing_and_phases(&vec![3,23,3,24,1002,24,10,24,1002,23,-1,23,101,5,23,23,1,24,23,23,4,23,99,0,0], &vec![0,1,2,3,4]));
        assert_eq!(65210, get_output_for_listing_and_phases(&vec![3,31,3,32,1002,32,10,32,1001,31,-2,31,1007,31,0,33,1002,33,7,33,1,33,31,31,1,32,31,31,4,31,99,0,0,0], &vec![1,0,4,3,2]));
    }

    fn get_output_for_listing_and_phases(listing: &[i32], phases: &[i32]) -> i32
    {
        let mut program = intcode::Program::new(&listing);
        let mut output = 0;
        for phase in phases {
            program.initialize(&listing);
            program.set_inputs(vec!(output, *phase));
            program.run();
            output = *program.outputs().last().unwrap();
        }
        output
    }

}

