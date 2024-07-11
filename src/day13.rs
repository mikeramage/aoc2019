use crate::intcode;
use crate::utils;
use std::collections::HashMap;
use std::fmt;
#[allow(unused_imports)]
use std::io::stdin;

#[derive(Clone, Copy, Debug, Hash, PartialEq, PartialOrd, Ord, Eq)]
struct Tile {
    tile_type: TileType,
}

impl Tile {
    pub fn new(tile_type_id: isize) -> Tile {
        Tile {
            tile_type: TileType::try_from(tile_type_id).unwrap_or_else(|err| {
                panic!(
                    "Bad input to tile constructor: {}. Error: {}",
                    tile_type_id, err
                )
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, PartialOrd, Ord, Eq)]
struct Position {
    x: isize,
    y: isize,
}

impl Position {
    pub fn new(x: isize, y: isize) -> Position {
        Position { x, y }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, PartialOrd, Ord, Eq)]
enum TileType {
    Empty,
    Wall,
    Block,
    Paddle,
    Ball,
}

impl TryFrom<isize> for TileType {
    type Error = String;
    fn try_from(id: isize) -> Result<Self, Self::Error> {
        match id {
            0 => Ok(Self::Empty),
            1 => Ok(Self::Wall),
            2 => Ok(Self::Block),
            3 => Ok(Self::Paddle),
            4 => Ok(Self::Ball),
            other => Err(format!("Unrecognized tile: {}", other)),
        }
    }
}

impl fmt::Display for TileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tile = match self {
            Self::Empty => " ",
            Self::Wall => "W",
            Self::Block => "B",
            Self::Paddle => "_",
            Self::Ball => "O",
        };
        write!(f, "{}", tile)
    }
}

#[allow(unused_variables)]
fn draw(
    program: &intcode::Program,
    tiles: &mut HashMap<Position, Tile>,
    min_x: isize,
    max_x: isize,
    min_y: isize,
    max_y: isize,
) {
    //As the output grows this function becomes increasingly inefficient as each time the program
    //is run the new output is a delta to the previous one. Maybe fix this later.
    program.outputs().chunks(3).for_each(|chunk| {
        if chunk[0] != -1 {
            tiles.insert(Position::new(chunk[0], chunk[1]), Tile::new(chunk[2]));
        }
    });

    //Uncomment this to see the game in action

    // for y in min_y..max_y {
    //     for x in min_x..max_x {
    //         match tiles.get(&Position::new(x, y)) {
    //             Some(tile) => print!("{}", tile.tile_type),
    //             None => print!("{}", TileType::Empty),
    //         }
    //     }
    //     println!();
    // }
}

pub fn day13() -> (usize, usize) {
    let initial_state: Vec<isize> = utils::parse_input_by_sep("input/day13.txt", ',');
    let mut program = intcode::Program::new(&initial_state);
    program.run();
    let part1 = program
        .outputs()
        .chunks(3)
        .filter_map(|x| if x[2] == 2 { Some(1) } else { None })
        .count();

    // OK - that was easy, start part two, remember to play for free :)
    //First learn the boundaries of the game max/min x and y - we assume this doesn't change for the lifetime of
    //the game (note that the scoreboard instruction doesn't appear until you "insert quarters", set the first
    //element of program to 2 - this means we don't accidentally set min_x = -1.)
    let mut tiles: HashMap<Position, Tile> = HashMap::new();
    program.outputs().chunks(3).for_each(|chunk| {
        tiles.insert(Position::new(chunk[0], chunk[1]), Tile::new(chunk[2]));
    });

    let (min_x, max_x, min_y, max_y) = tiles.keys().fold(
        (isize::MAX, isize::MIN, isize::MAX, isize::MIN),
        |(min_x, max_x, min_y, max_y), position| {
            (
                min_x.min(position.x),
                max_x.max(position.x),
                min_y.min(position.y),
                max_y.max(position.y),
            )
        },
    );

    // Assert the thing I said above
    assert!(min_x >= 0);

    program.initialize(&initial_state);
    program.set_value_at(0, 2);
    program.run();

    draw(&program, &mut tiles, min_x, max_x, min_y, max_y);

    //Now set the game loop
    let mut remaining_blocks = part1;

    while remaining_blocks > 0 {
        //Uncomment this and remove the automated logic to play!
        // let stdin = stdin();
        // let mut user_input: String = String::new();
        // stdin
        //     .read_line(&mut user_input)
        //     .unwrap_or_else(|err| panic!("Failed to get user input: {err}"));

        // let joystick_direction: char = user_input.trim().parse().unwrap_or_else(|err| {
        //     println!("Bad input: {err}");
        //     's' //permissive - assume no joystick movement.
        // });

        // let program_input = match joystick_direction {
        //     'a' => -1,
        //     'd' => 1,
        //     _ => 0,
        // };

        //Automated logic starts here.
        let paddle_x = tiles
            .iter()
            .filter_map(|(position, tile)| match tile.tile_type {
                TileType::Paddle => Some(position.x),
                _ => None,
            })
            .last()
            .unwrap();

        //Should write a common function - I've used this iterator pattern a few times, but I'm lazy and
        //I'm not reusing this code!
        let ball_x = tiles
            .iter()
            .filter_map(|(position, tile)| match tile.tile_type {
                TileType::Ball => Some(position.x),
                _ => None,
            })
            .last()
            .unwrap();

        let program_input = (ball_x - paddle_x).signum();
        //End of automated logic
        program.clear_outputs();
        program.add_input(program_input);
        program.run();
        draw(&program, &mut tiles, min_x, max_x, min_y, max_y);

        remaining_blocks = tiles
            .iter()
            .filter_map(|(_, tile)| match tile.tile_type {
                TileType::Block => Some(1),
                _ => None,
            })
            .count();
    }

    //Extract the score from the outputs
    let part2 = program
        .outputs()
        .chunks(3)
        .filter_map(|x| if x[0] == -1 { Some(x[2]) } else { None })
        .last()
        .unwrap();

    (part1, part2 as usize)
}
