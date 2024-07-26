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

#[derive(Clone, Copy, Ord, PartialEq, PartialOrd, Eq, Hash, Debug)]
struct HyperPosition {
    position: Position,
    level: isize,
}

impl HyperPosition {
    fn new(position: Position, level: isize) -> HyperPosition {
        HyperPosition { position, level }
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

    let (path_length_part1, path) = path_to_exit(
        &maze,
        &HyperPosition::new(entrance, 0),
        false,
        max_row_index,
        max_col_index,
    )
    .expect("Failed to solve maze :(");

    visualize_solution(&path, &ascii_maze, max_row_index, max_col_index);

    let (path_length_part2, path) = path_to_exit(
        &maze,
        &HyperPosition::new(entrance, 0),
        true,
        max_row_index,
        max_col_index,
    )
    .expect("Failed to solve maze :(");

    visualize_solution(&path, &ascii_maze, max_row_index, max_col_index);

    (path_length_part1, path_length_part2)
    // (path_length_part1, 0)
}

fn visualize_solution(
    path: &[HyperPosition],
    ascii_maze: &HashMap<Position, char>,
    max_row_index: isize,
    max_col_index: isize,
) {
    //First convert the path to a map of position to character
    let mut max_level = 0;
    let mut path_viz: HashMap<HyperPosition, char> = path
        .iter()
        .tuple_windows()
        .map(|(this, next)| {
            let c = match (
                next.position.row - this.position.row,
                next.position.col - this.position.col,
            ) {
                (0, 1) => '>',
                (0, -1) => '<',
                (1, 0) => 'v',
                (-1, 0) => '^',
                _ => '*', //Teleportation!
            };
            //Cheeky side-effect - track the max level we've seen
            max_level = max(max_level, this.level);
            (*this, c)
        })
        .collect();

    path_viz.insert(*path.last().unwrap(), '!');

    for level in 0..(max_level + 1) {
        //TODO change the range above to get a new graph for each level
        println!(
            "\n------------------------ Level {} ------------------------\n",
            level
        );
        for row in 0..(max_row_index + 1) {
            for col in 0..(max_col_index + 1) {
                print!(
                    "{}",
                    match path_viz.get(&HyperPosition::new(Position::new(row, col), level)) {
                        Some(c) => *c,
                        None => *ascii_maze.get(&Position::new(row, col)).unwrap(),
                    }
                );
            }
            println!();
        }
    }
}

// Returns the length of the path to the exit and the path itself for visualization.
fn path_to_exit(
    maze: &HashMap<Position, Node>,
    entrance: &HyperPosition,
    recursive: bool, //Entrance and Exit are only available in level 0. Inner portals go down a level, outer portals back up.
    max_row_index: isize,
    max_col_index: isize,
) -> Result<(usize, Vec<HyperPosition>), &'static str> {
    //Simple BFS algorithm. Note that the maze is a map of position rather than hyperposition to avoid unnecessary duplication/copying.

    //Practicing avoiding interior mutability of nodes so use auxiliary structure
    //to store parent relationships and path lengths.
    //Keyed by node position, value is the position of the node's parent and the
    //length of the path to this node

    let mut search_state: HashMap<HyperPosition, (HyperPosition, usize)> = HashMap::new();
    //Don't add the entrance to the search state as we use the absence of an entry
    //to terminate the evaluation of the path taken through the maze

    let mut frontier: VecDeque<HyperPosition> = VecDeque::new();
    frontier.push_back(*entrance);

    let mut explored: HashSet<HyperPosition> = HashSet::new();
    let directions = vec![(0, 1), (1, 0), (0, -1), (-1, 0)];
    let mut exit_position: Option<HyperPosition> = None;

    // let mut loop_counter = 0;

    while !frontier.is_empty() {
        let current_position = frontier.pop_front().unwrap();
        let current_node = maze.get(&current_position.position).unwrap();
        let current_path_length = match search_state.get(&current_position) {
            Some((_, path_length)) => *path_length,
            None => 0,
        };
        explored.insert(current_position);
        // if matches!(current_node.node_type, NodeType::Portal(_))
        //     && !is_outer_portal(current_position.position, max_row_index, max_col_index)
        // {
        //     //inner portal
        //     inner_portals_explored.insert(current_position.position);
        // }

        for (d_row, d_col) in directions.iter() {
            let mut new_position = HyperPosition::new(
                Position::new(
                    current_position.position.row + d_row,
                    current_position.position.col + d_col,
                ),
                current_position.level,
            );
            let mut new_node = maze.get(&new_position.position).unwrap();

            //Check if we're stepping into the void from a portal and update the new node to the
            //teleported-to position. In recursive mode this results in a level change
            if let NodeType::Void = new_node.node_type {
                //Void - stepping into the void from a portal jumps to its partner
                if let Some(partner_pos) = current_node.portal_partner {
                    //Overwrite the position and figure out the level
                    let new_level = if !recursive {
                        0
                    } else if is_outer_portal(
                        current_position.position,
                        max_row_index,
                        max_col_index,
                    ) {
                        current_position.level - 1
                    } else {
                        current_position.level + 1
                    };
                    new_position = HyperPosition::new(partner_pos, new_level);
                    new_node = maze.get(&new_position.position).unwrap();
                }
            }

            if explored.contains(&new_position) || new_position.level > 25 {
                //Ignore anything we've seen before or anything below depth 25
                //to avoid recursive loops (need to experiment with the value to
                // ensure optimality - I originally set to 100 and the optimum was at max
                // depth of 25, so I've set that to minimize running speed)
                //
                // There's probably a cleverer solution that spots and prunes
                // recursive loops, but this has the benefit of simplicity!
                continue;
            } else {
                //Take appropriate action depending on the new node type
                match new_node.node_type {
                    NodeType::Void | NodeType::Wall => continue, //Wall or void - ignore
                    NodeType::Exit => {
                        if !recursive || new_position.level == 0 {
                            //Exit -> success! Break out of the loop
                            //Success!
                            exit_position = Some(new_position);
                            search_state
                                .insert(new_position, (current_position, current_path_length + 1));
                            break;
                        } else {
                            continue; //Exit is a wall in recursive mode with level > 0
                        }
                    }
                    NodeType::Empty => {
                        //Add the new node to the frontier and update search state map
                        frontier.push_back(new_position);
                        search_state
                            .insert(new_position, (current_position, current_path_length + 1));
                    }
                    NodeType::Portal(_) => {
                        if !recursive
                            || !is_outer_portal(new_position.position, max_row_index, max_col_index)
                            || new_position.level != 0
                        {
                            //Add the new node to the frontier and update search state map
                            frontier.push_back(new_position);
                            search_state
                                .insert(new_position, (current_position, current_path_length + 1));
                        } else {
                            //Outer portal at level 0. Treat as wall
                            continue;
                        }
                    }
                    NodeType::Entrance => {
                        if !recursive {
                            unreachable!("Found Entrance, but we should never revisit!")
                        } else {
                            //Treat as wall
                            continue;
                        }
                    }
                }
            }
        }

        // loop_counter += 1;

        // if loop_counter % 1000 == 0 {
        //     println!("Loop counter: {}", loop_counter);
        //     println!("Current position: {:?}", current_position);
        //     println!("Current path length: {:?}", current_path_length);
        //     println!("Explored set size: {:?}", explored.len());
        //     println!("Frontier size: {:?}", frontier.len());
        // }
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

fn is_outer_portal(position: Position, max_row_index: isize, max_col_index: isize) -> bool {
    if position.row == 2
        || position.row == max_row_index - 2
        || position.col == 2
        || position.col == max_col_index - 2
    {
        return true;
    }
    false
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
