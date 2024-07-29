use crate::intcode;
use crate::utils;
#[allow(unused_imports)]
use std::io::stdin;

pub fn day21() -> (usize, usize) {
    let initial_state: Vec<isize> = utils::parse_input_by_sep("input/day21.txt", ',');
    let mut program = intcode::Program::new(&initial_state);

    let mut result = program.run();
    assert_eq!(intcode::ProgramResult::AwaitingInput, result);

    // To have a manual play ...
    // let stdin = stdin();
    // let mut walk = false;
    // while !walk {
    //     let mut user_input: String = String::new();
    //     stdin
    //         .read_line(&mut user_input)
    //         .unwrap_or_else(|err| panic!("Failed to get user input: {err}"));

    //     if user_input == "WALK\n" {
    //         walk = true;
    //     }
    //     for c in user_input.bytes().map(|b| b as isize) {
    //         program.add_input(c);
    //     }
    // }

    let program_input = "NOT A J
    NOT B T
    OR T J
    NOT C T
    OR T J
    AND D J
    WALK\n";
    for c in program_input.bytes().map(|b| b as isize) {
        program.add_input(c);
    }

    result = program.run();
    assert_eq!(result, intcode::ProgramResult::Halted);
    let part1 = program.remove_last_output().unwrap();

    (part1 as usize, 0)
}
