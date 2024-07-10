use crate::utils;
use num::integer::lcm;
use regex::Regex;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, Hash, PartialEq, PartialOrd, Ord, Eq)]
struct Vector(isize, isize, isize);

impl Vector {
    fn add(vector1: &Vector, vector2: &Vector) -> Vector {
        Vector(
            vector1.0 + vector2.0,
            vector1.1 + vector2.1,
            vector1.2 + vector2.2,
        )
    }

    fn signum_diff(vector1: &Vector, vector2: &Vector) -> Vector {
        Vector(
            (vector1.0 - vector2.0).signum(),
            (vector1.1 - vector2.1).signum(),
            (vector1.2 - vector2.2).signum(),
        )
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Ord, PartialOrd, Clone, Copy)]
struct Moon {
    id: usize,
    position: Vector,
    velocity: Vector,
}

impl Moon {
    pub fn new(id: usize, position: Vector, velocity: Vector) -> Moon {
        Moon {
            id,
            position,
            velocity,
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Ord, PartialOrd, Clone, Copy)]
struct OneDState {
    // Tuples of a single coordinate of position and velocity of each of the 4 moons, 
    moon0: (isize, isize),
    moon1: (isize, isize),
    moon2: (isize, isize),
    moon3: (isize, isize),
}

pub fn day12() -> (usize, usize) {
    let moon_positions: Vec<String> = utils::parse_input("input/day12.txt");
    let mut moons = parse_moons(&moon_positions);
    let time_steps: usize = 1000;
    (moons, _) = simulate_motion(moons, time_steps, false);

    let part1 = calculate_energy(&moons);

    moons = parse_moons(&moon_positions);
    let (_, periods) = simulate_motion(moons, 1_000_000, true);
    let part2 = lcm(lcm(periods[0], periods[1]), periods[2]);

    (part1 as usize, part2)
}

fn calculate_energy(moons: &[Moon]) -> isize {
    moons
        .iter()
        .map(|moon| {
            (moon.position.0.abs() + moon.position.1.abs() + moon.position.2.abs())
                * (moon.velocity.0.abs() + moon.velocity.1.abs() + moon.velocity.2.abs())
        })
        .sum()
}

fn simulate_motion(mut moons: Vec<Moon>, max_time_steps: usize, find_periods: bool) -> (Vec<Moon>, Vec<usize>) {
    let mut moonsets: Vec<HashSet<OneDState>> = vec![HashSet::new(); 3];
    let mut period_count: HashMap<usize, usize> = HashMap::new();
    let mut period_figurator: Vec<HashMap<OneDState, Vec<usize>>> = vec![HashMap::new(); 3];
    let mut counter = 0;

    'outer:
    while counter < max_time_steps {

        if find_periods {
            for i in 0..3 {
                //x, y and z coordinates are independent, so figure out if there's a regular period when 
                //the x coordinates are the same for positions and velocities across the 4 moons.
                let one_d_state: OneDState = match i {
                    //Grrr - can't dynamically index into tuple!
                    0 => OneDState{moon0: (moons[0].position.0, moons[0].velocity.0),
                        moon1: (moons[1].position.0, moons[1].velocity.0),
                        moon2: (moons[2].position.0, moons[2].velocity.0),
                        moon3: (moons[3].position.0, moons[3].velocity.0)},
                    1 => OneDState{moon0: (moons[0].position.1, moons[0].velocity.1),
                        moon1: (moons[1].position.1, moons[1].velocity.1),
                        moon2: (moons[2].position.1, moons[2].velocity.1),
                        moon3: (moons[3].position.1, moons[3].velocity.1)},
                    2 => OneDState{moon0: (moons[0].position.2, moons[0].velocity.2),
                        moon1: (moons[1].position.2, moons[1].velocity.2),
                        moon2: (moons[2].position.2, moons[2].velocity.2),
                        moon3: (moons[3].position.2, moons[3].velocity.2)},
                    other => panic!("Should only be iterating up to 2. Got {other}")
                };
                 
                if moonsets[i].contains(&one_d_state) {
                    //Seen this configuration before for the i coordinates of the moons
                    
                    period_figurator[i].entry(one_d_state).and_modify(|x| x.push(counter)).or_insert_with(|| vec![counter]);
                    if period_figurator[i].get(&one_d_state).unwrap().len() == 3 {
                        let diff = period_figurator[i].get(&one_d_state).unwrap()[2] - period_figurator[i].get(&one_d_state).unwrap()[1];
                        let diff2 = period_figurator[i].get(&one_d_state).unwrap()[1] - period_figurator[i].get(&one_d_state).unwrap()[0];
                        if diff == diff2 {
                            //Cycle found
                            period_count.insert(i, diff);
                        }
                        else {
                            panic!("Cycle not found :-(   first: {:?} second: {:?}. Moon: {:#?}\nConfigurator: {:#?}", diff, diff2, moons[i], period_figurator[i].get(&one_d_state).unwrap());
                        }

                        if period_count.len() == 3 {
                            break 'outer;
                        }
                    }
                } else {
                    moonsets[i].insert(one_d_state);
                }
            }
        }

        //Get velocity deltas due to gravity
        let mut deltas: Vec<Vector> = vec![];
        for moon in &moons {
            let mut delta = Vector(0, 0, 0);
            for other in &moons {
                if other.id != moon.id {
                    //Compare positions.
                    delta = Vector::add(
                        &delta,
                        &Vector::signum_diff(&other.position, &moon.position),
                    );
                }
            }
            deltas.push(delta);
        }

        //Apply the deltas to each moon's velocity and calculate the position
        for (moon, delta) in moons.iter_mut().zip(deltas) {
            moon.velocity = Vector::add(&moon.velocity, &delta);
            moon.position = Vector::add(&moon.position, &moon.velocity);
        }

        counter += 1;
    }

    (moons, period_count.values().copied().collect::<Vec<usize>>())
}

fn parse_moons(moon_positions: &[String]) -> Vec<Moon> {
    let moon_regex = Regex::new(r"<x=(?<x>.*), y=(?<y>.*), z=(?<z>.*)>").expect("Bad moon regex!");
    let mut moons: Vec<Moon> = vec![];
    for (id, line) in moon_positions.iter().enumerate() {
        let captures = moon_regex.captures(line).expect("Failed to match regex!");
        let x: isize = captures["x"].parse::<isize>().expect("Failed to parse x");
        let y: isize = captures["y"].parse::<isize>().expect("Failed to parse y");
        let z: isize = captures["z"].parse::<isize>().expect("Failed to parse z");
        moons.push(Moon::new(id, Vector(x, y, z), Vector(0, 0, 0)));
    }

    moons
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulate_moon_motion() {
        let moon_positions: Vec<String> = vec![
            "<x=-1, y=0, z=2>".to_string(),
            "<x=2, y=-10, z=-7>".to_string(),
            "<x=4, y=-8, z=8>".to_string(),
            "<x=3, y=5, z=-1>".to_string(),
        ];
        let mut moons = parse_moons(&moon_positions);
        assert_eq!(Vector(-1, 0, 2), moons[0].position);
        assert_eq!(Vector(0, 0, 0), moons[0].velocity);
        assert_eq!(Vector(2, -10, -7), moons[1].position);
        assert_eq!(Vector(0, 0, 0), moons[1].velocity);
        assert_eq!(Vector(4, -8, 8), moons[2].position);
        assert_eq!(Vector(0, 0, 0), moons[2].velocity);
        assert_eq!(Vector(3, 5, -1), moons[3].position);
        assert_eq!(Vector(0, 0, 0), moons[3].velocity);

        (moons, _) = simulate_motion(moons, 1, false);
        assert_eq!(Vector(2, -1, 1), moons[0].position);
        assert_eq!(Vector(3, -1, -1), moons[0].velocity);
        assert_eq!(Vector(3, -7, -4), moons[1].position);
        assert_eq!(Vector(1, 3, 3), moons[1].velocity);
        assert_eq!(Vector(1, -7, 5), moons[2].position);
        assert_eq!(Vector(-3, 1, -3), moons[2].velocity);
        assert_eq!(Vector(2, 2, 0), moons[3].position);
        assert_eq!(Vector(-1, -3, 1), moons[3].velocity);

        (moons, _) = simulate_motion(moons, 2, false);
        assert_eq!(Vector(5, -6, -1), moons[0].position);
        assert_eq!(Vector(0, -3, 0), moons[0].velocity);
        assert_eq!(Vector(0, 0, 6), moons[1].position);
        assert_eq!(Vector(-1, 2, 4), moons[1].velocity);
        assert_eq!(Vector(2, 1, -5), moons[2].position);
        assert_eq!(Vector(1, 5, -4), moons[2].velocity);
        assert_eq!(Vector(1, -8, 2), moons[3].position);
        assert_eq!(Vector(0, -4, 0), moons[3].velocity);

        (moons, _) = simulate_motion(moons, 7, false);
        assert_eq!(Vector(2, 1, -3), moons[0].position);
        assert_eq!(Vector(-3, -2, 1), moons[0].velocity);
        assert_eq!(Vector(1, -8, 0), moons[1].position);
        assert_eq!(Vector(-1, 1, 3), moons[1].velocity);
        assert_eq!(Vector(3, -6, 1), moons[2].position);
        assert_eq!(Vector(3, 2, -3), moons[2].velocity);
        assert_eq!(Vector(2, 0, 4), moons[3].position);
        assert_eq!(Vector(1, -1, -1), moons[3].velocity);

        assert_eq!(179, calculate_energy(&moons));
    }

    #[test]
    fn test_simulate_moon_motion_2() {
        let moon_positions: Vec<String> = vec![
            "<x=-8, y=-10, z=0>".to_string(),
            "<x=5, y=5, z=10>".to_string(),
            "<x=2, y=-7, z=3>".to_string(),
            "<x=9, y=-8, z=-3>".to_string(),
        ];

        let mut moons = parse_moons(&moon_positions);
        (moons, _) = simulate_motion(moons, 100, false);

        assert_eq!(Vector(8, -12, -9), moons[0].position);
        assert_eq!(Vector(-7, 3, 0), moons[0].velocity);
        assert_eq!(Vector(13, 16, -3), moons[1].position);
        assert_eq!(Vector(3, -11, -5), moons[1].velocity);
        assert_eq!(Vector(-29, -11, -1), moons[2].position);
        assert_eq!(Vector(-3, 7, 4), moons[2].velocity);
        assert_eq!(Vector(16, -13, 23), moons[3].position);
        assert_eq!(Vector(7, 1, 1), moons[3].velocity);
        assert_eq!(1940, calculate_energy(&moons));
    }

    #[test]
    fn test_steps_till_repeat() {
        let moon_positions: Vec<String> = vec![
            "<x=-1, y=0, z=2>".to_string(),
            "<x=2, y=-10, z=-7>".to_string(),
            "<x=4, y=-8, z=8>".to_string(),
            "<x=3, y=5, z=-1>".to_string(),
        ];
        let moons = parse_moons(&moon_positions);
        let (_, periods) = simulate_motion(moons, 3000, true);
        assert_eq!(2772, lcm(lcm(periods[0], periods[1]), periods[2]));
    }
}
