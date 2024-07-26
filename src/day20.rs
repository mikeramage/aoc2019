use std::cmp::max;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;

use itertools::Itertools;

#[derive(Clone, Copy, Ord, PartialEq, PartialOrd, Eq, Hash, Debug)]
struct Position {
    row: isize,
    col: isize,
}

impl Position {
    fn new(row: isize, col: isize) -> Position {
        Position { row, col }
    }
}

#[derive(Clone, Ord, PartialEq, PartialOrd, Eq, Hash, Debug)]
enum NodeType {
    Void,
    Empty,
    Portal(String),
    Entrance,
    Exit,
    Wall,
}

impl From<char> for NodeType {
    fn from(c: char) -> Self {
        match c {
            //N.B. this is only used in construction; it cannot distinguish portal from empty nodes
            '.' => NodeType::Empty,
            '#' => NodeType::Wall,
            _ => NodeType::Void,
        }
    }
}

#[derive(Clone, Debug)]
struct Node {
    node_type: NodeType,
    portal_partner: Option<Position>,
}

impl Node {
    fn new(node_type: NodeType, portal_partner: Option<Position>) -> Node {
        Node {
            node_type,
            portal_partner,
        }
    }
}

pub fn day20() -> (usize, usize) {
    let input = fs::read_to_string("input/day20.txt").expect("Could not read input file");
    let (ascii_maze, maze, entrance, max_row_index, max_col_index) = parse_input(&input);

    let (path_length, path) =
        path_to_exit(&maze, &entrance, false).expect("Failed to solve maze :(");

    //Quick visualization
    //First convert the path to a map of position to character
    let mut path_viz: HashMap<Position, char> = path
        .iter()
        .tuple_windows()
        .map(|(this, next)| {
            let c = match (next.row - this.row, next.col - this.col) {
                (0, 1) => '>',
                (0, -1) => '<',
                (1, 0) => 'v',
                (-1, 0) => '^',
                _ => '*', //Teleportation!
            };
            (*this, c)
        })
        .collect();

    path_viz.insert(*path.last().unwrap(), '!');

    for row in 0..(max_row_index + 1) {
        for col in 0..(max_col_index + 1) {
            print!(
                "{}",
                match path_viz.get(&Position::new(row, col)) {
                    Some(c) => *c,
                    None => *ascii_maze.get(&Position::new(row, col)).unwrap(),
                }
            );
        }
        println!();
    }

    (path_length, 0)
}

// Returns the length of the path to the exit and the path itself for visualization.
fn path_to_exit(
    maze: &HashMap<Position, Node>,
    entrance: &Position,
    recursive: bool, //Entrance and Exit are only available in level 0. Inner portals go down a level, outer portals back up.
) -> Result<(usize, Vec<Position>), &'static str> {
    //Simple BFS algorithm.

    //Practicing avoiding interior mutability of nodes so use auxiliary structure
    //to store parent relationships and path lengths.
    //Keyed by node position, value is the position of the node's parent and the
    //length of the path to this node, and the level (for recursive

    let mut search_state: HashMap<Position, (Position, usize)> = HashMap::new();
    //Don't add the entrance to the search state as we use the absence of an entry
    //to terminate the evaluation of the path taken through the maze

    let mut frontier: VecDeque<Position> = VecDeque::new();
    frontier.push_back(*entrance);

    let mut explored: HashSet<Position> = HashSet::new();
    let directions = vec![(0, 1), (1, 0), (0, -1), (-1, 0)];
    let mut exit_position: Option<Position> = None;

    while !frontier.is_empty() {
        let current_position = frontier.pop_front().unwrap();
        let current_node = maze.get(&current_position).unwrap();
        let current_path_length = match search_state.get(&current_position) {
            Some((_, path_length)) => *path_length,
            None => 0,
        };
        explored.insert(current_position);

        for (d_row, d_col) in directions.iter() {
            let mut new_position =
                Position::new(current_position.row + d_row, current_position.col + d_col);
            let mut new_node = maze.get(&new_position).unwrap();

            //Check if we're stepping into the void from a portal and update the new node to the
            //teleported-to position.
            if let NodeType::Void = new_node.node_type {
                //Void - stepping into the void from a portal jumps to its partner
                if let Some(partner_pos) = current_node.portal_partner {
                    //Overwrite the position
                    new_position = partner_pos;
                    new_node = maze.get(&new_position).unwrap();
                }
            }

            if explored.contains(&new_position) {
                //Ignore anything we've seen before
                continue;
            } else {
                //Take appropriate action depending on the new node type
                match new_node.node_type {
                    NodeType::Void | NodeType::Wall => continue, //Wall or void - ignore
                    NodeType::Exit => {
                        //Exit -> success! Break out of the loop
                        //Success!
                        exit_position = Some(new_position);
                        search_state
                            .insert(new_position, (current_position, current_path_length + 1));
                        break;
                    }
                    NodeType::Empty | NodeType::Portal(_) => {
                        //Add the new node to the frontier and update search state map
                        frontier.push_back(new_position);
                        search_state
                            .insert(new_position, (current_position, current_path_length + 1));
                    }
                    NodeType::Entrance => {
                        unreachable!("Found Entrance, but we should never revisit!")
                    }
                }
            }
        }
    }

    match exit_position {
        Some(position) => {
            let mut path_to_exit = vec![position];
            let (parent_position, path_length) = search_state.get(&position).expect(
                format!(
                    "Search state unexpectedly has no entry for position {:?}",
                    position
                )
                .as_str(),
            );

            path_to_exit.push(*parent_position);

            let mut state = search_state.get(&parent_position);
            while state.is_some() {
                let parent_position = state.unwrap().0;
                path_to_exit.push(parent_position);
                state = search_state.get(&parent_position);
            }

            path_to_exit.reverse();

            return Ok((*path_length, path_to_exit));
        }
        None => {
            return Err("Could not find a path to the exit");
        }
    }
}

//Returns
// - An ASCII representation of the maze for output
// - the hash map of position to node for the maze
// - the position of the entrance node
// - the max row index
// - the max column index
fn parse_input(
    input: &str,
) -> (
    HashMap<Position, char>,
    HashMap<Position, Node>,
    Position,
    isize,
    isize,
) {
    let mut labels: HashMap<Position, char> = HashMap::new();
    let mut portal_map: HashMap<String, Vec<Position>> = HashMap::new();
    let mut reverse_portal_map: HashMap<Position, String> = HashMap::new();
    let mut maze: HashMap<Position, Node> = HashMap::new();
    let mut max_row_index: isize = 0;
    let mut max_col_index: isize = 0;

    //First pass - create the map of the maze. This will initialize all the nodes. Labels are extracted to a
    //map to be parsed in the next step. Nodes containing labels are void, portal nodes are initialized to
    //empty, to be fixed up in the final step.
    let mut ascii_maze: HashMap<Position, char> = HashMap::new();
    for (row, line) in input.lines().enumerate() {
        for (col, c) in line.chars().enumerate() {
            let pos = Position::new(row as isize, col as isize);
            if c.is_ascii_uppercase() {
                labels.insert(pos, c);
            }
            ascii_maze.insert(pos, c);
            max_row_index = max(max_row_index, row as isize);
            max_col_index = max(max_col_index, col as isize);
        }
    }

    //Step 2 - extract the portal and Entrance/Exit labels and update the node types correspondingly. We'll create
    //a map from label to portal node position so we can match up portal nodes in the final step.
    //
    // The strategy is to process labels that have one other label and one empty node as opposite
    // neighbours. Labels with only a label neigbour (the rest void) are not directly processed, but picked up
    // when processing the partner label.
    for (pos, label) in &labels {
        //Need to parse out the portal labels.

        // Ignore anything on the boundary - they don't need direct processing and this allows us to avoid the
        // boundary conditions in subsequent logic
        if pos.row == 0 || pos.row == max_row_index || pos.col == 0 || pos.col == max_col_index {
            continue;
        }

        let d_pos: Vec<(isize, isize)> = vec![(0, 1), (1, 0), (0, -1), (-1, 0)];
        let mut empty_neighbour: Option<char> = None;
        let mut empty_neighbour_direction = (0, 0);
        let mut label_neighbour: Option<char> = None;
        for (d_row, d_col) in d_pos {
            let c = ascii_maze
                .get(&Position::new(pos.row + d_row, pos.col + d_col))
                .unwrap();
            if *c == '.' {
                empty_neighbour = Some(*c);
                empty_neighbour_direction = (d_row, d_col);
            }

            if let Some(c) = labels.get(&Position::new(pos.row + d_row, pos.col + d_col)) {
                label_neighbour = Some(*c);
            }
        }

        if empty_neighbour.is_some() {
            assert!(label_neighbour.is_some());
            let full_label: String = match empty_neighbour_direction {
                //empty neighbour to right or below so label is neighbour label + label
                (0, 1) | (1, 0) => vec![label_neighbour.unwrap(), *label]
                    .iter()
                    .collect::<String>(),
                //empty neighbour to left or above so label is neighbour label + label
                (0, -1) | (-1, 0) => vec![*label, label_neighbour.unwrap()]
                    .iter()
                    .collect::<String>(),
                _ => unreachable!(),
            };

            portal_map
                .entry(full_label.clone())
                .and_modify(|v| {
                    v.push(Position::new(
                        pos.row + empty_neighbour_direction.0,
                        pos.col + empty_neighbour_direction.1,
                    ))
                })
                .or_insert_with(|| {
                    vec![Position::new(
                        pos.row + empty_neighbour_direction.0,
                        pos.col + empty_neighbour_direction.1,
                    )]
                });

            reverse_portal_map.insert(
                Position::new(
                    pos.row + empty_neighbour_direction.0,
                    pos.col + empty_neighbour_direction.1,
                ),
                full_label,
            );
        };
    }

    let mut entrance_position = Position::new(0, 0);
    //Finally, construct the maze
    for (position, c) in &ascii_maze {
        match reverse_portal_map.get(&position) {
            Some(label) => {
                let node_type = match label.as_str() {
                    "AA" => {
                        entrance_position = *position;
                        NodeType::Entrance
                    }
                    "ZZ" => NodeType::Exit,
                    _ => NodeType::Portal(label.clone()),
                };
                let mut portal_partner = None; //None is relevant for entry/exit.
                for pos in portal_map
                    .get(label)
                    .expect("Key unexpectedly missing from portal map")
                {
                    if *pos != *position {
                        portal_partner = Some(*pos);
                    }
                }
                maze.insert(*position, Node::new(node_type, portal_partner));
            }
            None => {
                maze.insert(*position, Node::new(NodeType::from(*c), None));
            }
        }
    }

    (
        ascii_maze,
        maze,
        entrance_position,
        max_row_index,
        max_col_index,
    )
}
