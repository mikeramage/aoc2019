use itertools::Itertools;

use crate::utils;
use std::mem;
use std::{collections::HashMap, panic::Location};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Hash)]
enum LocationContent {
    Empty,
    Wall,
    Entrance,
    Key { name: char },
    Door { name: char, locked: bool }, //char is the name of the door, bool is whether it's locked: true if so, false if not.
}

impl TryFrom<char> for LocationContent {
    type Error = String;
    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '.' => Ok(LocationContent::Empty),
            '#' => Ok(LocationContent::Wall),
            '@' => Ok(LocationContent::Entrance),
            other => {
                if other.is_ascii_lowercase() {
                    Ok(LocationContent::Key { name: other })
                } else if other.is_ascii_uppercase() {
                    Ok(LocationContent::Door {
                        name: other,
                        locked: true,
                    })
                } else {
                    Err(format!("Character '{}' does not map to LocationContent", c))
                }
            }
        }
    }
}

impl From<LocationContent> for char {
    fn from(lc: LocationContent) -> char {
        match lc {
            LocationContent::Empty => '.',
            LocationContent::Wall => '#',
            LocationContent::Entrance => '@',
            LocationContent::Key { name } => name,
            LocationContent::Door { name, locked: _ } => name,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash)]
struct Position {
    x: isize,
    y: isize,
}

impl Position {
    fn new(x: isize, y: isize) -> Position {
        Position { x, y }
    }
}

pub fn day18() -> (usize, usize) {
    let input: Vec<String> = utils::parse_input("input/day18.txt");
    //Let's have an initial look - parse the input into a map of Position->LocationContent
    //and some other maps of specific LocationContent (keys and doors) for the interesting stuff
    let mut map: HashMap<Position, LocationContent> = HashMap::new();
    let mut keys: HashMap<LocationContent, Position> = HashMap::new();
    let mut doors: HashMap<LocationContent, Position> = HashMap::new();
    let mut entrance_position = Position::new(0, 0);
    let mut max_x = 0;
    let mut max_y = 0;
    input.iter().enumerate().for_each(|(x, line)| {
        line.chars().enumerate().for_each(|(y, c)| {
            let lc = LocationContent::try_from(c).expect("Bad character found while parsing input");
            let pos = Position::new(x as isize, y as isize);
            if x > max_x {
                max_x = x;
            }
            if y > max_y {
                max_y = y;
            }
            map.insert(pos, lc);
            match lc {
                LocationContent::Key { name: _ } => {
                    keys.insert(lc, pos);
                }
                LocationContent::Door { name: _, locked: _ } => {
                    doors.insert(lc, pos);
                }
                LocationContent::Entrance => {
                    entrance_position = pos;
                }
                _ => (),
            };
        })
    });

    //Visualize
    for y in 0..(max_y + 1) {
        for x in 0..(max_x + 1) {
            print!(
                "{}",
                char::from(*map.get(&Position::new(x as isize, y as isize)).unwrap())
            );
        }
        println!();
    }

    println!("Entrance position: {:?}", entrance_position);
    println!("Num keys: {}", keys.len());
    println!("Num doors: {}", doors.len());

    let content_types: Vec<LocationContent> = vec![
        LocationContent::Empty,
        LocationContent::Wall,
        LocationContent::Entrance,
    ];

    content_types.iter().for_each(|content| {
        println!(
            "Number of {:?}s: {:?}",
            content,
            map.values().filter(|x| *x == content).count()
        )
    });

    (0, 0)
}
