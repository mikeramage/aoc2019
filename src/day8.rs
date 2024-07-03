use std::fs;

///Day 8 solution
pub fn day8() -> (usize, usize) {
    let mut image = fs::read_to_string("input/day8.txt").expect("Couldn't read input file");
    println!("day8 vec: {:?}", image);

    let part1: i32 = 0;
    let part2: i32 = 0;
    (part1 as usize, part2 as usize)
}
