use crate::utils;
use itertools::Itertools;
use num::integer;
use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
};

#[derive(PartialEq, Hash, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
struct Asteroid {
    x: i32,
    y: i32,
}

impl Asteroid {
    fn new(x: i32, y: i32) -> Asteroid {
        Asteroid { x, y }
    }
}

#[derive(PartialEq, Hash, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
struct Direction {
    x: i32,
    y: i32,
}

impl Direction {
    //Reduces the inputs to scale by greatest common denominator
    pub fn new(x: i32, y: i32) -> Direction {
        if x == 0 && y == 0 {
            panic!("Undefined direction!");
        }

        let (dir_x, dir_y) = match (x, y) {
            (0, y) => (0, y.signum()),
            (x, 0) => (x.signum(), 0),
            _ => {
                let gcd = integer::gcd(x, y);
                (x / gcd, y / gcd)
            }
        };

        Direction { x: dir_x, y: dir_y }
    }
}

///Day 10 solution
pub fn day10() -> (usize, usize) {
    let input = utils::parse_input::<String>("input/day10.txt");
    let asteroids = parse_asteroid_map(&input);

    //Part 1 (also sets up part 2)
    //For asteroid A in the set, for each other asteroid B, define the Direction from A to B and put B in a Vec in the HashMap keyed by the Direction.
    //Return the max number of hashmap keys over all asteroids A and get a ref to A. Don't worry about the ordering of the Vec for now
    let direction_map = get_direction_map(&asteroids);

    let part1 = direction_map
        .values()
        .map(|asteroid_map| asteroid_map.iter().count())
        .max()
        .unwrap();
    let station_location: Asteroid = direction_map
        .iter()
        .filter(|(_, asteroid_map)| asteroid_map.len() == part1)
        .map(|x| *x.0)
        .last()
        .unwrap();

    let vaporized_asteroids = get_vaporized_asteroids(&direction_map, &station_location);
    let part2 = (vaporized_asteroids[199].x * 100 + vaporized_asteroids[199].y) as usize;

    (part1, part2)
}

fn get_vaporized_asteroids(
    direction_map: &HashMap<Asteroid, HashMap<Direction, Vec<Asteroid>>>,
    station_location: &Asteroid,
) -> Vec<Asteroid> {
    let station_map = direction_map
        .get(station_location)
        .expect("Station location not found in direction map");

    let mut station_map: HashMap<_, _> = station_map
        .iter()
        .map(|(&direction, asteroids)| {
            let mut sorted_asteroids = asteroids.clone();
            sorted_asteroids.sort_by(|a, b| {
                squared_distance(station_location, b)
                    .partial_cmp(&squared_distance(station_location, a))
                    .unwrap()
            });
            (direction, sorted_asteroids)
        })
        .collect();

    let sorted_keys: Vec<_> = station_map
        .keys()
        .sorted_by(|a, b| clockwise_compare(a, b))
        .cloned()
        .collect();

    let mut vaporized_asteroids = Vec::new();
    while !station_map.values().all(Vec::is_empty) {
        for k in &sorted_keys {
            if let Some(v) = station_map.get_mut(k) {
                if let Some(asteroid) = v.pop() {
                    vaporized_asteroids.push(asteroid);
                }
            }
        }
    }

    vaporized_asteroids
}

fn squared_distance(a: &Asteroid, b: &Asteroid) -> i32 {
    (a.x - b.x).pow(2) + (a.y - b.y).pow(2)
}

fn parse_asteroid_map(input: &[String]) -> HashSet<Asteroid> {
    let mut asteroids: HashSet<Asteroid> = HashSet::new();
    for (y, row) in input.iter().enumerate() {
        for (x, location) in row.chars().enumerate() {
            if location == '#' {
                assert!(asteroids.insert(Asteroid::new(x as i32, y as i32)));
            } else {
                assert_eq!('.', location);
            }
        }
    }
    asteroids
}

fn get_direction_map(
    asteroids: &HashSet<Asteroid>,
) -> HashMap<Asteroid, HashMap<Direction, Vec<Asteroid>>> {
    let mut direction_map: HashMap<Asteroid, HashMap<Direction, Vec<Asteroid>>> = HashMap::new();

    for asteroid in asteroids {
        let mut asteroid_map: HashMap<Direction, Vec<Asteroid>> = HashMap::new();
        for other in asteroids {
            if other == asteroid {
                continue;
            }
            let direction = Direction::new(other.x - asteroid.x, other.y - asteroid.y);
            asteroid_map
                .entry(direction)
                .and_modify(|v| v.push(*other))
                .or_insert_with(|| vec![*other]);
        }

        direction_map.insert(*asteroid, asteroid_map);
    }

    direction_map
}

// Quadrant of direction. +ve y is downwards. Top right is 1, bottom right 2, bottom left 3, top left 4
fn quadrant_order(d: &Direction) -> i32 {
    // (x >= 0 && y < 0) > (x > 0 && y >=0) > (x <= 0 && y > 0) > (x < 0 && y <= 0)
    if d.x >= 0 && d.y < 0 {
        //First quadrant
        1
    } else if d.x > 0 && d.y >= 0 {
        //Second quadrant
        2
    } else if d.x <= 0 && d.y > 0 {
        //Third quadrant
        3
    } else {
        assert!(d.x < 0 && d.y <= 0);
        4
    }
}

fn clockwise_compare(a: &Direction, b: &Direction) -> Ordering {
    //Direction is defined by 2 numbers, x and y, scaled by dividing by the greatest common denominator (num::rational::Ratio is no good because it doesn't distinguish between
    //x/-y and -x/y or x/y and -x/-y and that's an essential distinction for ordering). Clockwise ordering is defined as follows:
    // - First order by quadrant: (x>=0 and y>0) > (x>0 and y<=0) > (x<=0 and y<0) > (x<0 and y>=0)
    // - If the two directions are in the same quadrant, then the one with the greater (y as f32)/(x as f32) is bigger (this works for all 4 quadrants). For cases x = 0, set
    // x = some small epsilon such that y/epsilon >> max(y).

    //compare quadrants

    match quadrant_order(a).cmp(&quadrant_order(b)) {
        Ordering::Equal => {
            //Same quadrant.
            let epsilon: f32 = 0.00001;
            let ax = if a.x == 0 {
                match quadrant_order(a) {
                    1 => epsilon,
                    3 => -epsilon,
                    _ => panic!("x should only be 0 in quadrants 1 and 3"),
                }
            } else {
                a.x as f32
            };
            let bx = if b.x == 0 {
                match quadrant_order(b) {
                    1 => epsilon,
                    3 => -epsilon,
                    _ => panic!("x should only be 0 in quadrants 1 and 3"),
                }
            } else {
                b.x as f32
            };

            let a_gradient: f32 = (a.y as f32) / ax;
            let b_gradient: f32 = (b.y as f32) / bx;

            // Important - we want directions with the highest gradient to be
            // considered smallest in clockwork ordering. The vector pointing closest
            // to due north is the smallest, i.e. the one we start with. But that's
            // the one with the largest gradient, so swap a and b here.
            a_gradient.partial_cmp(&b_gradient).unwrap()
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_asteroid_map_1() {
        let input: Vec<String> = utils::parse_input::<String>("input/day10_test1.txt");
        let asteroids = parse_asteroid_map(&input);
        let direction_map = get_direction_map(&asteroids);
        assert_eq!(
            8,
            direction_map
                .iter()
                .map(|(_, asteroid_map)| asteroid_map.iter().count())
                .max()
                .unwrap()
        );
    }

    #[test]
    fn test_basic_asteroid_map_2() {
        let input: Vec<String> = utils::parse_input::<String>("input/day10_test2.txt");
        let asteroids = parse_asteroid_map(&input);
        let direction_map = get_direction_map(&asteroids);
        assert_eq!(
            33,
            direction_map
                .iter()
                .map(|(_, asteroid_map)| asteroid_map.iter().count())
                .max()
                .unwrap()
        );
    }

    #[test]
    fn test_basic_asteroid_map_3() {
        let input: Vec<String> = utils::parse_input::<String>("input/day10_test3.txt");
        let asteroids = parse_asteroid_map(&input);
        let direction_map = get_direction_map(&asteroids);
        assert_eq!(
            35,
            direction_map
                .iter()
                .map(|(_, asteroid_map)| asteroid_map.iter().count())
                .max()
                .unwrap()
        );
    }

    #[test]
    fn test_basic_asteroid_map_4() {
        let input: Vec<String> = utils::parse_input::<String>("input/day10_test4.txt");
        let asteroids = parse_asteroid_map(&input);
        let direction_map = get_direction_map(&asteroids);
        assert_eq!(
            41,
            direction_map
                .iter()
                .map(|(_, asteroid_map)| asteroid_map.iter().count())
                .max()
                .unwrap()
        );
    }

    #[test]
    fn test_basic_asteroid_map_5() {
        let input: Vec<String> = utils::parse_input::<String>("input/day10_test5.txt");
        let asteroids = parse_asteroid_map(&input);
        let direction_map = get_direction_map(&asteroids);
        assert_eq!(
            210,
            direction_map
                .iter()
                .map(|(_, asteroid_map)| asteroid_map.iter().count())
                .max()
                .unwrap()
        );
    }

    #[test]
    fn test_vaporization() {
        let input: Vec<String> = utils::parse_input::<String>("input/day10_test5.txt");
        let asteroids = parse_asteroid_map(&input);
        let direction_map = get_direction_map(&asteroids);
        let part1 = direction_map
            .iter()
            .map(|(_, asteroid_map)| asteroid_map.iter().count())
            .max()
            .unwrap();
        let station_location: Asteroid = direction_map
            .iter()
            .filter(|(_, asteroid_map)| asteroid_map.len() == part1)
            .map(|x| x.0.clone())
            .last()
            .unwrap();
        assert_eq!(Asteroid::new(11, 13), station_location);

        let vaporized_asteroids = get_vaporized_asteroids(&direction_map, &station_location);
        assert_eq!(299, vaporized_asteroids.len());
        assert_eq!(Asteroid::new(11, 12), vaporized_asteroids[0]);
        assert_eq!(Asteroid::new(12, 1), vaporized_asteroids[1]);
        assert_eq!(Asteroid::new(12, 2), vaporized_asteroids[2]);
        assert_eq!(Asteroid::new(12, 8), vaporized_asteroids[9]);
        assert_eq!(Asteroid::new(16, 0), vaporized_asteroids[19]);
        assert_eq!(Asteroid::new(16, 9), vaporized_asteroids[49]);
        assert_eq!(Asteroid::new(10, 16), vaporized_asteroids[99]);
        assert_eq!(Asteroid::new(9, 6), vaporized_asteroids[198]);
        assert_eq!(Asteroid::new(8, 2), vaporized_asteroids[199]);
        assert_eq!(Asteroid::new(10, 9), vaporized_asteroids[200]);
        assert_eq!(Asteroid::new(11, 1), vaporized_asteroids[298]);
    }
}
