use crate::intcode;
use crate::utils;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;

static ALL_DIRECTIONS: [Direction; 4] = [
    Direction::North,
    Direction::South,
    Direction::East,
    Direction::West,
];

#[derive(Debug, Clone)]
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

#[derive(Debug, Copy, Clone, PartialEq)]
enum LocationContents {
    Wall,
    Empty,
    Oxygen,
}

impl From<LocationContents> for isize {
    fn from(location_contents: LocationContents) -> isize {
        match location_contents {
            LocationContents::Wall => 0,
            LocationContents::Empty => 1,
            LocationContents::Oxygen => 2,
        }
    }
}

impl TryFrom<isize> for LocationContents {
    type Error = String;
    fn try_from(number: isize) -> Result<Self, Self::Error> {
        match number {
            0 => Ok(LocationContents::Wall),
            1 => Ok(LocationContents::Empty),
            2 => Ok(LocationContents::Oxygen),
            other => Err(format!("No match to LocationContents for {other}")),
        }
    }
}

#[derive(Debug, Clone)]
struct Path {
    path: Vec<Direction>,
    reverse_path: Vec<Direction>,
}

impl Path {
    pub fn new(path: Vec<Direction>) -> Path {
        let mut reverse_path = vec![];
        for direction in &path {
            reverse_path.push(direction.opposite());
        }
        reverse_path.reverse();

        Path { path, reverse_path }
    }

    pub fn forwards(&self) -> &Vec<Direction> {
        &self.path
    }

    pub fn reverse(&self) -> &Vec<Direction> {
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
                    LocationContents::Empty => {
                        if location.position == Position::new(0, 0) {
                            print!("X")
                        } else {
                            print!(".")
                        }
                    }
                    LocationContents::Oxygen => print!("O"),
                },
                None => print!(" "),
            }
        }
        println!();
    }

    let oxygen_position = known_locations
        .iter()
        .filter(|(_, location)| location.contents == LocationContents::Oxygen)
        .last()
        .unwrap()
        .0;

    let mut part2 = 0;
    let mut oxygenated_areas: HashSet<Position> = HashSet::from([*oxygen_position]);
    loop {
        part2 += 1;

        let mut newly_oxygenated_areas: Vec<Position> = vec![];
        for position in &oxygenated_areas {
            for direction in &ALL_DIRECTIONS {
                let new_position = match direction {
                    Direction::North => Position::new(position.0, position.1 + 1),
                    Direction::South => Position::new(position.0, position.1 - 1),
                    Direction::West => Position::new(position.0 - 1, position.1),
                    Direction::East => Position::new(position.0 + 1, position.1),
                };
                known_locations.entry(new_position).and_modify(|e| {
                    if let LocationContents::Empty = e.contents {
                        e.contents = LocationContents::Oxygen;
                        newly_oxygenated_areas.push(new_position);
                    }
                });
            }
        }

        oxygenated_areas.extend(newly_oxygenated_areas);

        if !known_locations
            .values()
            .any(|location| location.contents == LocationContents::Empty)
        {
            //All empty space now oxygenated
            break;
        }
    }

    (part1, part2)
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

    //println!("Initial explore queue: {:?}", explore_queue);
    //println!("Initial known locations: {:?}", known_locations);
    //println!();

    //Now the main loop
    loop {
        let position_to_explore = explore_queue.pop_front().unwrap();
        //println!("Exploring {:?}", position_to_explore);

        follow_path(program, position_to_explore, known_locations, true);

        for direction in &ALL_DIRECTIONS {
            let found_oxygen_system = check_in_direction(
                position_to_explore,
                direction,
                program,
                known_locations,
                &mut explore_queue,
            );

            if found_oxygen_system {
                return known_locations
                    .get(&position_to_explore)
                    .unwrap()
                    .path
                    .path
                    .len()
                    + 1;
            }
        }

        //retrace steps
        follow_path(program, position_to_explore, known_locations, false);

        //println!("Explore queue now: {:?}", explore_queue);
        //println!("Known locations now: {:?}", known_locations);
        //println!();
    }
}

fn follow_path(
    program: &mut intcode::Program,
    position: Position,
    known_locations: &HashMap<Position, Location>,
    forwards: bool,
) {
    let location = known_locations.get(&position).unwrap();

    let path = if forwards {
        location.path.forwards()
    } else {
        location.path.reverse()
    };

    //println!("  Following path forwards? {}. Path: {:?}", forwards, path);

    for direction in path {
        program.add_input((*direction).into());
        program.run();
        assert_eq!(
            LocationContents::Empty,
            LocationContents::try_from(program.remove_last_output().unwrap()).unwrap()
        )
    }
}

//Checks in specified direction and returns True if we've found the Oxygen System, false otherwise
fn check_in_direction(
    position_to_explore: Position,
    direction: &Direction,
    program: &mut intcode::Program,
    known_locations: &mut HashMap<Position, Location>,
    explore_queue: &mut VecDeque<Position>,
) -> bool {
    // println!(
    //     "  Checking around position: {:?} in direction: {:?}",
    //     position_to_explore, direction
    // );

    let new_position = match direction {
        Direction::North => Position::new(position_to_explore.0, position_to_explore.1 + 1),
        Direction::South => Position::new(position_to_explore.0, position_to_explore.1 - 1),
        Direction::West => Position::new(position_to_explore.0 - 1, position_to_explore.1),
        Direction::East => Position::new(position_to_explore.0 + 1, position_to_explore.1),
    };

    let mut path = known_locations
        .get(&position_to_explore)
        .unwrap()
        .path
        .path
        .clone();
    path.push(*direction);

    let mut found_oxygen_system = false;

    //This closure does a lot of heavy lifting - i.e. it has side effects such as running the program
    //and adding the new location to the explore queue if it's empty.
    known_locations.entry(new_position).or_insert_with(|| {
        program.add_input((*direction).into());
        program.run();

        match LocationContents::try_from(program.remove_last_output().unwrap()) {
            Ok(LocationContents::Wall) => {
                //println!("  Found wall!");
                Location::new(new_position, LocationContents::Wall, Path::new(path))
            }
            Ok(LocationContents::Empty) => {
                //Droid has moved forward - move back
                program.add_input(direction.opposite().into());
                program.run();
                explore_queue.push_back(new_position);
                //println!("  Found empty!");
                Location::new(new_position, LocationContents::Empty, Path::new(path))
            }
            Ok(LocationContents::Oxygen) => {
                //Droid has moved forward - move back.
                program.add_input(direction.opposite().into());
                program.run();
                found_oxygen_system = true;
                //println!("  Found oxygen system!!!");
                Location::new(new_position, LocationContents::Oxygen, Path::new(path))
            }
            Err(err) => panic!("Output doesn't correspond to location! Message: {err}"),
        }
    });

    found_oxygen_system
}
