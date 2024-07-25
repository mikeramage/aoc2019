use std::collections::HashMap;

use crate::intcode;
use crate::utils;

pub fn day19() -> (usize, usize) {
    let initial_state: Vec<isize> = utils::parse_input_by_sep("input/day19.txt", ',');
    let mut program = intcode::Program::new(&initial_state);

    let tractor_beam_map: HashMap<(usize, usize), usize> = (0..100)
        .flat_map(|x| (0..100).map(move |y| (x, y)))
        .map(|(x, y)| {
            let output = calculate_beam(&initial_state, &mut program, x, y);
            ((x as usize, y as usize), output as usize)
        })
        .collect();

    let mut count: usize = 0;
    for x in 0..50 {
        for y in 0..50 {
            count += tractor_beam_map.get(&(x, y)).unwrap();
        }
    }

    for y in 0..100 {
        for x in 0..100 {
            print!("{}", tractor_beam_map.get(&(x, y)).unwrap());
        }
        println!();
    }

    //Approximate the gradient of the left edge of the tractor beam (it might be curved, but this is just to get an approximate location to search from)
    //From that printout I can see that if I start at (100, 70) and increase x I should soon hit a 1.
    let mut x = 100;
    let mut y = 70;
    let mut current_value = calculate_beam(&initial_state, &mut program, x, y);
    assert_eq!(0, current_value);

    while current_value == 0 {
        x += 1;
        current_value = calculate_beam(&initial_state, &mut program, x, y);
    }

    // println!(
    //     "Start of beam at y=70, x: {}, current_value: {}",
    //     x, current_value
    // );
    let x_min = x;
    let beam_gradient = y as f32 / x as f32;
    // println!(
    //     "Approx gradient for finding start of beam: {}",
    //     beam_gradient
    // );

    //Find the width at this y coordinate (y = 70)
    while current_value == 1 {
        x += 1;
        current_value = calculate_beam(&initial_state, &mut program, x, y);
    }

    // println!(
    //     "Width of beam at y=70, width: {}, current_value: {}",
    //     x - x_min,
    //     current_value
    // );
    let width_gradient = (x - x_min) as f32 / y as f32;
    // println!("Approx width gradient w.r.t y: {}", width_gradient);

    //Simple linear maths suggests that the beam will be wide enough around
    // y = 100 (1 + beam_gradient) / (beam_gradient * width_gradient)
    y = (100.0 * (1.0 + beam_gradient) / (beam_gradient * width_gradient)) as isize;
    x = (y as f32 / beam_gradient) as isize;
    // println!("Coord estimate: ({}, {})", x, y);

    //Algorithm. Start from estimate coordinates.
    // - If beam is 0, move right till we find the first 1.
    // - Move right until we find the x coordinate of the first 0 after the 1s - i.e. the right hand edge of the ones.
    // - Subtract 100 from x - that's the potential winning coordinates.
    // - Check value and assuming 1 (should be or something's gone very wrong with the assumption of linearity of the width)
    //    - Add 99 to y and check. If beam is 1 and beam at y+1 = 0, we've hit the target. If beam at y + 1 is 1, y is too big. Reduce initial y by 1 and repeat
    //                             If beam is 0, y is too small. Increase initial y by 1 and repeat. Assume we can keep x as it is.

    //break when we find the coordinate.
    let mut y_init = y;
    let x_init = x;
    #[allow(unused_assignments)]
    let mut x_candidate = x;
    #[allow(unused_assignments)]
    let mut y_candidate = y;
    loop {
        x = x_init;
        y = y_init;
        current_value = calculate_beam(&initial_state, &mut program, x, y);
        while current_value == 0 {
            x += 1;
            current_value = calculate_beam(&initial_state, &mut program, x, y);
        }

        while current_value == 1 {
            x += 1;
            current_value = calculate_beam(&initial_state, &mut program, x, y);
        }

        x -= 100;
        x_candidate = x;
        y_candidate = y;
        assert_eq!(1, calculate_beam(&initial_state, &mut program, x, y));

        y += 99;
        current_value = calculate_beam(&initial_state, &mut program, x, y);
        if current_value == 1 {
            if calculate_beam(&initial_state, &mut program, x, y + 1) == 0 {
                //hit target!
                break;
            } else {
                //y too big.
                y_init -= 1;
            }
        } else {
            //y too small
            y_init += 1;
        }
    }

    let day2 = x_candidate * 10_000 + y_candidate;

    (count, day2 as usize)
}

fn calculate_beam(
    initial_state: &[isize],
    program: &mut intcode::Program,
    x: isize,
    y: isize,
) -> isize {
    program.initialize(initial_state);
    program.add_input(x);
    program.add_input(y);
    program.run();
    program.remove_last_output().unwrap()
}
