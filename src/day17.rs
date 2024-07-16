use crate::intcode;
use crate::utils;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Orientation {
    Up, //Up is pointing in negative y direction (yes, I know)
    Down,
    Left,
    Right,
    Tumbling,
}

impl Display for Orientation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let output: &str = match self {
            Orientation::Down => "v",
            Orientation::Up => "^",
            Orientation::Left => "<",
            Orientation::Right => ">",
            Orientation::Tumbling => "X",
        };
        write!(f, "{output}")
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct Position {
    //Coordinates start at (0, 0) in upper left. y increases down the way.
    x: isize,
    y: isize,
}

impl Position {
    pub fn new(x: isize, y: isize) -> Position {
        Position { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Item {
    Scaffold,
    Empty,
    Robot(Orientation),
}

impl Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Item::Scaffold => write!(f, "#"),
            Item::Empty => write!(f, "."),
            Item::Robot(orientation) => orientation.fmt(f),
        }
    }
}

impl TryFrom<isize> for Item {
    type Error = String;
    fn try_from(item: isize) -> Result<Self, Self::Error> {
        match item {
            35 => Ok(Item::Scaffold),
            46 => Ok(Item::Empty),
            94 => Ok(Item::Robot(Orientation::Up)),
            118 => Ok(Item::Robot(Orientation::Down)),
            60 => Ok(Item::Robot(Orientation::Left)),
            62 => Ok(Item::Robot(Orientation::Right)),
            88 => Ok(Item::Robot(Orientation::Tumbling)),
            other => Err(format!("Bad item identifier: {}", other)),
        }
    }
}

pub fn day17() -> (usize, usize) {
    let input: Vec<isize> = utils::parse_input_by_sep("input/day17.txt", ',');
    let mut program = intcode::Program::new(&input);
    program.run();

    let mut scaffold_map: HashMap<Position, Item> = HashMap::new();
    let mut robot_start = Position::new(0, 0);
    let mut x = 0;
    let mut y = 0;

    //Visualize the map and assign locations to coordinate map
    for output in program.outputs() {
        if *output == 10 {
            //newline
            println!();
            y += 1;
            x = 0;
        } else {
            let item = Item::try_from(*output).expect("Bad item");
            scaffold_map.insert(Position::new(x, y), item);
            print!("{}", item);
            if let Item::Robot(_) = item {
                robot_start = Position::new(x, y);
            }
            x += 1;
        }
    }

    //The above should have found a non-zero position for the robot starting point.
    assert_ne!(robot_start, Position::new(0, 0));

    //Ok, now do part1. Intersection is any scaffold in the scaffold map whose immediate non-diagonal neighbours
    //are also scaffold. For any intersection, multiply x and y, then sum over intersections. Easy!
    let part1: isize = scaffold_map
        .iter()
        .filter(|(position, item)| {
            **item == Item::Scaffold && {
                let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)];
                directions.iter().all(|(dx, dy)| {
                    check_scaffold(
                        &scaffold_map,
                        &Position::new(position.x + *dx, position.y + *dy),
                    )
                })
            }
        })
        .map(|(position, _)| position.x * position.y)
        .sum();

    //For part2 we need to find a path we can break into 3 subroutines of 20 instructions of 20 characters or fewer
    //First, just take a naive approach. From inspection you can just follow the path around like a string, going
    //straight across all intersections only turning when required.
    // let path: Vec<String> = calculate_path(&scaffold_map, robot_start);

    // println!("Path: {:?}", path);

    //This gives: ["L", "6", "R", "12", "R", "8", "R", "8", "R", "12", "L", "12", "R", "8", "R", "12", "L", "12", "L", "6", "R", "12", "R", "8", "R", "12", "L", "12", "L", "4", "L", "4", "L", "6", "R", "12", "R", "8", "R", "12", "L", "12", "L", "4", "L", "4", "L", "6", "R", "12", "R", "8", "R", "12", "L", "12", "L", "4", "L", "4", "R", "8", "R", "12", "L", "12"]
    //which can be split up by inspection:
    // A = L6, R12, R8
    // B = R8, R12, L12
    // C = R12, L12, L4, L4
    // path = A, B, B, A, C, A, C, A, C, B
    // So the naive approach works. Was I meant to do some programming here to algorithmically find A, B and C from the path?
    // ... Yep, probably. I should come back and do some kind of greedy algorithm for breaking down the instructions into A, B and C.
    program.initialize(&input);
    program.set_value_at(0, 2);
    let program_result = program.run();
    assert_eq!(intcode::ProgramResult::AwaitingInput, program_result);

    set_inputs_and_run(
        &mut program,
        prepare_ascii_input("A,B,B,A,C,A,C,A,C,B\n"),
        intcode::ProgramResult::AwaitingInput,
    );
    set_inputs_and_run(
        &mut program,
        prepare_ascii_input("L,6,R,12,R,8\n"),
        intcode::ProgramResult::AwaitingInput,
    );
    set_inputs_and_run(
        &mut program,
        prepare_ascii_input("R,8,R,12,L,12\n"),
        intcode::ProgramResult::AwaitingInput,
    );
    set_inputs_and_run(
        &mut program,
        prepare_ascii_input("R,12,L,12,L,4,L,4\n"),
        intcode::ProgramResult::AwaitingInput,
    );
    set_inputs_and_run(
        &mut program,
        prepare_ascii_input("n\n"),
        intcode::ProgramResult::Halted,
    );

    (part1 as usize, *program.outputs().last().unwrap() as usize)
}

fn prepare_ascii_input(input: &str) -> Vec<isize> {
    input.bytes().map(|b| b as isize).rev().collect()
}

fn set_inputs_and_run(
    program: &mut intcode::Program,
    inputs: Vec<isize>,
    expected_result: intcode::ProgramResult,
) {
    program.set_inputs(inputs);
    let result = program.run();
    assert_eq!(expected_result, result);
}

#[allow(dead_code)]
fn calculate_path(scaffold_map: &HashMap<Position, Item>, robot_start: Position) -> Vec<String> {
    //Initialize the output path, current orientation, current position and move counter
    let mut path = vec![];
    let mut current_orientation = match scaffold_map.get(&robot_start).unwrap() {
        Item::Robot(orientation) => *orientation,
        _ => panic!("Robot not at start location!!!"),
    };
    let mut current_position = robot_start;
    let mut move_counter = 0;

    loop {
        //Update orientation if required
        if !update_orientation(
            &mut current_orientation,
            scaffold_map,
            &mut current_position,
            &mut move_counter,
            &mut path,
        ) {
            break;
        }

        //Now we're pointing in the right direction, let's move - increment the move counter and update the position!
        move_counter += 1;

        let (dx, dy) = match current_orientation {
            Orientation::Up => (0, -1),
            Orientation::Down => (0, 1),
            Orientation::Left => (-1, 0),
            Orientation::Right => (1, 0),
            Orientation::Tumbling => panic!("AAAAAAGH! Tumbling through space"),
        };

        current_position = Position::new(current_position.x + dx, current_position.y + dy);
    }

    path
}

fn check_scaffold(scaffold_map: &HashMap<Position, Item>, position: &Position) -> bool {
    matches!(scaffold_map.get(position), Some(&Item::Scaffold))
}

fn update_path_and_reset_counter(move_counter: &mut isize, path: &mut Vec<String>) {
    if *move_counter > 0 {
        path.push(move_counter.to_string());
        *move_counter = 0;
    }
}

fn update_orientation(
    current_orientation: &mut Orientation,
    scaffold_map: &HashMap<Position, Item>,
    current_position: &mut Position,
    move_counter: &mut isize,
    path: &mut Vec<String>,
) -> bool {
    let (dx, dy) = match current_orientation {
        Orientation::Up => (0, -1),
        Orientation::Down => (0, 1),
        Orientation::Left => (-1, 0),
        Orientation::Right => (1, 0),
        Orientation::Tumbling => panic!("Aaaargh! Tumbling into space"),
    };

    let forward_pos = Position::new(current_position.x + dx, current_position.y + dy);

    if check_scaffold(scaffold_map, &forward_pos) {
        //Scaffold straight ahead - carry on
        return true;
    }

    // No scaffolding ahead; need to change orientation. Check to the left and right (no going back!)
    let (right_dx, right_dy, left_dx, left_dy) = match current_orientation {
        Orientation::Up => (1, 0, -1, 0),
        Orientation::Down => (-1, 0, 1, 0),
        Orientation::Left => (0, -1, 0, 1),
        Orientation::Right => (0, 1, 0, -1),
        Orientation::Tumbling => panic!("Aaaargh! Tumbling into space"),
    };

    let right_pos = Position::new(current_position.x + right_dx, current_position.y + right_dy);
    let left_pos = Position::new(current_position.x + left_dx, current_position.y + left_dy);

    if check_scaffold(scaffold_map, &right_pos) {
        //Scaffolding to the right
        update_path_and_reset_counter(move_counter, path);
        path.push("R".to_string());
        *current_orientation = match *current_orientation {
            Orientation::Up => Orientation::Right,
            Orientation::Down => Orientation::Left,
            Orientation::Left => Orientation::Up,
            Orientation::Right => Orientation::Down,
            Orientation::Tumbling => panic!("AAAAARGH! Tumbling into space"),
        }
    } else if check_scaffold(scaffold_map, &left_pos) {
        //Scaffolding to left
        update_path_and_reset_counter(move_counter, path);
        path.push("L".to_string());
        *current_orientation = match *current_orientation {
            Orientation::Up => Orientation::Left,
            Orientation::Down => Orientation::Right,
            Orientation::Left => Orientation::Down,
            Orientation::Right => Orientation::Up,
            Orientation::Tumbling => panic!("AAAAARGH! Tumbling into space"),
        }
    } else {
        //No scaffolding! Must be end
        update_path_and_reset_counter(move_counter, path);
        return false;
    }

    true
}
