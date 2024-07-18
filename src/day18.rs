use crate::utils;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet, VecDeque},
    rc::{Rc, Weak},
    thread::current,
};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Hash)]
enum LocationContent {
    Empty,
    Wall,
    Entrance,
    Key(char),
    Door(char), //char is the name of the door, bool is whether it's locked: true if so, false if not.
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
                    Ok(LocationContent::Key(other))
                } else if other.is_ascii_uppercase() {
                    Ok(LocationContent::Door(other))
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
            LocationContent::Key(name) => name,
            LocationContent::Door(name) => name,
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

#[derive(Debug, Clone)]
struct Location {
    content: LocationContent,
    position: Position,
    parent: RefCell<Weak<Location>>,
    children: RefCell<Vec<Rc<Location>>>,
    explored: RefCell<bool>,
}

impl Location {
    fn new(
        content: LocationContent,
        position: Position,
        parent: RefCell<Weak<Location>>,
        children: RefCell<Vec<Rc<Location>>>,
        explored: RefCell<bool>,
    ) -> Location {
        Location {
            content,
            position,
            parent,
            children,
            explored,
        }
    }
}

pub fn day18() -> (usize, usize) {
    let input: Vec<String> = utils::parse_input("input/day18.txt");
    //Let's have an initial look - parse the input into a map of Position->Rc<Location>
    let mut location_map: HashMap<Position, Rc<Location>> = HashMap::new();
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
            location_map.insert(
                pos,
                Rc::new(Location::new(
                    lc,
                    pos,
                    RefCell::new(Weak::new()),
                    RefCell::new(vec![]),
                    RefCell::new(false),
                )),
            );
            match lc {
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
                char::from(
                    location_map
                        .get(&Position::new(x as isize, y as isize))
                        .unwrap()
                        .content
                )
            );
        }
        println!();
    }

    println!("Entrance position: {:?}", entrance_position);

    //OK - let's just do a simple breadth-first search, treating doors as dead ends, to see which keys
    //are immediately available and what the path lengths of each are, and what the initial set of blocking
    //doors are.
    let root = location_map
        .get(&entrance_position)
        .expect("Couldn't get entrance location!");
    let mut frontier: VecDeque<RefCell<Weak<Location>>> = VecDeque::new();
    frontier.push_back(RefCell::new(Rc::downgrade(&root)));

    // breadth first loop
    let (key_paths, blocking_doors) = find_keys_and_doors(&mut frontier, &location_map);
    println!("Keys: {:?}", key_paths);
    println!("Doors: {:?}", blocking_doors);

    (0, 0)
}

fn find_keys_and_doors(
    frontier: &mut VecDeque<RefCell<Weak<Location>>>,
    location_map: &HashMap<Position, Rc<Location>>,
) -> (HashMap<LocationContent, usize>, HashSet<LocationContent>) {
    let mut key_paths: HashMap<LocationContent, usize> = HashMap::new();
    let mut blocking_doors: HashSet<LocationContent> = HashSet::new();

    // While we've still got paths to check
    while !frontier.is_empty() {
        //Typically we'd check if the goal state is satisfied by the frontier, but we're doing an exhaustive search
        //of the space so this is not necessary.
        let current_location = frontier
            .pop_front()
            .expect("No location at front of queue!")
            .borrow_mut()
            .upgrade()
            .expect("Aaargh, something cleared up this node under my feet!");
        match current_location.content {
            LocationContent::Empty | LocationContent::Entrance => {
                expand_node(frontier, location_map, &current_location)
            }
            LocationContent::Door(_) => {
                blocking_doors.insert(current_location.content);
            }
            LocationContent::Key(_) => {
                key_paths.insert(current_location.content, path_length(&current_location));
                expand_node(frontier, location_map, &current_location)
            }
            LocationContent::Wall => {} //Wall - do nothing. We're done.
        }

        *current_location.explored.borrow_mut() = true;
    }

    (key_paths, blocking_doors)
}

//Finds neighbours
fn expand_node(
    frontier: &mut VecDeque<RefCell<Weak<Location>>>,
    location_map: &HashMap<Position, Rc<Location>>,
    current_location: &Rc<Location>,
) {
    //Expanding the node means adding any locations to the N,S,E,W of the current location that haven't yet been
    //explored to the frontier and updating child, parent nodes of the current location.
    for (dx, dy) in [(1, 0), (0, 1), (-1, 0), (0, -1)] {
        let location = location_map.get(&Position::new(
            current_location.position.x + dx,
            current_location.position.y + dy,
        ));
        if let Some(loc) = location {
            if !*loc.explored.borrow() {
                frontier.push_back(RefCell::new(Rc::downgrade(loc)));
                current_location.children.borrow_mut().push(Rc::clone(loc));
                *loc.parent.borrow_mut() = Rc::downgrade(current_location);
            }
        }
    }
}

fn path_length(location: &Location) -> usize {
    let mut length = 0;
    let mut parent = location.parent.borrow().upgrade();
    while parent.is_some() {
        length += 1;
        parent = parent.unwrap().parent.borrow().upgrade();
    }

    length
}
