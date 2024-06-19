use itertools::Itertools;

///Day 4 solution
pub fn day4() -> (usize, usize) {
    let start = "137683";
    let end = "596254"; //End of range + 1 so I can write start..end, only includes up to 596253.
    let mut count: usize = 0;
    let mut count2: usize = 0;

    for digit_string in start.parse::<usize>().unwrap()..end.parse::<usize>().unwrap() {
        if is_candidate_password(&digit_string.to_string()) {
            count += 1;
        }

        if is_candidate_password_part2(&digit_string.to_string()) {
            count2 += 1;
        }
    }

    (count, count2)
}

fn is_candidate_password(digit_string: &str) -> bool {
    if digit_string.len() != 6 {
        return false;
    }

    if digit_string.chars().sorted().collect::<String>() != *digit_string {
        return false;
    }

    let mut char_iter = digit_string.chars().peekable();
    let mut repeated_digits = false;
    while let Some(c) = char_iter.next() {
        if let Some(next) = char_iter.peek() {
            if c == *next {
                repeated_digits = true;
                break;
            }
        }
    }

    if !repeated_digits {
        return false;
    }

    true
}

fn is_candidate_password_part2(digit_string: &str) -> bool {
    if digit_string.len() != 6 {
        return false;
    }

    if digit_string.chars().sorted().collect::<String>() != *digit_string {
        return false;
    }

    let mut char_iter = digit_string.chars().peekable();
    let mut consecutive_matching_digits = 0; //number of times we've encountered this character in a row
    let mut exact_double_found = false;
    while let Some(c) = char_iter.next() {
        if let Some(next) = char_iter.peek() {
            if c == *next {
                // Digit matches
                consecutive_matching_digits += 1;
            } else {
                // Digit doesn't match, but if we've found 2 we're good
                if consecutive_matching_digits == 1 {
                    exact_double_found = true;
                }
                //reset the number of matching digits.
                consecutive_matching_digits = 0;
            }
        } else if consecutive_matching_digits == 1 {
            //End of the list, but we might just have found the final perfect double
            exact_double_found = true;
            consecutive_matching_digits = 0;
        }
    }

    if !exact_double_found {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_candidate_passwords() {
        assert!(is_candidate_password("111111"));
        assert!(!is_candidate_password("223450"));
        assert!(!is_candidate_password("123789"));
        assert!(is_candidate_password_part2("112233"));
        assert!(!is_candidate_password_part2("123444"));
        assert!(is_candidate_password_part2("111122"));
    }
}
