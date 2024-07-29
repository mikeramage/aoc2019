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

    let mut ascii_input = "NOT A J
    NOT B T
    OR T J
    NOT C T
    OR T J
    AND D J
    WALK\n";

    add_ascii_input(&mut program, ascii_input);

    result = program.run();
    assert_eq!(result, intcode::ProgramResult::Halted);

    //Uncomment if springdroid fails
    // print_failed_attempt(&program);

    let part1 = program.remove_last_output().unwrap();

    program.initialize(&initial_state);
    let mut result = program.run();
    assert_eq!(intcode::ProgramResult::AwaitingInput, result);
    ascii_input = "NOT A J
    NOT B T
    OR T J
    NOT C T
    OR T J
    AND D J
    NOT E T
    NOT T T
    OR H T
    AND T J
    RUN\n";
    add_ascii_input(&mut program, ascii_input);

    result = program.run();
    assert_eq!(result, intcode::ProgramResult::Halted);

    //Uncomment if springdroid fails
    // print_failed_attempt(&program);

    let part2 = program.remove_last_output().unwrap();

    (part1 as usize, part2 as usize)
}

fn add_ascii_input(program: &mut intcode::Program, ascii_input: &str) {
    for c in ascii_input.bytes().map(|b| b as isize) {
        program.add_input(c);
    }
}

#[allow(dead_code)]
fn print_failed_attempt(program: &intcode::Program) {
    for c in program.outputs().iter().map(|c| *c as u8 as char) {
        print!("{}", c);
    }
}
