use crate::utils;
use std::collections::HashSet;

#[derive(Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug)]
struct PathDescriptor {
    direction: Direction,
    length: usize,
}

impl PathDescriptor {
    fn new(direction: char, length: usize) -> PathDescriptor {
        PathDescriptor {
            direction: match direction {
                'U' => Direction::Up,
                'D' => Direction::Down,
                'L' => Direction::Left,
                'R' => Direction::Right,
                _ => panic!("Wrong direction!"),
            },
            length,
        }
    }
}

#[derive(Eq, Hash, PartialEq, PartialOrd, Debug)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn new(x: i32, y: i32) -> Point {
        Point { x, y }
    }

    fn manhattan_distance_from_origin(&self) -> usize {
        self.x.unsigned_abs() as usize + self.y.unsigned_abs() as usize
    }
}

#[derive(PartialEq, Debug)]
enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Debug)]
struct LineSegment {
    orientation: Orientation,
    direction: Direction,
    min: i32, //minimum x (horizontal) or y (vertical) value
    max: i32, //maximum x (horizontal) or y (vertical) value
    c: i32,   //constant x (vertical) or y (horizontal) value
}

impl LineSegment {
    fn new(
        orientation: Orientation,
        direction: Direction,
        min: i32,
        max: i32,
        c: i32,
    ) -> LineSegment {
        LineSegment {
            orientation,
            direction,
            min,
            max,
            c,
        }
    }

    fn intersects(&self, other: &LineSegment) -> bool {
        other.orientation != self.orientation
            && other.min <= self.c
            && self.c <= other.max
            && self.min <= other.c
            && other.c <= self.max
    }
}

///Day 3 solution
pub fn day3() -> (usize, usize) {
    let wires: Vec<String> = utils::parse_input("input/day3.txt");

    let wire1 = parse_wire(&wires[0]);
    let wire2 = parse_wire(&wires[1]);
    println!("lengths: 1: {}, 2: {}", wire1.len(), wire2.len());

    // Going to do two different algorithms here:
    // 1. Iterate through the path descriptions in wire 1 and map to line segments
    // with start and end values and constant x or y, horizontal or vertical.
    // Do the same for wire 2. Iterate through both lists to find line segments that intersect
    // i.e. the wire 1 segment is perpendicular to the wire 2 segment and the constant value of each is
    // between the max and min of the other - intersection has the (x, y) value of the constants.
    // For each intersection found, update the minimum Manhattan distance (excepting 0) yielding the answer
    // when the lists are exhausted. This is O(M*N) where M is the number of path segments in wire 1 and N
    // the number of segments in wire 2. For the input M = N = 301, so this is about 90,600.
    //
    // 2. Trace wire 1 - for each path segment put each point traversed in a HashSet. Then trace wire 2. For
    // each point traversed test if it's in the set. If it is, there's an intersection. Store the min
    // Manhattan distance seen so far. This is O(P + Q) where P is the total number of points traversed in wire 1 and Q
    // is the total number of points traversed in wire 2. P and Q are both about 301 * 500 (on average) steps = 31,000
    //
    // Numbers suggest 2 will be slightly faster, but require more memory (O(P+Q) ~ 30,000, vs O(M + N) ~ 600).
    // Let's see ...
    //
    // After testing, 1. takes 0.9ms, 2 takes 130ms! So 1 is much faster.
    // For part 2, only going to use method 1.

    let (part1, part2) = min_manhattan_distance_of_intersections(&wire1, &wire2);

    #[allow(unused)]
    let part1_alt: usize = min_manhattan_distance_of_intersections_alt(&wire1, &wire2);

    (part1, part2)
}

fn parse_wire(wire_description: &str) -> Vec<PathDescriptor> {
    wire_description
        .split(',')
        .map(|x| {
            PathDescriptor::new(
                x.as_bytes()[0] as char, //Know this is just ASCII so can assume indexing will work
                std::str::from_utf8(&x.as_bytes()[1..])
                    .unwrap()
                    .parse()
                    .unwrap(),
            )
        })
        .collect::<Vec<PathDescriptor>>()
}

fn min_manhattan_distance_of_intersections(
    wire1: &[PathDescriptor],
    wire2: &[PathDescriptor],
) -> (usize, usize) {
    let mut min_manhattan_distance_so_far = 0;
    let mut min_steps_so_far = 0;
    let mut intersection_found = false;
    let mut wire1_steps_so_far = 0;
    let mut wire2_steps_so_far = 0;
    let wire1_segments: Vec<LineSegment> = line_segments(wire1);
    let wire2_segments: Vec<LineSegment> = line_segments(wire2);

    // O(N*M)
    for segment1 in wire1_segments {
        for segment2 in &wire2_segments {
            if segment1.intersects(segment2) {
                let intersection_point = match segment1.orientation {
                    Orientation::Horizontal => Point::new(segment2.c, segment1.c), //Segment 1 is horizontal so segment1.c is the y coordinate of the intersection
                    Orientation::Vertical => Point::new(segment1.c, segment2.c), // and vice-versa.
                };

                // Ignore (0, 0)
                if intersection_point.x != 0 || intersection_point.y != 0 {
                    update_min_manhattan_distance(
                        &intersection_point,
                        intersection_found,
                        &mut min_manhattan_distance_so_far,
                    );

                    let mut total_steps = wire1_steps_so_far + wire2_steps_so_far;

                    // Add steps from start of segment1 to point of intersection
                    total_steps += match segment1.direction {
                        Direction::Up | Direction::Right => segment2.c - segment1.min,
                        Direction::Down | Direction::Left => segment1.max - segment2.c,
                    };

                    // Add steps from start of segment2 to point of intersection
                    total_steps += match segment2.direction {
                        Direction::Up | Direction::Right => segment1.c - segment2.min,
                        Direction::Down | Direction::Left => segment2.max - segment1.c,
                    };

                    if !intersection_found || total_steps < min_steps_so_far {
                        min_steps_so_far = total_steps;
                    }

                    intersection_found = true;
                }
            }
            wire2_steps_so_far += segment2.max - segment2.min;
        }
        wire2_steps_so_far = 0;
        wire1_steps_so_far += segment1.max - segment1.min;
    }

    (min_manhattan_distance_so_far, min_steps_so_far as usize)
}

fn line_segments(wire: &[PathDescriptor]) -> Vec<LineSegment> {
    let mut x: i32 = 0;
    let mut y: i32 = 0;

    wire.iter()
        .map(|pd| match pd.direction {
            Direction::Up => {
                // Increment first so that last expression in branch is the line segment - this makes
                // calculating
                y += pd.length as i32;
                LineSegment::new(
                    Orientation::Vertical,
                    Direction::Up,
                    y - pd.length as i32,
                    y,
                    x,
                )
            }
            Direction::Down => {
                y -= pd.length as i32;
                LineSegment::new(
                    Orientation::Vertical,
                    Direction::Down,
                    y,
                    y + pd.length as i32,
                    x,
                )
            }
            Direction::Right => {
                x += pd.length as i32;
                LineSegment::new(
                    Orientation::Horizontal,
                    Direction::Right,
                    x - pd.length as i32,
                    x,
                    y,
                )
            }
            Direction::Left => {
                x -= pd.length as i32;
                LineSegment::new(
                    Orientation::Horizontal,
                    Direction::Left,
                    x,
                    x + pd.length as i32,
                    y,
                )
            }
        })
        .collect()
}

fn min_manhattan_distance_of_intersections_alt(
    wire1: &[PathDescriptor],
    wire2: &[PathDescriptor],
) -> usize {
    let mut wire1_traversed_points: HashSet<Point> = HashSet::new();

    let mut x: i32 = 0;
    let mut y: i32 = 0;
    let mut min_manhattan_distance_so_far: usize = 0;
    let mut intersection_found = false;

    for descriptor in wire1 {
        match descriptor.direction {
            Direction::Up => {
                y += descriptor.length as i32;
                for i in y - descriptor.length as i32..y {
                    wire1_traversed_points.insert(Point::new(x, i));
                }
            }
            Direction::Down => {
                y -= descriptor.length as i32;
                for i in y..y + descriptor.length as i32 {
                    wire1_traversed_points.insert(Point::new(x, i));
                }
            }
            Direction::Right => {
                x += descriptor.length as i32;
                for i in x - descriptor.length as i32..x {
                    wire1_traversed_points.insert(Point::new(i, y));
                }
            }
            Direction::Left => {
                x -= descriptor.length as i32;
                for i in x..x + descriptor.length as i32 {
                    wire1_traversed_points.insert(Point::new(i, y));
                }
            }
        }
    }

    x = 0;
    y = 0;
    for descriptor in wire2 {
        match descriptor.direction {
            Direction::Up => {
                y += descriptor.length as i32;
                for i in y - descriptor.length as i32..y {
                    let point = Point::new(x, i);
                    if wire1_traversed_points.contains(&point) && (point.x != 0 || point.y != 0) {
                        update_min_manhattan_distance(
                            &point,
                            intersection_found,
                            &mut min_manhattan_distance_so_far,
                        );
                        intersection_found = true;
                    }
                }
            }
            Direction::Down => {
                y -= descriptor.length as i32;
                for i in y..y + descriptor.length as i32 {
                    let point = Point::new(x, i);
                    if wire1_traversed_points.contains(&point) && (point.x != 0 || point.y != 0) {
                        update_min_manhattan_distance(
                            &point,
                            intersection_found,
                            &mut min_manhattan_distance_so_far,
                        );
                        intersection_found = true;
                    }
                }
            }
            Direction::Right => {
                x += descriptor.length as i32;
                for i in x - descriptor.length as i32..x {
                    let point = Point::new(i, y);
                    if wire1_traversed_points.contains(&point) && (point.x != 0 || point.y != 0) {
                        update_min_manhattan_distance(
                            &point,
                            intersection_found,
                            &mut min_manhattan_distance_so_far,
                        );
                        intersection_found = true;
                    }
                }
            }
            Direction::Left => {
                x -= descriptor.length as i32;
                for i in x..x + descriptor.length as i32 {
                    let point = Point::new(i, y);
                    if wire1_traversed_points.contains(&point) && (point.x != 0 || point.y != 0) {
                        update_min_manhattan_distance(
                            &point,
                            intersection_found,
                            &mut min_manhattan_distance_so_far,
                        );
                        intersection_found = true;
                    }
                }
            }
        }
    }

    min_manhattan_distance_so_far
}

fn update_min_manhattan_distance(
    point: &Point,
    intersection_found: bool,
    min_manhattan_distance_so_far: &mut usize,
) {
    let intersection_manhattan_distance = point.manhattan_distance_from_origin();
    if !intersection_found || intersection_manhattan_distance < *min_manhattan_distance_so_far {
        *min_manhattan_distance_so_far = intersection_manhattan_distance;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_min_manhattan_distance() {
        let mut wire1 = parse_wire("R8,U5,L5,D3");
        let mut wire2 = parse_wire("U7,R6,D4,L4");
        assert_eq!(
            (6, 30),
            min_manhattan_distance_of_intersections(&wire1, &wire2)
        );
        assert_eq!(
            6,
            min_manhattan_distance_of_intersections_alt(&wire1, &wire2)
        );
        wire1 = parse_wire("R75,D30,R83,U83,L12,D49,R71,U7,L72");
        wire2 = parse_wire("U62,R66,U55,R34,D71,R55,D58,R83");
        assert_eq!(
            (159, 610),
            min_manhattan_distance_of_intersections(&wire1, &wire2)
        );
        assert_eq!(
            159,
            min_manhattan_distance_of_intersections_alt(&wire1, &wire2)
        );
        wire1 = parse_wire("R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51");
        wire2 = parse_wire("U98,R91,D20,R16,D67,R40,U7,R15,U6,R7");
        assert_eq!(
            (135, 410),
            min_manhattan_distance_of_intersections(&wire1, &wire2)
        );
        assert_eq!(
            135,
            min_manhattan_distance_of_intersections_alt(&wire1, &wire2)
        );
    }
}
