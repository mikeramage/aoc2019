use crate::utils;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug)]
struct Object {
    orbits: Option<String>,
    orbited_by: Vec<String>,
}

impl Object {
    fn new(orbits: Option<String>, orbited_by: Vec<String>) -> Object {
        Object { orbits, orbited_by }
    }
}

///Day 6 solution
pub fn day6() -> (usize, usize) {
    let orbit_string = utils::parse_input::<String>("input/day6.txt");
    let orbits = parse_orbits(&orbit_string);
    let orbit_map = build_orbit_map(&orbits);
    let (direct_orbits, indirect_orbits) = count_orbits(&orbit_map);

    // let num_transfers = num_transfers(&orbit_map);
    let num_transfers = num_transfers(&orbit_map).expect("Aargh");
    (direct_orbits + indirect_orbits, num_transfers as usize)
}

fn parse_orbits(orbits: &[String]) -> Vec<Vec<String>> {
    orbits
        .iter()
        .map(|x| x.split(')').map(|s| String::from(s)).collect())
        .collect()
}

fn build_orbit_map(orbits: &[Vec<String>]) -> HashMap<String, Object> {
    let mut object_map = HashMap::<String, Object>::new();

    for orbit in orbits {
        assert_eq!(
            2,
            orbit.len(),
            "Wrong number of objects {} in orbit relation",
            orbit.len()
        );

        let orbitee = &orbit[0];
        let orbiter = &orbit[1];

        object_map
            .entry(orbitee.clone())
            .and_modify(|x| x.orbited_by.push(orbiter.clone()))
            .or_insert_with(|| Object::new(None, vec![orbiter.clone()]));
        object_map
            .entry(orbiter.clone())
            .and_modify(|x| x.orbits = Some(orbitee.clone()))
            .or_insert_with(|| Object::new(Some(orbitee.clone()), vec![]));
    }

    object_map
}

fn count_orbits(object_map: &HashMap<String, Object>) -> (usize, usize) {
    let direct_orbits = object_map.values().filter(|x| x.orbits.is_some()).count();
    let mut total_orbits = 0;
    for object in object_map.values() {
        let mut current_object = object;
        while current_object.orbits.is_some() {
            total_orbits += 1;
            current_object = object_map
                .get(&current_object.orbits.clone().unwrap())
                .unwrap();
        }
    }

    let indirect_orbits = total_orbits - direct_orbits;

    (direct_orbits, indirect_orbits)
}

// fn num_transfers_older(object_map: &HashMap<String, Object>) -> i32 {
//     let origin = object_map.get("YOU").unwrap();
//     let mut explored = HashSet::<String>::new();
//     explored.insert("YOU".to_string());
//     let mut active: Vec<String> = origin.orbited_by.clone();
//     active.push(origin.orbits.clone().unwrap());
//     let target = "SAN";
//     let mut transfer_counter = 0;
//     'outer: loop {
//         //Forever - we're gonna find it!
//         //First check the active set for the target. If we find it we're done.
//         for object_name in &active {
//             if object_name == target {
//                 break 'outer;
//             }
//             //Now add the active set to the explored set.
//             explored.insert(object_name.clone());
//         }

//         let mut new_active: Vec<String> = vec![];

//         for object_name in &active {
//             let object = object_map.get(object_name.as_str()).unwrap();
//             for orbiter in &object.orbited_by {
//                 if !explored.contains(orbiter.as_str()) {
//                     new_active.push(orbiter.to_string());
//                 }
//             }

//             let orbits = object.orbits.clone();
//             if let Some(orbitee) = orbits {
//                 if !explored.contains(orbitee.as_str()) {
//                     new_active.push(orbitee)
//                 }
//             }
//         }

//         active = new_active;
//         transfer_counter += 1;
//     }

//     transfer_counter - 1 //orbital transfers are not inclusive of start and end so the above algorithm will overcount by 1
// }

// fn num_transfers_old(object_map: &HashMap<String, Object>) -> Result<i32, &'static str> {
//     let origin = object_map.get("YOU").ok_or("Origin not found")?;
//     let mut explored = HashSet::new();
//     explored.insert("YOU".to_string());
//     let mut active: Vec<String> = origin.orbited_by.iter().cloned().collect();
//     if let Some(orbit) = &origin.orbits {
//         active.push(orbit.clone());
//     }
//     let target = "SAN";
//     let mut transfer_counter = 0;

//     while !active.contains(&target.to_string()) {
//         let mut new_active = Vec::new();

//         for object_name in &active {
//             if let Some(object) = object_map.get(object_name) {
//                 new_active.extend(
//                     object
//                         .orbited_by
//                         .iter()
//                         .filter(|orbiter| !explored.contains(*orbiter))
//                         .cloned(),
//                 );

//                 if let Some(orbitee) = &object.orbits {
//                     if !explored.contains(orbitee) {
//                         new_active.push(orbitee.clone());
//                     }
//                 }
//             }
//             explored.insert(object_name.clone());
//         }

//         if new_active.is_empty() {
//             return Err("Path to target not found");
//         }

//         active = new_active;
//         transfer_counter += 1;
//     }

//     Ok(transfer_counter - 1) // Adjust for inclusive start and end
// }

fn num_transfers(
    object_map: &HashMap<String, Object>,
) -> Result<i32, &'static str> {
    let origin = object_map.get("YOU").ok_or("Origin not found")?;
    let mut explored = HashSet::new();
    explored.insert("YOU");

    let mut active = HashSet::new();
    active.extend(origin.orbited_by.iter().map(String::as_str));
    if let Some(orbit) = &origin.orbits {
        active.insert(orbit);
    }

    let target = "SAN";
    let mut transfer_counter = 0;

    while !active.contains(target) {
        let mut new_active = HashSet::new();

        for &object_name in active.iter() {
            if let Some(object) = object_map.get(object_name) {
                new_active.extend(
                    object
                        .orbited_by
                        .iter()
                        .map(String::as_str)
                        .filter(|&orbiter| !explored.contains(orbiter)),
                );

                if let Some(orbitee) = &object.orbits {
                    if !explored.contains(orbitee.as_str()) {
                        new_active.insert(orbitee.as_str());
                    }
                }
            }
            explored.insert(object_name);
        }

        if new_active.is_empty() {
            return Err("Path to target not found");
        }

        active = new_active;
        transfer_counter += 1;
    }

    Ok(transfer_counter - 1) // Adjust for inclusive start and end
}

#[cfg(test)]
#[test]
fn test_count_orbits() {
    let orbit_string: Vec<String> = "COM)B
B)C
C)D
D)E
E)F
B)G
G)H
D)I
E)J
J)K
K)L"
    .lines()
    .map(|x| x.to_string())
    .collect();
    let orbits = parse_orbits(&orbit_string);
    let orbit_map = build_orbit_map(&orbits);
    let (direct_orbits, indirect_orbits) = count_orbits(&orbit_map);
    assert_eq!(11, direct_orbits);
    assert_eq!(31, indirect_orbits);
}

#[test]
fn test_orbital_transfers() {
    let orbit_string: Vec<String> = "COM)B
B)C
C)D
D)E
E)F
B)G
G)H
D)I
E)J
J)K
K)L
K)YOU
I)SAN"
        .lines()
        .map(|x| x.to_string())
        .collect();
    let orbits = parse_orbits(&orbit_string);
    let orbit_map = build_orbit_map(&orbits);
    assert_eq!(Ok(4), num_transfers(&orbit_map));
}
