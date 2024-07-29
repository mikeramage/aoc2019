use crate::intcode;
use crate::utils;
use std::io::stdin;

pub fn day21() -> (usize, usize) {
    let initial_state: Vec<isize> = utils::parse_input_by_sep("input/day21.txt", ',');
    let mut program = intcode::Program::new(&initial_state);
    let stdin = stdin();
    let mut walk = false;

    let mut result = program.run();
    assert_eq!(intcode::ProgramResult::AwaitingInput, result);
    let mut program_input = vec![];

    while !walk {
        let mut user_input: String = String::new();
        stdin
            .read_line(&mut user_input)
            .unwrap_or_else(|err| panic!("Failed to get user input: {err}"));

        if user_input == "WALK\n" {
            walk = true;
        }
        let mut program_line = prepare_ascii_input(&user_input);
        program_input.append(&mut program_line);
    }

    println!("Program input: {:?}", program_input);
    program.set_inputs(program_input);
    result = program.run();
    assert_eq!(result, intcode::ProgramResult::Halted);
    for c in program.outputs() {
        print!("{}", *c as u8 as char);
    }

    (0, 0)
}

fn prepare_ascii_input(input: &str) -> Vec<isize> {
    input.bytes().map(|b| b as isize).rev().collect()
}
