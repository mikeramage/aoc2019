use std::fs;

///Day 8 solution
pub fn day8() -> (usize, usize) {
    let image = fs::read_to_string("input/day8.txt").expect("Couldn't read input file");
    let image_len = image.len();
    const WIDTH: usize = 25;
    const HEIGHT: usize = 6;
    assert!(image_len % (WIDTH * HEIGHT) == 0);
    let layer_size = WIDTH * HEIGHT;

    let part1 = ones_times_twos_for_fewest_zeros(&image, layer_size);

    let part2: i32 = 0;

    let decoded_image = decode_image(&image, layer_size);

    for i in 0..HEIGHT {
        for j in 0..WIDTH {
            if decoded_image[i * WIDTH + j] == '0' {
                print!(" ");
            } else {
                print!("*");
            }
        }
        println!();
    }

    (part1, part2 as usize)
}

fn ones_times_twos_for_fewest_zeros(image: &str, layer_size: usize) -> usize {
    let layers: Vec<&str> = image
        .as_bytes()
        .chunks(layer_size)
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap();

    let (num_ones, num_twos) = layers
        .iter()
        .map(|layer| (layer.chars().filter(|&c| c == '0').count(), layer))
        .min_by_key(|&(zero_count, _)| zero_count)
        .map(|(_, layer)| {
            (
                layer.chars().filter(|&c| c == '1').count(),
                layer.chars().filter(|&c| c == '2').count(),
            )
        })
        .unwrap();

    num_ones * num_twos
}

fn decode_image(image: &str, layer_size: usize) -> Vec<char> {
    let layers: Vec<&str> = image
        .as_bytes()
        .chunks(layer_size)
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap();

    let mut decoded_image: Vec<char> = Vec::new();

    for i in 0..layer_size {
        for layer in &layers {
            let char_at_i = layer.chars().nth(i).unwrap();
            if char_at_i != '2' {
                decoded_image.push(char_at_i);
                break;
            }
        }
    }

    decoded_image
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ones_times_twos() {
        assert_eq!(1, ones_times_twos_for_fewest_zeros("123456789012", 6));
    }

    #[test]
    fn test_decode_image() {
        assert_eq!(
            "0110",
            decode_image("0222112222120000", 4)
                .iter()
                .collect::<String>()
        );
    }
}
