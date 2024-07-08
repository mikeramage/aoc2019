use crate::intcode;
use crate::utils;
use std::collections::HashMap;

#[derive(Debug, Copy, Clone)]
enum Color {
    Black,
    White,
}

impl From<Color> for isize {
    fn from(color: Color) -> isize {
        match color {
            Color::Black => 0,
            Color::White => 1,
        }
    }
}

impl TryFrom<isize> for Color {
    type Error = String;
    fn try_from(number: isize) -> Result<Self, Self::Error> {
        match number {
            0 => Ok(Color::Black),
            1 => Ok(Color::White),
            other => Err(format!("No match to Color for {other}")),
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum Orientation {
    North,
    South,
    East,
    West,
}

#[derive(Debug, Copy, Clone)]
enum Direction {
    Left,
    Right,
}

impl From<Direction> for isize {
    fn from(direction: Direction) -> isize {
        match direction {
            Direction::Left => 0,
            Direction::Right => 1,
        }
    }
}

impl TryFrom<isize> for Direction {
    type Error = String;
    fn try_from(number: isize) -> Result<Self, Self::Error> {
        match number {
            0 => Ok(Direction::Left),
            1 => Ok(Direction::Right),
            other => Err(format!("No match to Direction for {other}")),
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

#[derive(Debug, Copy, Clone)]
enum BrainState {
    AwaitingInput,
    Done,
}

impl TryFrom<intcode::ProgramResult> for BrainState {
    type Error = String;

    #[allow(unreachable_patterns)]
    fn try_from(program_result: intcode::ProgramResult) -> Result<Self, Self::Error> {
        match program_result {
            intcode::ProgramResult::AwaitingInput => Ok(BrainState::AwaitingInput),
            intcode::ProgramResult::Halted => Ok(BrainState::Done),
            other => Err(format!(
                "Program result {:?} does not map to BrainState",
                other
            )),
        }
    }
}

struct RobotOutput {
    direction: Direction,
    color: Color,
    brain_state: BrainState,
}

impl RobotOutput {
    pub fn new(direction: Direction, color: Color, brain_state: BrainState) -> RobotOutput {
        RobotOutput {
            direction,
            color,
            brain_state,
        }
    }
}

struct Robot {
    program: intcode::Program,
}

impl Robot {
    pub fn process_input(&mut self, color: Color) -> Result<RobotOutput, String> {
        self.program.add_input(color.into());
        let brain_state = BrainState::try_from(self.program.run())?;
        let direction = Direction::try_from(
            self.program
                .remove_last_output()
                .ok_or("Robot malfunction: output contains no direction")?,
        )?;
        let new_color = Color::try_from(
            self.program
                .remove_last_output()
                .ok_or("Robot malfunction: output contains no color to paint")?,
        )?;
        Ok(RobotOutput::new(direction, new_color, brain_state))
    }

    pub fn new(program: intcode::Program) -> Robot {
        Robot { program }
    }
}

#[derive(Debug, Clone, Copy)]
struct PanelProperties {
    color: Color,
    times_painted: i32,
}

impl PanelProperties {
    pub fn new(color: Color, times_painted: i32) -> PanelProperties {
        PanelProperties {
            color,
            times_painted,
        }
    }
}

///Day 11 solution
pub fn day11() -> (usize, usize) {
    let initial_state: Vec<isize> = utils::parse_input_by_sep("input/day11.txt", ',');
    let program = intcode::Program::new(&initial_state);
    let mut robot = Robot::new(program);

    //Map of painted panels position to color and number of times painted
    let mut panels: HashMap<Position, PanelProperties> = HashMap::new();
    run_robot(&mut panels, &mut robot);
    let part1 = panels.len();

    //Reinit the panels
    panels = HashMap::new();
    panels.insert(Position::new(0, 0), PanelProperties::new(Color::White, 0));
    let program = intcode::Program::new(&initial_state);
    robot = Robot::new(program);
    run_robot(&mut panels, &mut robot);
    let (min_x, max_x, min_y, max_y) = (
        panels.keys().map(|p| p.0).min().unwrap(),
        panels.keys().map(|p| p.0).max().unwrap(),
        panels.keys().map(|p| p.1).min().unwrap(),
        panels.keys().map(|p| p.1).max().unwrap(),
    );

    for y in (min_y..(max_y + 1)).rev() {
        for x in min_x..(max_x + 1) {
            match panels.get(&Position::new(x, y)) {
                Some(pp) => match pp.color {
                    Color::Black => print!(" "),
                    Color::White => print!("*"),
                },
                None => print!(" "),
            }
        }
        println!();
    }

    (part1, 0)
}

fn run_robot(panels: &mut HashMap<Position, PanelProperties>, robot: &mut Robot) {
    let mut orientation = Orientation::North;
    let mut position = Position::new(0, 0);
    loop {
        let input = match panels.get(&position) {
            Some(panel_properties) => panel_properties.color,
            None => {
                panels.insert(position, PanelProperties::new(Color::Black, 0));
                Color::Black
            }
        };

        let output = robot.process_input(input).expect("Robot broken");
        //Paint
        panels.entry(position).and_modify(|panel_properties| {
            panel_properties.color = output.color;
            panel_properties.times_painted += 1;
        });
        //Turn
        orientation = match output.direction {
            Direction::Left => match orientation {
                Orientation::North => Orientation::West,
                Orientation::South => Orientation::East,
                Orientation::East => Orientation::North,
                Orientation::West => Orientation::South,
            },
            Direction::Right => match orientation {
                Orientation::North => Orientation::East,
                Orientation::South => Orientation::West,
                Orientation::East => Orientation::South,
                Orientation::West => Orientation::North,
            },
        };
        //Move
        position = match orientation {
            Orientation::North => Position::new(position.0, position.1 + 1),
            Orientation::South => Position::new(position.0, position.1 - 1),
            Orientation::East => Position::new(position.0 + 1, position.1),
            Orientation::West => Position::new(position.0 - 1, position.1),
        };

        //Halt condition?
        match output.brain_state {
            BrainState::AwaitingInput => continue,
            BrainState::Done => break,
        }
    }
}
