use std::fmt::Debug;
use std::fs;
use std::str;

/// Takes a file containing a list of strings, one per line, and returns the
/// list as a vector of T
pub fn parse_input<T: str::FromStr>(input_file: &str) -> Vec<T>
where
    <T as str::FromStr>::Err: Debug,
{
    let input = fs::read_to_string(input_file).expect("Oh dear, couldn't read file!");
    let vector: Vec<T> = input.lines().map(|line| line.parse().unwrap()).collect();
    vector
}

/// Input is a single line, with Ts separated by separator chars. Return a Vec of T.
pub fn parse_input_by_sep<T: str::FromStr>(input_file: &str, separator: char) -> Vec<T>
where
    <T as str::FromStr>::Err: Debug,
{
    let input = fs::read_to_string(input_file).expect("Oh dear, couldn't read file!");
    let vector: Vec<T> = input.split(separator).map(|x| x.parse().unwrap()).collect();
    vector
}

/// Parse input by splitting on blank lines and returning as a vector of strings
#[allow(dead_code)]
pub fn parse_input_by_blank_lines(input_file: &str) -> Vec<String> {
    let input = fs::read_to_string(input_file).expect("Oh dear, couldn't read file!");
    let vector: Vec<String> = input.split("\n\n").map(|x| x.to_string()).collect();
    vector
}

/// Parse input by creating a vector of strings, one string per line. Then for each line convert each character to
/// a digit and store as a usize. Each element of the resulting vector is a vector of usizes obtained from each character in the corresponding line.
#[allow(dead_code)]
pub fn parse_input_usizes(input_file: &str) -> Vec<Vec<usize>> {
    let input = fs::read_to_string(input_file).expect("Oh dear, couldn't read file!");
    let mut vector: Vec<String> = input.lines().map(|line| line.parse().unwrap()).collect();
    vector
        .iter_mut()
        .map(|x: &mut String| {
            x.chars()
                .map(|y| y.to_digit(10).unwrap() as usize)
                .collect()
        })
        .collect::<Vec<Vec<usize>>>()
}

/// As for parse_input_usize, but return a vec of a vec of chars.
#[allow(dead_code)]
pub fn parse_input_chars(input_file: &str) -> Vec<Vec<char>> {
    let input = fs::read_to_string(input_file).expect("Oh dear, couldn't read file!");
    let mut vector: Vec<String> = input.lines().map(|line| line.parse().unwrap()).collect();
    vector
        .iter_mut()
        .map(|x: &mut String| x.chars().collect())
        .collect::<Vec<Vec<char>>>()
}

/// Each line is a <separator>-separated list of strings, parse this into a Vec of Vec of strings - top-level vec
/// is split by newlines, inner vec split by separator character.
#[allow(dead_code)]
pub fn parse_input_sep_strings(input_file: &str, separator: char) -> Vec<Vec<String>> {
    let input = fs::read_to_string(input_file).expect("Oh dear, couldn't read file!");
    let mut vector: Vec<String> = input.lines().map(|line| line.to_string()).collect();
    vector
        .iter_mut()
        .map(|x: &mut String| x.split(separator).map(|y: &str| y.to_string()).collect())
        .collect::<Vec<Vec<String>>>()
}

/// Converts a string representing a binary number of up to 63 characters, e.g. "0100110110111" and converts to a usize.
///
/// It works like this:
///  - Converts the string to a Chars iterator and enumerates - so if you've got [(0, '0'), (1, '1'), (2, '0')...]
///  - Maps these values by left shifting the appropriate amount: ((length of the string - 1) - index) in the enumeration.
/// To see this, remember you want to left shift the last index by 0.
///  - Sums them.
#[allow(dead_code)]
pub fn string_binary_to_usize(binary_as_string: String) -> usize {
    assert!(binary_as_string.len() < 64); //Obviously this breaks if usize != 64 but I don't care.
    let binary_as_chars = binary_as_string.chars();
    let shift = binary_as_string.len() - 1;
    let number: usize = binary_as_chars
        .enumerate()
        .map(|(index, digit)| {
            if let '1' = digit {
                1 << (shift - index)
            } else {
                0
            }
        })
        .sum();
    number
}
