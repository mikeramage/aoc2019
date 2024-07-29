use crate::intcode;
use crate::intcode::ProgramResult;
use crate::utils;
use itertools::Itertools;
use std::sync::mpsc;
use std::thread;

// Each amplifier has a copy of the program, an mpsc receiver and an mpsc sender. For example amplifier C will have receiver corresponding to B's sender and
// sender corresponding to D's receiver.
struct Amplifier {
    program: intcode::Program,
    sender: mpsc::Sender<isize>,
    receiver: mpsc::Receiver<isize>,
}

impl Amplifier {
    fn new(
        program: intcode::Program,
        sender: mpsc::Sender<isize>,
        receiver: mpsc::Receiver<isize>,
        phase: isize,
    ) -> Amplifier {
        let mut amplifier = Amplifier {
            program,
            sender,
            receiver,
        };
        amplifier.program.add_input(phase);
        amplifier
    }
}

///Day 7 solution
pub fn day7() -> (usize, usize) {
    let initial_state: Vec<isize> = utils::parse_input_by_sep("input/day7.txt", ',');
    let mut program = intcode::Program::new(&initial_state);

    let part1 = (0..5)
        .permutations(5)
        .map(|phases| {
            phases.into_iter().fold(0, |acc, phase| {
                program.initialize(&initial_state);
                program.add_input(phase);
                program.add_input(acc);
                program.run();
                *program
                    .outputs()
                    .last()
                    .expect("Expected at least one output")
            })
        })
        .max()
        .expect("Expected a maximum value from permutations");

    // Let's have some fun with Rust's message passing.

    // Initialize a copy of the program to use - we'll clone it for each amplifier.
    program.initialize(&initial_state);

    let part2 = (5..10)
        .permutations(5)
        .map(|phases| do_feedback_loop(&phases, &program))
        .max()
        .expect("Expected a maximum value from permutations");

    (part1 as usize, part2 as usize)
}

fn do_feedback_loop(phases: &[isize], program: &intcode::Program) -> isize {
    // Create 5 mpsc channels for A->B, B->C, C->D, D->E and E (and to get things going, main)->A.

    //Create the one for E-A first; this is a special case
    let (last_sender, initial_receiver) = mpsc::channel();
    let mut prev_receiver = initial_receiver;

    let mut amplifiers = Vec::new();

    for &phase in phases.iter().take(4) {
        let (sender, receiver) = mpsc::channel();
        amplifiers.push(Amplifier::new(
            program.clone(),
            sender,
            prev_receiver,
            phase,
        ));
        prev_receiver = receiver;
    }

    let sender_main = last_sender.clone();
    amplifiers.push(Amplifier::new(
        program.clone(),
        last_sender,
        prev_receiver,
        phases[4],
    ));

    let mut amplifier_thread_handles = vec![];

    for mut amplifier in amplifiers {
        amplifier_thread_handles.push(thread::spawn(move || -> isize {
            loop {
                let input = amplifier.receiver.recv().expect("Failed to get input");
                amplifier.program.add_input(input);
                let result = amplifier.program.run();
                match amplifier
                    .sender
                    .send(*amplifier.program.outputs().last().unwrap())
                {
                    Ok(_) => (),
                    Err(err) => {
                        if result == ProgramResult::Halted {
                            //assume the other end of the channel is broken because everything
                            //halted and it's a success.
                        } else {
                            //If this happened halfway through, there's a problem!
                            panic!("Error sending to next amplifier: {err}")
                        }
                    }
                }
                match result {
                    ProgramResult::AwaitingInput => continue,
                    ProgramResult::Halted => break,
                }
            }
            *amplifier.program.outputs().last().unwrap()
        }));
    }

    //Send in the first input to A.
    sender_main.send(0).expect("Sending initial input failed!");

    //When all threads are finished, get the output from the last one (E).
    amplifier_thread_handles
        .into_iter()
        .map(|x| x.join().unwrap())
        .last()
        .expect("Expected amplifier E to have an output")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_amplifiers() {
        assert_eq!(
            43210,
            get_output_for_listing_and_phases(
                &vec![3, 15, 3, 16, 1002, 16, 10, 16, 1, 16, 15, 15, 4, 15, 99, 0, 0],
                &vec![4, 3, 2, 1, 0]
            )
        );
        assert_eq!(
            54321,
            get_output_for_listing_and_phases(
                &vec![
                    3, 23, 3, 24, 1002, 24, 10, 24, 1002, 23, -1, 23, 101, 5, 23, 23, 1, 24, 23,
                    23, 4, 23, 99, 0, 0
                ],
                &vec![0, 1, 2, 3, 4]
            )
        );
        assert_eq!(
            65210,
            get_output_for_listing_and_phases(
                &vec![
                    3, 31, 3, 32, 1002, 32, 10, 32, 1001, 31, -2, 31, 1007, 31, 0, 33, 1002, 33, 7,
                    33, 1, 33, 31, 31, 1, 32, 31, 31, 4, 31, 99, 0, 0, 0
                ],
                &vec![1, 0, 4, 3, 2]
            )
        );
    }

    #[test]
    fn test_feedback_loop() {
        assert_eq!(
            139629729,
            do_feedback_loop(
                &vec![9, 8, 7, 6, 5],
                &intcode::Program::new(&vec![
                    3, 26, 1001, 26, -4, 26, 3, 27, 1002, 27, 2, 27, 1, 27, 26, 27, 4, 27, 1001,
                    28, -1, 28, 1005, 28, 6, 99, 0, 0, 5
                ])
            )
        );
        assert_eq!(
            18216,
            do_feedback_loop(
                &vec![9, 7, 8, 5, 6],
                &intcode::Program::new(&vec![
                    3, 52, 1001, 52, -5, 52, 3, 53, 1, 52, 56, 54, 1007, 54, 5, 55, 1005, 55, 26,
                    1001, 54, -5, 54, 1105, 1, 12, 1, 53, 54, 53, 1008, 54, 0, 55, 1001, 55, 1, 55,
                    2, 53, 55, 53, 4, 53, 1001, 56, -1, 56, 1005, 56, 6, 99, 0, 0, 0, 0, 10
                ])
            )
        );
    }

    fn get_output_for_listing_and_phases(listing: &[isize], phases: &[isize]) -> isize {
        let mut program = intcode::Program::new(&listing);
        let mut output = 0;
        for phase in phases {
            program.initialize(&listing);
            program.add_input(*phase);
            program.add_input(output);
            program.run();
            output = *program.outputs().last().unwrap();
        }
        output
    }
}
