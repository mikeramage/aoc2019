use std::fs;

///Day 8 solution
pub fn day8() -> (usize, usize) {
    let image = fs::read_to_string("input/day8.txt").expect("Couldn't read input file");
    let image_len = image.len();
    const WIDTH: usize = 25;
    const HEIGHT: usize = 6;
    assert!(image_len % (WIDTH * HEIGHT) == 0);
    let layer_size = WIDTH * HEIGHT;
    let layers = to_layers(&image, layer_size);

    let part1 = ones_times_twos_for_fewest_zeros(&layers);

    let part2: i32 = 0;

    decode_image(&layers, layer_size)
        .chunks(WIDTH)
        .for_each(|row| {
            row.iter()
                .for_each(|&c| print!("{}", if c == '0' { ' ' } else { '*' }));
            println!();
        });

    (part1, part2 as usize)
}

fn ones_times_twos_for_fewest_zeros(layers: &[&str]) -> usize {
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

fn decode_image(layers: &[&str], layer_size: usize) -> Vec<char> {
    (0..layer_size)
        .map(|i| {
            layers
                .iter()
                .map(|layer| layer.chars().nth(i).unwrap())
                .find(|&c| c != '2')
                .unwrap()
        })
        .collect()
}

fn to_layers(image: &str, layer_size: usize) -> Vec<&str> {
    image
        .as_bytes()
        .chunks(layer_size)
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ones_times_twos() {
        assert_eq!(
            1,
            ones_times_twos_for_fewest_zeros(&to_layers("123456789012", 6))
        );
    }

    #[test]
    fn test_decode_image() {
        assert_eq!(
            "0110",
            decode_image(&to_layers("0222112222120000", 4), 4)
                .iter()
                .collect::<String>()
        );
    }
}
