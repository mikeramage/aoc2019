use crate::intcode;
use crate::utils;
use std::collections::HashMap;
use std::collections::VecDeque;

struct Location {
    position: Position,
    contents: LocationContents,
    path: Path, //Shortest path to location from origin.
}

impl Location {
    pub fn new(position: Position, contents: LocationContents, path: Path) -> Location {
        Location {
            position,
            contents,
            path,
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum LocationContents {
    Wall,
    Empty,
    OxygenSystem,
}

impl From<LocationContents> for isize {
    fn from(location_contents: LocationContents) -> isize {
        match location_contents {
            LocationContents::Wall => 0,
            LocationContents::Empty => 1,
            LocationContents::OxygenSystem => 2,
        }
    }
}

impl TryFrom<isize> for LocationContents {
    type Error = String;
    fn try_from(number: isize) -> Result<Self, Self::Error> {
        match number {
            0 => Ok(LocationContents::Wall),
            1 => Ok(LocationContents::Empty),
            2 => Ok(LocationContents::OxygenSystem),
            other => Err(format!("No match to LocationContents for {other}")),
        }
    }
}

struct Path {
    path: Vec<Direction>,
    reverse_path: Vec<Direction>,
}

impl Path {
    pub fn new(path: Vec<Direction>) -> Path {
        Path {
            path,
            reverse_path: vec![],
        }
    }

    pub fn reverse(&mut self) -> &Vec<Direction> {
        // lazily evaluate the reverse path
        if self.reverse_path.is_empty() {
            for direction in &self.path {
                self.reverse_path.push(direction.opposite());
                //Don't reverse the path - it's a stack. LIFO.
                // self.reverse_path.reverse();
            }
        }
        &self.reverse_path
    }
}

#[derive(Debug, Copy, Clone)]
enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    pub fn opposite(&self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        }
    }
}

impl From<Direction> for isize {
    fn from(direction: Direction) -> isize {
        match direction {
            Direction::North => 1,
            Direction::South => 2,
            Direction::West => 3,
            Direction::East => 4,
        }
    }
}

impl TryFrom<isize> for Direction {
    type Error = String;
    fn try_from(number: isize) -> Result<Self, Self::Error> {
        match number {
            1 => Ok(Direction::North),
            2 => Ok(Direction::South),
            3 => Ok(Direction::West),
            4 => Ok(Direction::East),
            other => Err(format!("No match to MovementCommand for {other}")),
        }
    }
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Copy, Clone, Debug)]
struct Position(i32, i32);

impl Position {
    pub fn new(x: i32, y: i32) -> Position {
        Position(x, y)
    }
}

///Day 15 solution
pub fn day15() -> (usize, usize) {
    let initial_state: Vec<isize> = utils::parse_input_by_sep("input/day15.txt", ',');
    let mut program = intcode::Program::new(&initial_state);

    //Going to implement a breadth-first search. This is a bit different from the traditional search as the droid actually
    //has to move around to explore nodes (locations). Start with a simple approach where the droid
    //backtracks to the origin every time it gets a new node. If that's not performant enough,
    //I can store multiple paths to each location, choose the path that has most recent divergence from the
    //current location and backtrack only to that point along the droid's current path and then jump
    //to that path.

    //This stores known locations - we've not necessarily explored these yet, but they are guaranteed either to
    //have been explored or they've been added to the explore queue.
    let mut known_locations: HashMap<Position, Location> = HashMap::new();

    //This implements the search and returns the shortest path
    let part1 = find_shortest_path_to_oxygen_system(&mut program, &mut known_locations);

    //Adapt the print logic from day 11.
    let (min_x, max_x, min_y, max_y) = (
        known_locations.keys().map(|p| p.0).min().unwrap(),
        known_locations.keys().map(|p| p.0).max().unwrap(),
        known_locations.keys().map(|p| p.1).min().unwrap(),
        known_locations.keys().map(|p| p.1).max().unwrap(),
    );

    for y in (min_y..(max_y + 1)).rev() {
        for x in min_x..(max_x + 1) {
            match known_locations.get(&Position::new(x, y)) {
                Some(location) => match location.contents {
                    LocationContents::Wall => print!("#"),
                    LocationContents::Empty => print!("."),
                    LocationContents::OxygenSystem => print!("O"),
                },
                None => print!(" "),
            }
        }
        println!();
    }

    (part1, 0)
}

fn find_shortest_path_to_oxygen_system(
    program: &mut intcode::Program,
    known_locations: &mut HashMap<Position, Location>,
) -> usize {
    //Stores a queue of locations to explore. Push new locations onto the back, pop the next one from the front
    let mut explore_queue: VecDeque<Position> = VecDeque::new();

    //Create the initial origin location. It's empty.
    let origin = Location::new(
        Position::new(0, 0),
        LocationContents::Empty,
        Path::new(vec![]),
    );
    explore_queue.push_back(origin.position);
    known_locations.insert(Position::new(0, 0), origin);
    let all_directions = vec![
        Direction::North,
        Direction::South,
        Direction::East,
        Direction::West,
    ];

    //Now the main loop
    loop {
        let position_to_explore = &explore_queue.pop_front().unwrap();

        follow_path_to_position(program, *position_to_explore, known_locations);

        for direction in &all_directions {
            check_in_direction(
                *position_to_explore,
                direction,
                program,
                known_locations,
                &mut explore_queue,
            );
        }
    }

    0
}

fn follow_path_to_position(
    program: &mut intcode::Program,
    position: Position,
    known_locations: &HashMap<Position, Location>,
) {
    let location = known_locations.get(&position).unwrap();

    for direction in &location.path.path {
        program.add_input((*direction).into());
        program.run();
        assert_eq!(
            isize::from(LocationContents::Empty),
            program.remove_last_output().unwrap()
        )
    }
}

fn check_in_direction(
    position_to_explore: Position,
    direction: &Direction,
    program: &mut intcode::Program,
    known_locations: &mut HashMap<Position, Location>,
    explore_queue: &mut VecDeque<Position>,
) {
    let new_position = match direction {
        Direction::North => Position::new(position_to_explore.0, position_to_explore.1 + 1),
        Direction::South => Position::new(position_to_explore.0, position_to_explore.1 - 1),
        Direction::West => Position::new(position_to_explore.0 - 1, position_to_explore.1),
        Direction::East => Position::new(position_to_explore.0 + 1, position_to_explore.1),
    };

    if known_locations.contains_key(&new_position) {
        //ignore it - we've alread found it by another path
    } else {
        program.add_input((*direction).into());
        program.run();
        if let Some(output) = program.remove_last_output() {
            // TODO!
        }
    }
}
