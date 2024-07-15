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
        .fold(signal.clone(), |acc, _| do_one_phase(&acc, &BASE))
        .iter()
        .map(|d| d.to_string())
        .take(8)
        .collect::<String>()
        .parse::<usize>()
        .unwrap();

    //For part 2, my starting index is 5,974,057. The matrix of patterns is lower-triangular (all zeros in
    // the bottom left, with num zeros for row i = i (assuming 0-indexing)). By row 5,974,057 all the non-zero
    //entries from 5,974,057 to 6,500,000 are 1. As a result applying one phase to the digits from indices
    //5,974,057 to 6,500,000 simply sums all those digits. We just do this 100 times. Naively
    //that's 500,001*500,000/2 * 100 operations = trillions of operations. But non-naively, once we do it for
    //5,974,057 we can get the values for each of the following digits by subtracting off the previous digit.
    //So to calculate output digit with index 5,974,058, we take the previouly calculated value for 5,974,057
    //and subtract the input value at index 5,974,057. That's only
    //1,000,000 operations per phase. 100,000,000 should be more than manageable so let's not oversimplify further.
    let mut new_signal = signal.repeat(10_000);
    let initial_index = signal[0..7]
        .iter()
        .map(|d| d.to_string())
        .collect::<String>()
        .parse::<usize>()
        .unwrap();

    println!("Initial index:{}", initial_index);

    //100 phases - let's do some old skool for loops as I don't currently have the energy to use functional style
    //Note - it would have been neater to build up from the bottom, while my algorithm of adding them all up then subtracting
    //one at a time is all a wee bit sad and twice as much work as it should have been.
    for _i in 0..100 {
        let mut running_value = 0; //initially built up as the new value of the initial index, but
                                   //reduced by the value of the kth index for each subsequent index ot give the new kth index

        #[allow(clippy::needless_range_loop)]
        for j in (initial_index)..new_signal.len() {
            running_value += new_signal[j];
        }

        let mut old_value = new_signal[initial_index];
        new_signal[initial_index] = running_value.abs() % 10;

        #[allow(clippy::needless_range_loop)]
        for k in (initial_index + 1)..new_signal.len() {
            //Value is the value at the initial index minus the current value of the kth element
            running_value -= old_value;
            old_value = new_signal[k];
            new_signal[k] = running_value.abs() % 10;
        }
    }

    //Think that's it. Now just output values initial_index to initial_index + 8 and return
    let part2 = new_signal[initial_index..(initial_index + 8)]
        .iter()
        .map(|d| d.to_string())
        .collect::<String>()
        .parse::<usize>()
        .unwrap();

    (part1, part2)
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
