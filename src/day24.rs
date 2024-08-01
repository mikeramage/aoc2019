use std::{
    collections::{HashMap, HashSet},
    fs,
};

const NUM_ROWS: usize = 5;
const NUM_COLS: usize = 5;
const INNER_INDICES: [usize; 4] = [7, 11, 13, 17]; //when eris is flattened, the 4 inner indices affected by bugs on the level above
const OUTER_INDICES: [usize; 16] = [0, 1, 2, 3, 4, 5, 9, 10, 14, 15, 19, 20, 21, 22, 23, 24]; // the 16 outer indices affected by bugs on the level below.

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Tile {
    Empty,
    Bug,
}

impl TryFrom<char> for Tile {
    type Error = String;
    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '.' => Ok(Tile::Empty),
            '#' => Ok(Tile::Bug),
            other => Err(format!("Invalid character for Tile : {}", other).to_string()),
        }
    }
}

impl From<Tile> for char {
    fn from(tile: Tile) -> char {
        match tile {
            Tile::Empty => '.',
            Tile::Bug => '#',
        }
    }
}

pub fn day24() -> (usize, usize) {
    let input = fs::read_to_string("input/day24.txt").expect("Could not open file");
    let mut eris = parse_input(&input);
    let mut hyper_eris: HashMap<isize, Vec<Vec<Tile>>> = HashMap::new(); //Map of depth to eris

    let mut seen: HashSet<Vec<Vec<Tile>>> = HashSet::new();

    while !seen.contains(&eris) {
        seen.insert(eris.clone());
        eris = evolve_one_minute(&eris);
    }

    let part1 = biodiversity_rating(&eris);

    eris = parse_input(&input);
    hyper_eris.insert(0, eris);

    for _ in 0..200 {
        hyper_eris = evolve_one_minute_recursive(&hyper_eris);
    }

    let part2: usize = count_bugs(&hyper_eris);

    (part1, part2)
}

fn count_bugs(hyper_eris: &HashMap<isize, Vec<Vec<Tile>>>) -> usize {
    hyper_eris
        .iter()
        .map(|(_, eris)| {
            eris.iter()
                .flatten()
                .filter(|tile| **tile == Tile::Bug)
                .count()
        })
        .sum()
}

fn biodiversity_rating(eris: &[Vec<Tile>]) -> usize {
    eris.iter()
        .flatten()
        .enumerate()
        .filter(|(_, tile)| **tile == Tile::Bug)
        .map(|(index, _)| 2_usize.pow(index as u32))
        .sum()
}

fn parse_input(input: &str) -> Vec<Vec<Tile>> {
    input
        .lines()
        .map(|line| {
            line.chars()
                .map(|c| {
                    Tile::try_from(c).unwrap_or_else(|err| {
                        panic!("Could not extract Tile from character: {}", err)
                    })
                })
                .collect::<Vec<Tile>>()
        })
        .collect::<Vec<Vec<Tile>>>()
}

//Pedestrian because I'm tired - this code totally sucks
fn evolve_one_minute_recursive(
    hyper_eris: &HashMap<isize, Vec<Vec<Tile>>>,
) -> HashMap<isize, Vec<Vec<Tile>>> {
    let mut new_hyper_eris: HashMap<isize, Vec<Vec<Tile>>> = HashMap::new();

    let mut new_levels: Vec<isize> = vec![];
    for (depth, eris) in hyper_eris {
        //First of all, do we need to create any new levels as a result of processing this existing level?
        // We need to create a new level (to be populated later) if the following conditions hold:
        // - We're at the max or min level of eris
        // - There is no existing level below and there is more than one bug on the outer rim of this level (1st or last rows and colums)
        // - There is no existing level above and there is more than one bug on the inner rim of this level (middle 3x3 square excluding the
        //   center at 2,2).
        // We'll just note this just now, and create the new levels later - the existing tiles don't depend on these
        let mut new_eris: Vec<Vec<Tile>> = vec![];
        if *depth == *hyper_eris.keys().min().unwrap() && bug_on_rim(eris, &OUTER_INDICES) {
            new_levels.push(*depth - 1);
        }

        if *depth == *hyper_eris.keys().max().unwrap() && bug_on_rim(eris, &INNER_INDICES) {
            new_levels.push(*depth + 1);
        }

        for (row, tiles) in eris.iter().enumerate() {
            let mut row_vector = vec![];
            for (col, _) in tiles.iter().enumerate() {
                let num_adjacent_bugs = number_adjacent_bugs(hyper_eris, eris, row, col, *depth);
                match eris[row][col] {
                    Tile::Empty => {
                        if (num_adjacent_bugs == 1 || num_adjacent_bugs == 2)
                            && (row != 2 || col != 2)
                        {
                            row_vector.push(Tile::Bug);
                        } else {
                            row_vector.push(Tile::Empty);
                        }
                    }
                    Tile::Bug => {
                        if num_adjacent_bugs == 1 {
                            row_vector.push(Tile::Bug);
                        } else {
                            row_vector.push(Tile::Empty);
                        }
                    }
                }
            }
            new_eris.push(row_vector);
        }

        new_hyper_eris.insert(*depth, new_eris);
    }

    for level in new_levels {
        if level < *hyper_eris.keys().min().unwrap() {
            //New outer level
            add_new_level(hyper_eris, &mut new_hyper_eris, level, &INNER_INDICES);
        } else {
            assert!(level > *hyper_eris.keys().max().unwrap());
            //New inner level
            add_new_level(hyper_eris, &mut new_hyper_eris, level, &OUTER_INDICES);
        }
    }

    new_hyper_eris
}

fn add_new_level(
    hyper_eris: &HashMap<isize, Vec<Vec<Tile>>>,
    new_hyper_eris: &mut HashMap<isize, Vec<Vec<Tile>>>,
    level: isize,
    indices: &[usize],
) {
    let mut new_eris: Vec<Vec<Tile>> = vec![];
    //Initialize to empty
    for _ in 0..NUM_ROWS {
        let mut row_vector = vec![];
        for _ in 0..NUM_COLS {
            row_vector.push(Tile::Empty);
        }
        new_eris.push(row_vector);
    }

    let reference_eris = new_eris.clone();

    for index in indices {
        let row = *index / NUM_ROWS; //Floor division gives us the row
        let col = *index % NUM_COLS; //Modulo gives the column
        let num_adjacent_bugs = number_adjacent_bugs(hyper_eris, &reference_eris, row, col, level);
        if num_adjacent_bugs == 1 || num_adjacent_bugs == 2 {
            new_eris[row][col] = Tile::Bug;
        } else {
            new_eris[row][col] = Tile::Empty;
        }
    }
    new_hyper_eris.insert(level, new_eris);
}

fn number_adjacent_bugs(
    hyper_eris: &HashMap<isize, Vec<Vec<Tile>>>,
    reference_eris: &[Vec<Tile>],
    row: usize,
    col: usize,
    depth: isize,
) -> usize {
    let mut adjacents = vec![];
    let d = [(1, 0), (0, 1), (-1, 0), (0, -1)];
    for (dr, dc) in d {
        if (row as isize) + dr < 0
            || (col as isize) + dc < 0
            || (row as isize) + dr >= NUM_ROWS as isize
            || (col as isize) + dc >= NUM_COLS as isize
        {
            //Look to the outer level if it exists
            if let Some(eris) = hyper_eris.get(&(depth - 1)) {
                if (row as isize) + dr < 0 {
                    // Looking to the tile above the top row, which is the 1, 2 element of the outer layer
                    adjacents.push(eris[1][2]);
                } else if (row as isize) + dr >= NUM_ROWS as isize {
                    //Looking below
                    adjacents.push(eris[3][2]);
                } else if (col as isize) + dc < 0 {
                    //Looking left
                    adjacents.push(eris[2][1]);
                } else {
                    //Looking right
                    assert!((col as isize) + dc >= NUM_COLS as isize);
                    adjacents.push(eris[2][3]);
                }
            }
        } else if (row as isize) + dr == 2 && (col as isize) + dc == 2 {
            //Move to the inner level if it exists
            if let Some(eris) = hyper_eris.get(&(depth + 1)) {
                if row == 2 && col == 1 {
                    //Right side of this tile, so left side of inner eris
                    for r in eris.iter().take(NUM_ROWS) {
                        adjacents.push(r[0]);
                    }
                } else if row == 1 && col == 2 {
                    //Bottom of this tile, top side of inner eris
                    for j in 0..NUM_COLS {
                        adjacents.push(eris[0][j]);
                    }
                } else if row == 2 && col == 3 {
                    //Left side of this tile, right side of inner eris
                    for r in eris.iter().take(NUM_ROWS) {
                        adjacents.push(r[4]);
                    }
                } else {
                    assert_eq!(3, row);
                    assert_eq!(2, col);
                    //Top side of this tile, bottom side of inner eris
                    for j in 0..NUM_COLS {
                        adjacents.push(eris[4][j]);
                    }
                }
            }
        } else {
            //Tile at the same depth
            adjacents.push(
                reference_eris[((row as isize) + dr) as usize][((col as isize) + dc) as usize],
            );
        }
    }

    adjacents.iter().filter(|tile| **tile == Tile::Bug).count()
}

fn bug_on_rim(eris: &[Vec<Tile>], indices: &[usize]) -> bool {
    eris.iter()
        .flatten()
        .enumerate()
        .any(|(index, tile)| indices.contains(&index) && *tile == Tile::Bug)
}

fn evolve_one_minute(eris: &[Vec<Tile>]) -> Vec<Vec<Tile>> {
    eris.iter()
        .enumerate()
        .map(|(row, tiles)| {
            tiles
                .iter()
                .enumerate()
                .map(|(col, tile)| {
                    let d: [(isize, isize); 4] = [(1, 0), (0, 1), (-1, 0), (0, -1)];
                    let adjacents = d
                        .iter()
                        .map(|(dr, dc)| {
                            match eris.get(if row as isize + *dr >= 0 {
                                (row as isize + *dr) as usize
                            } else {
                                NUM_ROWS //Exceeds bounds - guaranteed to return None
                            }) {
                                Some(tiles) => match tiles.get(if col as isize + *dc >= 0 {
                                    (col as isize + *dc) as usize
                                } else {
                                    NUM_COLS //Exceeds bounds - guaranteed to return None
                                }) {
                                    Some(tile) => *tile,
                                    None => Tile::Empty,
                                },
                                None => Tile::Empty,
                            }
                        })
                        .collect::<Vec<Tile>>();
                    let num_bugs = adjacents.iter().filter(|adj| **adj == Tile::Bug).count();
                    match *tile {
                        Tile::Empty => {
                            if num_bugs == 1 || num_bugs == 2 {
                                Tile::Bug
                            } else {
                                Tile::Empty
                            }
                        }
                        Tile::Bug => {
                            if num_bugs == 1 {
                                Tile::Bug
                            } else {
                                Tile::Empty
                            }
                        }
                    }
                })
                .collect::<Vec<Tile>>()
        })
        .collect::<Vec<Vec<Tile>>>()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_evolve_one_minute() {
        let mut eris = parse_input(
            "....#
#..#.
#..##
..#..
#....",
        );
        eris = evolve_one_minute(&eris);
        let mut expected = parse_input(
            "#..#.
####.
###.#
##.##
.##..",
        );
        assert_eq!(expected, eris);

        eris = evolve_one_minute(&eris);
        expected = parse_input(
            "#####
....#
....#
...#.
#.###",
        );
        assert_eq!(expected, eris);

        eris = evolve_one_minute(&eris);
        expected = parse_input(
            "#....
####.
...##
#.##.
.##.#",
        );
        assert_eq!(expected, eris);

        eris = evolve_one_minute(&eris);
        expected = parse_input(
            "####.
....#
##..#
.....
##...",
        );
        assert_eq!(expected, eris);
    }

    #[test]
    fn test_biodiversity_rating() {
        let eris = parse_input(
            ".....
.....
.....
#....
.#...",
        );
        assert_eq!(2129920, biodiversity_rating(&eris));
    }

    #[test]
    fn test_evolve_one_minute_recursive() {
        let eris = parse_input(
            "....#
#..#.
#..##
..#..
#....",
        );
        let mut hyper_eris: HashMap<isize, Vec<Vec<Tile>>> = HashMap::new();
        hyper_eris.insert(0, eris);

        for _ in 0..10 {
            hyper_eris = evolve_one_minute_recursive(&hyper_eris);
        }

        let mut expected = parse_input(
            "..#..
.#.#.
....#
.#.#.
..#..",
        );

        assert_eq!(expected, *hyper_eris.get(&-5).unwrap());

        expected = parse_input(
            "...#.
...##
.....
...##
...#.",
        );

        assert_eq!(expected, *hyper_eris.get(&-4).unwrap());

        expected = parse_input(
            "#.#..
.#...
.....
.#...
#.#..",
        );

        assert_eq!(expected, *hyper_eris.get(&-3).unwrap());

        expected = parse_input(
            ".#.##
....#
....#
...##
.###.",
        );

        assert_eq!(expected, *hyper_eris.get(&-2).unwrap());

        expected = parse_input(
            "#..##
...##
.....
...#.
.####",
        );

        assert_eq!(expected, *hyper_eris.get(&-1).unwrap());

        expected = parse_input(
            ".#...
.#.##
.#...
.....
.....",
        );

        assert_eq!(expected, *hyper_eris.get(&0).unwrap());
        expected = parse_input(
            ".##..
#..##
....#
##.##
#####",
        );

        assert_eq!(expected, *hyper_eris.get(&1).unwrap());

        expected = parse_input(
            "###..
##.#.
#....
.#.##
#.#..",
        );

        assert_eq!(expected, *hyper_eris.get(&2).unwrap());

        expected = parse_input(
            "..###
.....
#....
#....
#...#",
        );

        assert_eq!(expected, *hyper_eris.get(&3).unwrap());

        expected = parse_input(
            ".###.
#..#.
#....
##.#.
.....",
        );

        assert_eq!(expected, *hyper_eris.get(&4).unwrap());

        expected = parse_input(
            "####.
#..#.
#..#.
####.
.....",
        );

        assert_eq!(expected, *hyper_eris.get(&5).unwrap());
        assert_eq!(11, hyper_eris.len());
        assert_eq!(99, count_bugs(&hyper_eris));
    }
}
