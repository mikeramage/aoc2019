use crate::intcode;
use crate::utils;
use num::Rational32;
use std::collections::HashMap;

#[derive(PartialEq, Hash, Eq, PartialOrd, Ord, Debug)]
struct Point(i32, i32);

impl Point {
    fn new(x: i32, y: i32) -> Point {
        Point(x, y)
    }
}

#[derive(Debug, PartialEq)]
struct Asteroid;

impl Asteroid {
    fn new() -> Asteroid {
        Asteroid {}
    }
}

///Day 10 solution
pub fn day10() -> (usize, usize) {
    let input = utils::parse_input::<String>("input/day10.txt");
    let mut asteroids: HashMap<Point, Asteroid> = HashMap::new();
    for (y, row) in input.iter().enumerate() {
        for (x, location) in row.chars().enumerate() {
            if location == '#' {
                asteroids.insert(Point::new(x as i32, y as i32), Asteroid::new());
            } else {
                assert_eq!('.', location);
            }
        }
    }

    (0, 0)
}
