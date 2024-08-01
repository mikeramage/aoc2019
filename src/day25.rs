use itertools::Itertools;

use crate::intcode;
use crate::utils;
// use std::io::stdin;

pub fn day25() -> (usize, usize) {
    let initial_state: Vec<isize> = utils::parse_input_by_sep("input/day25.txt", ',');
    let mut program = intcode::Program::new(&initial_state);

    //Uncomment this and remove the automated logic to play!
    //Uncomment all the prints to see the output
    // let stdin = stdin();
    // let mut user_input: String = String::new();
    let program_result = program.run();
    assert_eq!(intcode::ProgramResult::AwaitingInput, program_result);
    print_outputs(&mut program);
    let input = "east
take whirled peas
east
north
take prime number
south
east
east
east
take dark matter
west
west
west
west
north
take coin
west
south
take antenna
north
north
west
take astrolabe
east
south
east
south
west
north
take fixed point
north
take weather machine
east
";
    for line in input.lines() {
        add_ascii_input(&mut program, format!("{}\n", line).as_str());
        program.run();
        // print_outputs(&mut program);
    }

    //Try all combinations of 8 items
    let items = [
        "dark matter",
        "coin",
        "whirled peas",
        "fixed point",
        "astrolabe",
        "prime number",
        "antenna",
        "weather machine",
    ];
    for item in items {
        add_ascii_input(&mut program, format!("drop {}\n", item).as_str());
        program.run();
        // print_outputs(&mut program);
    }

    for k in 1..=items.len() {
        for combo in items.iter().combinations(k) {
            for item in &combo {
                add_ascii_input(&mut program, format!("take {}\n", **item).as_str());
                program.run();
                // print_outputs(&mut program);
            }

            add_ascii_input(&mut program, "south\n");
            program.run();
            // print_outputs(&mut program);

            for item in &combo {
                add_ascii_input(&mut program, format!("drop {}\n", **item).as_str());
                program.run();
                // print_outputs(&mut program);
            }
        }
    }

    (0, 0)
}

fn add_ascii_input(program: &mut intcode::Program, ascii_input: &str) {
    for c in ascii_input.bytes().map(|b| b as isize) {
        program.add_input(c);
    }
}

fn print_outputs(program: &mut intcode::Program) {
    for i in program.outputs() {
        print!("{}", char::from(*i as u8));
    }
    program.clear_outputs();
}
