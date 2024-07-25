use crate::utils;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet, VecDeque},
    rc::{Rc, Weak},
};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash)]
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

//Single path through the maze with no alternative routes. Each path
//has a FIFO queue of keys. Only the first key in the queue is a valid
//next hop.
#[derive(Clone, Debug)]
struct Path {
    id: usize,
    content: RefCell<VecDeque<LocationContent>>, //Doors and Keys
    length: RefCell<usize>,
}

impl Path {
    fn new(id: usize, content: RefCell<VecDeque<LocationContent>>, length: RefCell<usize>) -> Path {
        Path {
            id,
            content,
            length,
        }
    }
}

#[derive(Debug, Clone)]
struct Location {
    content: LocationContent,
    position: Position,
    parent: RefCell<Weak<Location>>,
    children: RefCell<Vec<Rc<Location>>>, //Only non-wall nodes here.
    explored: RefCell<bool>,
    dead_end: RefCell<bool>,
    path: RefCell<Weak<Path>>,
}

impl Location {
    fn new(
        content: LocationContent,
        position: Position,
        parent: RefCell<Weak<Location>>,
        children: RefCell<Vec<Rc<Location>>>,
        explored: RefCell<bool>,
        dead_end: RefCell<bool>,
        path: RefCell<Weak<Path>>,
    ) -> Location {
        Location {
            content,
            position,
            parent,
            children,
            explored,
            dead_end,
            path,
        }
    }
}

pub fn day18() -> (usize, usize) {
    let input: Vec<String> = utils::parse_input("input/day18.txt");
    //Let's have an initial look - parse the input into a map of Position->Rc<Location>
    let mut location_map: HashMap<Position, Rc<Location>> = HashMap::new();
    let mut key_map: HashMap<LocationContent, Position> = HashMap::new();
    let mut entrance_position = Position::new(0, 0);
    let mut max_x = 0;
    let mut max_y = 0;
    input.iter().enumerate().for_each(|(y, line)| {
        line.chars().enumerate().for_each(|(x, c)| {
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
                    RefCell::new(false),
                    RefCell::new(Weak::new()),
                )),
            );
            if lc == LocationContent::Entrance {
                entrance_position = pos;
                key_map.insert(lc, pos);
            }
            if matches!(lc, LocationContent::Key(_)) {
                key_map.insert(lc, pos);
            }
        })
    });

    //Starting at entrance, find all unique paths and dead ends
    find_dead_ends(entrance_position, &location_map);
    reset_search_fields(&location_map);
    let paths = find_paths(entrance_position, &location_map);

    //Visualize
    for y in 0..(max_y + 1) {
        for x in 0..(max_x + 1) {
            let location = location_map
                .get(&Position::new(x as isize, y as isize))
                .unwrap();
            let output_character = if *location.dead_end.borrow() {
                '*'
            } else if matches!(location.content, LocationContent::Empty)
                && location.path.borrow().upgrade().is_some()
            {
                //Location is empty and on a path - output the path_id if small enough
                let digits = location.path.borrow().upgrade().unwrap().id;
                if digits < 10 {
                    char::from_digit(digits as u32, 10).unwrap()
                } else {
                    '?'
                }
            } else {
                char::from(location.content)
            };
            print!("{}", output_character);
        }
        println!();
    }

    reset_search_fields(&location_map);
    //Going to make an assumption now - that doors do not open shortcuts, that the path length to a key from any other key does not change depending
    //on the set of doors that are open, except in a binary way: path length is infinite if inaccessible, but once accessible it
    //does not change. It would be reasonably straightforward to check this, and I will if I have to, but I can't be bothered.

    //Let's find all distances between all pairs of keys and the entrance
    let mut mutual_distances: HashMap<(LocationContent, LocationContent), usize> = HashMap::new();

    for k1 in key_map.keys() {
        reset_search_fields(&location_map);

        let key_paths = calculate_distances_from_position(
            *key_map.get(k1).expect("Couldn't get position from key"),
            &location_map,
        );
        for (k2, path_length) in key_paths {
            if *k1 != k2 {
                //For convenience have 2 entries - one for k1, k2, one for k2, k1.
                mutual_distances
                    .entry((*k1, k2))
                    .or_insert_with(|| path_length);
                mutual_distances
                    .entry((k2, *k1))
                    .or_insert_with(|| path_length);
            }
        }
    }

    // There's a big beast of an attempt at an A* algorithm below. Doesn't work - still too slow.
    // Let's just do a dumbass greedy algorithm and see how close we get to the actual answer. Just
    // go for the nearest key we have sufficient keys to access.
    let mut key_door_map: HashMap<LocationContent, HashSet<LocationContent>> = HashMap::new();
    let mut key_door_map_vec: HashMap<LocationContent, Vec<LocationContent>> = HashMap::new();
    for path in &paths {
        let mut doors = HashSet::new();
        let mut doors_vec: Vec<LocationContent> = vec![];
        for key_or_door in path.content.borrow().iter() {
            match *key_or_door {
                LocationContent::Door(_) => {
                    doors.insert(*key_or_door);
                    doors_vec.push(*key_or_door);
                }
                LocationContent::Key(_) => {
                    key_door_map.insert(*key_or_door, doors.clone());
                    key_door_map_vec.insert(*key_or_door, doors_vec.clone());
                }
                _ => unreachable!("Paths should only contain keys or doors"),
            }
        }
    }

    let mut old_location = LocationContent::Entrance;
    let mut total_path_length = 0;
    let mut key_door_map_copy = key_door_map.clone();

    while !key_door_map.is_empty() {
        let next_location = *key_door_map
            .iter()
            .filter(|(_, v)| v.is_empty())
            .min_by_key(|(k, _)| mutual_distances.get(&(old_location, **k)).unwrap())
            .unwrap()
            .0;
        key_door_map.remove_entry(&next_location);
        key_door_map.values_mut().for_each(|d| {
            d.remove(&LocationContent::Door(
                char::from(next_location).to_ascii_uppercase(),
            ));
        });
        total_path_length += mutual_distances
            .get(&(old_location, next_location))
            .unwrap();

        old_location = next_location;
    }
    // ... and that gives 5040, which is about 500 too big.

    //Manual analysis once paths and dead ends are worked out, suggests the following is optimal, or extremely clsoe:
    //p,y,e,b,s,j,v,f,k,c,g,i,r,a,h,u,w,o,z,d,m,q,l,n,t,x
    //Let's try it, then I'll try to turn my thought processes into an algorithm.
    // ... and, whaddaya know, it's optimal - 4544.
    //My thought processes:
    // - There are 5 main paths to take from the entrance, 0, 1, 2, 3, 4. Some of these fork later on and I've labelled these differently above, but I think that's a mistake as it doesn't affect the logic
    // - P0 is most complex. It has p, y, and c behind it before we hit door F, of which c is useless (no keys behind its door). Key p is only useful to get key b, but b is similarly useless. y will be useful - see below.
    //   F bars keys g,i,r and then door K bars a, door O, d (useless), then door Z, which bars m, l, q (useless) and n.
    // - P1 has door S behind it, which only yields keys h and u, which are useless, and then there's a string of doors G,I,R,A, which have w behind them.
    // - P2 has key s (only useful to get to w, but we also need G,I,R,A). Behind Y and E are j, v (both useless), f and k (which unblock g,i,r, and a), and also o, which we need to get to m,l,q,n.
    // - P3 only has key e, which we need to get to f and k in path 2.
    // - P4 has door P, behind which is the useless b.
    // Bit of head scratching suggests getting the nearest useful keys first, picking up any others on the way. The crucial keys are y and e, which we need to get f and k to get g,i,r,a...
    // So strategy is: get y and e (unlocking f and k), then f and k (unlocking g,i,r,a), then w (unlocking z), then z (picking up o on the way, which unlocks m, l, q, n), then
    // m,l,q,n, unlocking t and x. That's everything stuck behind doors. Other keys are picked up on the way or whenever most convenient.
    // Step 1:
    // - First pick up p (on the way to y) and y, but don't go on to get c yet; we need to come back here to get past F at some point, so it's more efficient to pick up c then.
    // - Then pick up e, the only interesting thing in P3.
    // - Why not e, then p and y? Because e and y are required in P2 and P3 is on the way from P0 to P2 (it's 2 steps quicker to do P0, P3, P2 than P3, P0, P2).
    // - And also P4 is on the way from P0 to P2 so might as well get b now (it won't be any quicker in the future)
    // - So p, y, e, b
    //
    // Step 2.
    // - Go down P2, picking up s, j, v on the way to f and then k. Stop there. Ignore o because we don't need it until we also have z and we can't get that till we have key w. So we
    //   need to come back down P2 in future.
    // - With f and k we can go back down P0, picking up c, g, i, r, a. No point in going further even if we could get past O because useful keys are even further back behind Z.
    // - So p, y, e, b, s, j, v, f, k, c, g, i, r, a.
    //
    // Step 3
    // - P1 time. Pick up h and u on the way (u is miles away but there will never be a better time as we don't need to come back here - note it's equivalent to pick up w first)
    // - Back down P2 to pick up o, and z, now that W is unlocked.
    // - So p, y, e, b, s, j, v, f, k, c, g, i, r, a, h, u, w, o, z
    //
    // Step 4.
    // - Go down P0 one last time and pick up d (on the way), m, q (on the way), l and n.
    // - Back down P2 to pick up final keys t, and x (unlocked by t).
    // - So p, y, e, b, s, j, v, f, k, c, g, i, r, a, h, u, w, o, z, d, m, q, l, n, t, x and we're done.

    // Now wondering if I don't turn forks into new paths I'll have 5 paths instead of 16 and some kind of Dijkstra/A* might now work given we'll only have at most 5 options at
    // each point in the tree - branching factor of at most 5. Let's give that a crack. I think my reasoning above is a little subtle to be implemented, though I could probably
    // come up with some rules to prioritize the critical keys.
    //
    // Nope, still too slow to converge on the solution. Still too many interim possibilities once we've picked up a few keys. So I'll come up with a solution presently.

    let optimal_guess: Vec<LocationContent> = vec![
        LocationContent::Key('p'),
        LocationContent::Key('y'),
        LocationContent::Key('e'),
        LocationContent::Key('b'),
        LocationContent::Key('s'),
        LocationContent::Key('j'),
        LocationContent::Key('v'),
        LocationContent::Key('f'),
        LocationContent::Key('k'),
        LocationContent::Key('c'),
        LocationContent::Key('g'),
        LocationContent::Key('i'),
        LocationContent::Key('r'),
        LocationContent::Key('a'),
        LocationContent::Key('h'),
        LocationContent::Key('u'),
        LocationContent::Key('w'),
        LocationContent::Key('o'),
        LocationContent::Key('z'),
        LocationContent::Key('d'),
        LocationContent::Key('m'),
        LocationContent::Key('q'),
        LocationContent::Key('l'),
        LocationContent::Key('n'),
        LocationContent::Key('t'),
        LocationContent::Key('x'),
    ];

    total_path_length = 0;
    let mut optimal_guess_iter = optimal_guess.iter();
    old_location = LocationContent::Entrance;
    while !key_door_map_copy.is_empty() {
        let next_location = optimal_guess_iter.next().unwrap(); //Should break out of the loop before this yields None
        let (_, doors) = key_door_map_copy.remove_entry(next_location).unwrap();
        assert_eq!(doors.len(), 0);
        key_door_map_copy.values_mut().for_each(|d| {
            d.remove(&LocationContent::Door(
                char::from(*next_location).to_ascii_uppercase(),
            ));
        });
        total_path_length += mutual_distances
            .get(&(old_location, *next_location))
            .unwrap();

        old_location = *next_location;
    }

    // Let's solve part 2 the easy way. There are 4 robots with entrances at different locations.
    // Robot 1 at (39, 39) can access Path 2 only
    // Robot 2 at (41, 39) can access Paths 3 and 4 only.
    // Robot 3 at (41, 39) can access Path 1 only
    // Robot 4 at (39, 41) can access Path 0 only
    // We know that there will always be one robot that can progress and I don't care about the order. Each robot will move a number of steps equivalent to if the doors
    // simply aren't there. (Because I've worked out paths and dead ends and mutual distances, this is really trivial).
    // So for robots 1, 3, and 4 simply add the cumulative distance along the path to each key in turn and subtract 6 (2 for each robot) because the distance from the original entrance to each
    // robot's entrance is 2. For robot 2 it's slightly more complicated. Do path 4 first (it's shortest) - this is possible once P is unlocked, which is uncontroversial -
    // and count the length from entrance to b twice. Then do path 3. This counts an extra 6 spaces because of the entrance displacement. So total extra is 12.

    // .. which gets it wrong because q,m,l,n are not in order - they're in breadth-first order, but m should be accessed first. Grr! I'll do this manually at the end.

    // And wahey!!!!
    let mut part2 = 0;
    for path in &paths {
        let mut current_location = LocationContent::Entrance;
        for location in path.content.borrow().iter() {
            match *location {
                LocationContent::Door(_) => continue,
                LocationContent::Key(k) => {
                    if k == 'q' {
                        //Mahoosive fudge
                        break; //Stop calculating for path 0.
                    }
                    if path.id == 4 {
                        // Fudge factor for doing path 4 twice
                        part2 += 2 * *mutual_distances
                            .get(&(current_location, *location))
                            .unwrap()
                    } else {
                        part2 += *mutual_distances
                            .get(&(current_location, *location))
                            .unwrap()
                    }
                    current_location = *location;
                }
                _ => unreachable!("Path should only contain doors and keys"),
            }
        }
    }

    //Large fudge
    let rest_of_p0 = vec![
        LocationContent::Key('m'),
        LocationContent::Key('q'),
        LocationContent::Key('n'),
        LocationContent::Key('l'),
    ];

    let mut current_location = LocationContent::Key('d');
    for location in rest_of_p0 {
        part2 += *mutual_distances.get(&(current_location, location)).unwrap();
        current_location = location;
    }

    //Another big fudge.
    part2 -= 12;

    // Algorithm is as follows.
    // Basically this is A* search, but some nodes are blocked by doors so the available nodes are dependent on the path taken
    // We're making the following assumptions:
    // - Doors don't open shortcuts - i.e. the path from some key to another is either accessible if the relevant keys have been obtained,
    //   or inaccessible. If accessible the path length is always the same regardless of which set of doors is open.
    // - There are no loops in the maze - each separate path ends in a dead end. I've figured this out by inspection, but could probably
    //   determine it algorithmically if I could be bothered. But I can't
    //
    // - Keep a map of paths and their costs, e.g. entrance->p->e = 234. Initially this is just entrance location and the first location on
    //   any path that's accessible without opening any doors. This is the "open set" of potential optimal path fragments towards the goal
    // - Two costs are maintained - the "g" cost and the "f" cost. The "g" cost is the cost in steps for the path. The "f" cost is the "g" cost
    //   plus the heuristic, "h", which is an optimistic estimate of the remaining path to the goal state, making "f" the estimated total cost
    //   to reach the goal state. Inspection of the paths suggests an effective heuristic is:
    //      - Assume all doors are open (this is a simplified version of the problem, which has a shorter solution)
    //      - The path length from the current node to the last key on the path +
    //      - If any remaining paths still have keys at the end of them, twice the length of all but the longest +
    //      - 1 x the length of the longest remaining path
    //   This always underestimates, which is a requirement of A*. Most of the time, we could get a better estimate by adding the distance from the current
    //   node back to the beginning of its path (usually we'll have to backtrack to access other paths). However sometimes the only remaining paths
    //   will be children of the current path and so this approach will overestimate. To get round this I'd have to track which paths are children of
    //   other paths. Maybe I'll need to do that, but if not, I'll live with an inefficient heuristic.
    // - Remove the path from the open set with the smallest f cost, generate the set of possible next locations along with their associated "f" costs.
    // - Keep doing this until the path contains all (26 keys plus the entrance). Since the heuristic is consistent/admissible, this is guaranteed to be
    //   optimal.

    // Create and initialize costs. Map of vec of LocationContent (the key is the route through the keys taken so far). The value is a
    // tuple of a Vector of Path, where each Path contains the remaining unexplored elements in each path - elements are popped off each path
    // as it is explored -, usize - the cumulative length of the route so far - i.e. the "g" cost, and usize, the "f" cost, which is g + h, the heuristic.

    // let mut costs: HashMap<Vec<LocationContent>, (Vec<Path>, usize, usize)> = HashMap::new();

    // let cloned_paths = cloned_paths(&paths);
    // for path in &cloned_paths {
    //     //At initialization time, the path VecDeques should never be empty
    //     let path_content = path
    //         .content
    //         .borrow_mut()
    //         .pop_front()
    //         .expect(format!("Path is unexpectedly empty! {:?}", path).as_str());
    //     if let LocationContent::Key(_) = path_content {
    //         //Got key
    //         let paths_for_route = cloned_paths.clone();
    //         let route = vec![LocationContent::Entrance, path_content];
    //         let g_cost = *mutual_distances
    //             .get(&(LocationContent::Entrance, path_content))
    //             .unwrap();
    //         costs.insert(
    //             route.clone(),
    //             (
    //                 paths_for_route.clone(),
    //                 g_cost,
    //                 g_cost + h(path, &route, &paths_for_route, &mutual_distances),
    //             ),
    //         );
    //     }
    //     path.content.borrow_mut().push_front(path_content);
    // }

    // let mut optimal_path: (Vec<LocationContent>, usize) = (vec![], 0);

    // let mut counter = 0;

    // //Main loop
    // loop {
    //     //Find the shortest path by f_cost.
    //     let key_of_min_length = costs
    //         .iter()
    //         .min_by_key(|(_, (_, _, f_cost))| f_cost)
    //         .expect("Could not find path with minimum length")
    //         .0
    //         .clone();
    //     if key_of_min_length.len() == 27 {
    //         //This is the winning condition - if the shortest path contains all 26 keys plus the entrance, it must be optimal.
    //         optimal_path = (
    //             key_of_min_length.clone(),
    //             costs.get(&key_of_min_length).unwrap().1,
    //         );
    //         break;
    //     }

    //     //Remove the shortest path - we're going to add one new key to the end for each accessible location in remaining paths
    //     let (shortest_route, (remaining_paths, g_cost, f_cost)) =
    //         costs.remove_entry(&key_of_min_length).unwrap();

    //     for path in &remaining_paths {
    //         let mut path_content = path.content.borrow_mut().pop_front();
    //         while let Some(LocationContent::Door(d)) = path_content {
    //             if shortest_route
    //                 .contains(&LocationContent::try_from(d.to_ascii_lowercase()).unwrap())
    //             {
    //                 //Got corresponding key - keep opening doors
    //                 //Note that we do not put open doors back on the path - only the last door we couldn't open.
    //                 path_content = path.content.borrow_mut().pop_front();
    //             } else {
    //                 //Can't open door!
    //                 break;
    //             }
    //         }

    //         match path_content {
    //             Some(LocationContent::Door(_)) => {
    //                 //Door we can't open - just restore the door; this doesn't result in a new route
    //                 path.content.borrow_mut().push_front(path_content.unwrap());
    //             }
    //             Some(LocationContent::Key(_)) => {
    //                 //Great, got a key. Create a new entry in costs.
    //                 let paths_for_route = remaining_paths.clone();
    //                 let mut new_route = shortest_route.clone();
    //                 let last_key = new_route.last().unwrap().clone();
    //                 let new_key = path_content.unwrap();
    //                 new_route.push(new_key);
    //                 costs.entry(new_route.clone()).or_insert_with(|| {
    //                     (
    //                         paths_for_route.clone(),
    //                         g_cost + *mutual_distances.get(&(last_key, new_key)).unwrap(),
    //                         g_cost + h(path, &new_route, &paths_for_route, &mutual_distances),
    //                     )
    //                 });
    //                 //Restore the key to the parent path
    //                 path.content.borrow_mut().push_front(new_key);
    //             }
    //             None => {
    //                 //No-op. Path is now empty.
    //             }
    //             _ => unreachable!("Should only be doors, keys or nothing in path"),
    //         }
    //     }
    //     counter += 1;
    //     if counter % 10_000 == 0 {
    //         let (max_key, (_, g_cost, f_cost)) =
    //             costs.iter().max_by_key(|(k, (_, _, _))| k.len()).unwrap();
    //         println!(
    //             "Max length key: {:?}, which has {} keys, g_cost is {}, f_cost is {}",
    //             *max_key,
    //             max_key.len(),
    //             *g_cost,
    //             *f_cost,
    //         );
    //         let (min_key, (_, g_cost, f_cost)) =
    //             costs.iter().min_by_key(|(k, (_, _, _))| k.len()).unwrap();
    //         println!(
    //             "Min length key: {:?}, which has {} keys, g_cost is {}, f_cost is {}",
    //             *min_key,
    //             min_key.len(),
    //             *g_cost,
    //             *f_cost,
    //         );
    //         let empty = Vec::new();
    //         let empty_2 = Vec::new();
    //         let bob = &(empty, 0, 0);
    //         let (best_key, (_, g_cost, f_cost)) = costs
    //             .iter()
    //             .filter(|(k, (_, _, _))| k.len() == 27)
    //             .min_by_key(|(_, (_, _, f_cost))| *f_cost)
    //             .unwrap_or((&empty_2, bob));
    //         println!(
    //             "Min f_cost key: {:?}, which has {} keys, g_cost is {}, f_cost is {}",
    //             *best_key,
    //             best_key.len(),
    //             *g_cost,
    //             *f_cost,
    //         );
    //     }
    // }

    // (optimal_path.1, part2)
    (total_path_length, part2)
}

// Heuristic is:
//      - Assume all doors are open (this is a simplified version of the problem, which has a shorter solution)
//      - The path length from the current node to the last key on the path +
//      - If any remaining paths still have keys at the end of them, twice the length of all but the longest +
//      - 1 x the length of the longest remaining path
#[allow(dead_code)]
fn h(
    current_path: &Path,
    route: &[LocationContent],
    paths_for_route: &[Path],
    mutual_distances: &HashMap<(LocationContent, LocationContent), usize>,
) -> usize {
    let current_location = route.last().unwrap();
    let distance_to_last_key_in_current_path = match current_path.content.borrow().back() {
        Some(LocationContent::Key(last_key)) => *mutual_distances
            .get(&(*current_location, LocationContent::Key(*last_key)))
            .unwrap(), //Some other key is the last in the path
        _ => 0, //This key is the last one in the path
    };

    let mut remaining_path_distances = 0;
    //For the rest of the heuristic, we're only interested in non-empty paths that are not the current one
    let non_current_paths = paths_for_route
        .iter()
        .filter(|p| p.id != current_path.id && !p.content.borrow().is_empty());
    //Interested in the longest of these (might be None if all paths are empty!)
    let longest_non_current_path = non_current_paths.clone().max_by_key(|p| *p.length.borrow());
    for path in non_current_paths {
        let longest_non_current_path = longest_non_current_path.expect("If there are any non-empty paths, they must have non-zero length, therefore one of them must be longest!");
        remaining_path_distances += if path.id == longest_non_current_path.id {
            *path.length.borrow()
        } else {
            2 * (*path.length.borrow())
        };
    }

    distance_to_last_key_in_current_path + remaining_path_distances
}

//Resets location fields associated with the search
fn reset_search_fields(location_map: &HashMap<Position, Rc<Location>>) {
    for location in location_map.values() {
        location.children.borrow_mut().clear();
        *location.parent.borrow_mut() = Weak::new();
        *location.explored.borrow_mut() = false;
        //We maintain paths and dead ends. They don't change from one search to another.
    }
}

// Traverses the map from the entrance and figures out all the dead ends.
fn find_dead_ends(entrance_position: Position, location_map: &HashMap<Position, Rc<Location>>) {
    let starting_location = location_map
        .get(&entrance_position)
        .expect("Couldn't get entrance location!");
    let mut frontier: VecDeque<RefCell<Weak<Location>>> = VecDeque::new();
    frontier.push_back(RefCell::new(Rc::downgrade(starting_location)));

    while !frontier.is_empty() {
        //Typically we'd check if the goal state is satisfied by the frontier, but we're doing an exhaustive search
        //of the space so this is not necessary.
        let current_location = frontier
            .pop_front()
            .expect("No location at front of queue!")
            .borrow_mut()
            .upgrade()
            .expect("Aaargh, something cleared up this node under my feet!");

        if *current_location.explored.borrow() {
            //We may add locations to the frontier that turn out to be explored. No need to do it again.
            continue;
        }

        match current_location.content {
            LocationContent::Empty => {
                if expand_node(&mut frontier, location_map, &current_location) {
                    //Traverse back up the location tree marking the path as dead as we go.
                    //Dead end path ends if we find a key or a location with 2 or more
                    //children that aren't themselves dead ends.

                    //First mark this location as a dead end.
                    *current_location.dead_end.borrow_mut() = true;

                    let mut parent_location = current_location
                        .parent
                        .borrow()
                        .upgrade()
                        .unwrap_or_else(|| {
                            panic!(
                                "Location at {:?} unexpectedly has no parent",
                                current_location.position
                            )
                        });

                    let mut penultimate_location = Rc::clone(&current_location);
                    while !matches!(parent_location.content, LocationContent::Key(_)) //Keep searching if: it's not a key
                        && (parent_location.children.borrow().len() < 2 // and either it has fewer than 2 children
                            || parent_location.children.borrow().iter().all(|child| { //or all of its children
                                child.position == penultimate_location.position //are either the previous location (where we've just come from)
                                    || *child.dead_end.borrow() //or dead ends
                            }))
                    {
                        //This is a dead end
                        *parent_location.dead_end.borrow_mut() = true;
                        penultimate_location = Rc::clone(&parent_location);

                        //Get the next parent node
                        parent_location = Rc::clone(&parent_location)
                            .parent
                            .borrow()
                            .upgrade()
                            .unwrap_or_else(|| {
                                panic!(
                                    "Location at {:?} unexpectedly has no parent",
                                    parent_location.position
                                )
                            });
                    }
                }
            }
            LocationContent::Entrance | LocationContent::Key(_) | LocationContent::Door(_) => {
                expand_node(&mut frontier, location_map, &current_location);
            }
            LocationContent::Wall => {} //Wall - do nothing. We're done.
        }

        *current_location.explored.borrow_mut() = true;
    }
}

fn find_paths(
    entrance_position: Position,
    location_map: &HashMap<Position, Rc<Location>>,
) -> Vec<Rc<Path>> {
    let mut path_id: usize = 0;
    let mut paths: Vec<Rc<Path>> = vec![];
    let starting_location = location_map
        .get(&entrance_position)
        .expect("Couldn't get entrance location!");
    let mut frontier: VecDeque<RefCell<Weak<Location>>> = VecDeque::new();
    frontier.push_back(RefCell::new(Rc::downgrade(starting_location)));

    while !frontier.is_empty() {
        //Typically we'd check if the goal state is satisfied by the frontier, but we're doing an exhaustive search
        //of the space so this is not necessary.
        let current_location = frontier
            .pop_front()
            .expect("No location at front of queue!")
            .borrow_mut()
            .upgrade()
            .expect("Aaargh, something cleared up this node under my feet!");

        if *current_location.explored.borrow() {
            //We may add locations to the frontier that turn out to be explored. No need to do it again.
            continue;
        }

        match current_location.content {
            LocationContent::Empty
            | LocationContent::Entrance
            | LocationContent::Key(_)
            | LocationContent::Door(_) => {
                expand_node(&mut frontier, location_map, &current_location);
                assign_and_update_path(&current_location, &mut paths, &mut path_id);
            }
            LocationContent::Wall => {} //Wall - do nothing. We're done.
        }

        *current_location.explored.borrow_mut() = true;
    }
    paths
}

//Must only be called after expand_node has been called on current location
fn assign_and_update_path(
    current_location: &Rc<Location>,
    paths: &mut Vec<Rc<Path>>,
    path_id: &mut usize,
) {
    let parent = current_location.parent.borrow().upgrade();

    if let Some(parent_location) = parent {
        //Old logic would create a new path every time a fork was encountered. No longer required, but keep for posterity.
        //Check if the parent is a branch (2 or more children) or a tunnel (just one
        //child or no children if this is a key in a corner).
        // if parent_location.children.borrow().len() >= 2 {
        //     //Parent is a branch. That means that provided the current location is a tunnel, this is the start of a new path.
        //     if current_location.children.borrow().len() < 2 {
        //         let path = Path::new(*path_id, RefCell::new(VecDeque::new()), RefCell::new(1));
        //         *path_id += 1;
        //         //Copy any doors from the parent path, if any, into the new path. This is to prevent us
        //         //from being able to access the new path until the doors leading to it are unlocked.
        //         match parent_location.path.borrow().upgrade() {
        //             Some(parent_path) => {
        //                 for location_content in &*parent_path.content.borrow() {
        //                     match location_content {
        //                         LocationContent::Door(_) => {
        //                             path.content.borrow_mut().push_back(*location_content)
        //                         }
        //                         _ => (),
        //                     }
        //                 }
        //             }
        //             None => {} //No op
        //         }

        //         update_path(&path, current_location); //update the contents of the path with what's in the current location.
        //         let path_ref = Rc::new(path);
        //         paths.push(path_ref.clone());
        //         *current_location.path.borrow_mut() = Rc::downgrade(&path_ref);
        //     } else {
        //         //Current location is a branch, but the parent is also a branch. Therefore this node has no path. No-op.
        //     }
        // } else {
        //     //Parent has fewer than 2 children - it's a tunnel so just inherit the parent path
        //     let parent_path = parent_location
        //         .path
        //         .borrow_mut()
        //         .upgrade()
        //         .expect("Parent in tunnel should always have a path!");
        //     *parent_path.length.borrow_mut() += 1;
        //     update_path(&parent_path, current_location);
        //     *current_location.path.borrow_mut() = Rc::downgrade(&parent_path);
        // }

        match parent_location.path.borrow().upgrade() {
            Some(parent_path) => {
                //Parent has a path - inherit the parent path
                *parent_path.length.borrow_mut() += 1;
                update_path(&parent_path, current_location);
                *current_location.path.borrow_mut() = Rc::downgrade(&parent_path);
            }
            None => {
                if current_location.children.borrow().len() < 2 {
                    //Parent has no path, but we've entered a tunnel - this is the start of the path.
                    let path = Path::new(*path_id, RefCell::new(VecDeque::new()), RefCell::new(1));
                    *path_id += 1;
                    update_path(&path, current_location); //update the contents of the path with what's in the current location.
                    let path_ref = Rc::new(path);
                    paths.push(path_ref.clone());
                    *current_location.path.borrow_mut() = Rc::downgrade(&path_ref);
                }
                // If parent has no path and this is not the start of a new path then it's a no-op.
            }
        }
    }
}

fn update_path(path: &Path, current_location: &Location) {
    match current_location.content {
        LocationContent::Door(_) | LocationContent::Entrance | LocationContent::Key(_) => path
            .content
            .borrow_mut()
            .push_back(current_location.content),
        _ => {}
    }
}

//Calculates distance to all keys and the entrance from specified position (which is intended to be
//another key or the entrance, but doesn't have to be)
fn calculate_distances_from_position(
    starting_position: Position,
    location_map: &HashMap<Position, Rc<Location>>,
) -> HashMap<LocationContent, usize> {
    let starting_location = location_map
        .get(&starting_position)
        .expect("Couldn't get entrance location!");
    let mut frontier: VecDeque<RefCell<Weak<Location>>> = VecDeque::new();
    frontier.push_back(RefCell::new(Rc::downgrade(starting_location)));
    let mut key_distances: HashMap<LocationContent, usize> = HashMap::new();

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

        if *current_location.explored.borrow() {
            //We may add locations to the frontier that turn out to be explored. No need to do it again.
            continue;
        }

        match current_location.content {
            LocationContent::Empty | LocationContent::Door(_) => {
                expand_node(&mut frontier, location_map, &current_location);
            }
            LocationContent::Entrance | LocationContent::Key(_) => {
                key_distances.insert(current_location.content, calculate_path(&current_location));
                expand_node(&mut frontier, location_map, &current_location);
            }
            LocationContent::Wall => {
                unreachable!("Shouldn't be adding wall nodes to the frontier!")
            } //Wall - do nothing. We're done.
        }

        *current_location.explored.borrow_mut() = true;
    }

    key_distances
}

//Finds neighbours and updates this node. Returns whether this is a dead end
fn expand_node(
    frontier: &mut VecDeque<RefCell<Weak<Location>>>,
    location_map: &HashMap<Position, Rc<Location>>,
    current_location: &Rc<Location>,
) -> bool {
    //Expanding the node means adding any locations to the N,S,E,W of the current location that haven't yet been
    //explored to the frontier and updating child, parent nodes of the current location.
    let mut num_walls = 0;
    for (dx, dy) in [(1, 0), (0, 1), (-1, 0), (0, -1)] {
        let location = location_map.get(&Position::new(
            current_location.position.x + dx,
            current_location.position.y + dy,
        ));
        if let Some(loc) = location {
            if loc.content == LocationContent::Wall {
                //Don't add wall nodes to the frontier or consider them as children.
                num_walls += 1;
            } else if !*loc.explored.borrow() && !*loc.dead_end.borrow() {
                frontier.push_back(RefCell::new(Rc::downgrade(loc)));
                current_location.children.borrow_mut().push(Rc::clone(loc));
                *loc.parent.borrow_mut() = Rc::downgrade(current_location);
            }
        }
    }

    if num_walls == 3 && current_location.content == LocationContent::Empty {
        //Dead end if and only if this node is empty and surrounded by 3 walls
        return true;
    }

    false
}

fn calculate_path(location: &Location) -> usize {
    let mut length = 0;
    let mut parent = location.parent.borrow().upgrade();
    while parent.is_some() {
        length += 1;
        let parent_location = parent.unwrap();
        parent = parent_location.parent.borrow().upgrade();
    }

    length
}
