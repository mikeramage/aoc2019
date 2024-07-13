use std::fs;
use std::iter;

static BASE: [isize; 4] = [0, 1, 0, -1];

pub fn day16() -> (usize, usize) {
    let signal: Vec<isize> = fs::read_to_string("input/day16.txt")
        .expect("Bad input")
        .trim()
        .chars()
        .map(|c| c.to_digit(10).unwrap() as isize)
        .collect::<Vec<isize>>();

    let part1: usize = (0..100)
        .fold(signal, |acc, _| do_one_phase(&acc, &BASE))
        .iter()
        .map(|d| d.to_string())
        .take(8)
        .collect::<String>()
        .parse::<usize>()
        .unwrap();

    (part1, 0)
}

fn do_one_phase(input: &[isize], base: &[isize]) -> Vec<isize> {
    input
        .iter()
        .enumerate()
        .map(|(i, _)| {
            base.iter()
                .flat_map(|d| iter::repeat(*d).take(i + 1))
                .cycle()
                .skip(1)
                .zip(input.iter())
                .map(|(input_elem, base_elem)| input_elem * *base_elem)
                .sum::<isize>()
                .abs()
                % 10
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_do_one_phase() {
        let mut input = "12345678"
            .chars()
            .map(|c| c.to_digit(10).unwrap() as isize)
            .collect::<Vec<isize>>();
        let mut output = do_one_phase(&input, &BASE);
        assert_eq!(
            output,
            "48226158"
                .chars()
                .map(|c| c.to_digit(10).unwrap() as isize)
                .collect::<Vec<isize>>()
        );
        input = output;
        output = do_one_phase(&input, &BASE);
        assert_eq!(
            output,
            "34040438"
                .chars()
                .map(|c| c.to_digit(10).unwrap() as isize)
                .collect::<Vec<isize>>()
        );
        input = output;
        output = do_one_phase(&input, &BASE);
        assert_eq!(
            output,
            "03415518"
                .chars()
                .map(|c| c.to_digit(10).unwrap() as isize)
                .collect::<Vec<isize>>()
        );
        input = output;
        output = do_one_phase(&input, &BASE);
        assert_eq!(
            output,
            "01029498"
                .chars()
                .map(|c| c.to_digit(10).unwrap() as isize)
                .collect::<Vec<isize>>()
        );
    }

    #[test]
    fn test_do_100_phases() {
        let mut input = "80871224585914546619083218645595"
            .chars()
            .map(|c| c.to_digit(10).unwrap() as isize)
            .collect::<Vec<isize>>();
        assert_eq!(
            "24176176"
                .chars()
                .map(|c| c.to_digit(10).unwrap() as isize)
                .collect::<Vec<isize>>(),
            (0..100)
                .fold(input, |acc, _| do_one_phase(&acc, &BASE))
                .iter()
                .map(|d| *d)
                .take(8)
                .collect::<Vec<isize>>()
        );

        input = "19617804207202209144916044189917"
            .chars()
            .map(|c| c.to_digit(10).unwrap() as isize)
            .collect::<Vec<isize>>();
        assert_eq!(
            "73745418"
                .chars()
                .map(|c| c.to_digit(10).unwrap() as isize)
                .collect::<Vec<isize>>(),
            (0..100)
                .fold(input, |acc, _| do_one_phase(&acc, &BASE))
                .iter()
                .map(|d| *d)
                .take(8)
                .collect::<Vec<isize>>()
        );

        input = "69317163492948606335995924319873"
            .chars()
            .map(|c| c.to_digit(10).unwrap() as isize)
            .collect::<Vec<isize>>();
        assert_eq!(
            "52432133"
                .chars()
                .map(|c| c.to_digit(10).unwrap() as isize)
                .collect::<Vec<isize>>(),
            (0..100)
                .fold(input, |acc, _| do_one_phase(&acc, &BASE))
                .iter()
                .map(|d| *d)
                .take(8)
                .collect::<Vec<isize>>()
        );
    }
}
