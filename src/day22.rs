use std::fs;

//Constants for part 2
const N: i128 = 119315717514047;
const M: i128 = 101741582076661;

#[derive(Debug, Clone, Copy)]
enum Technique {
    New,
    Cut(i128),
    Increment(i128),
}

impl Technique {
    fn apply(&self, cards: &mut [usize]) {
        match self {
            Self::New => cards.reverse(),
            Self::Cut(n) => {
                let left: &[usize];
                let right: &[usize];
                if *n > 0 {
                    (left, right) = cards.split_at(*n as usize);
                } else {
                    (left, right) = cards.split_at(cards.len() - n.unsigned_abs() as usize);
                }
                let mut tmp: Vec<usize> = right.to_vec();
                tmp.append(&mut left.to_vec());
                cards.copy_from_slice(tmp.as_slice()); //Perhaps would be better to have apply return a new Vec<usize>
            }
            Self::Increment(n) => {
                let mut tmp: Vec<usize> = Vec::new();
                cards.clone_into(&mut tmp);
                for (index, card) in cards.iter().enumerate() {
                    tmp[(index * *n as usize) % cards.len()] = *card;
                }
                cards.copy_from_slice(tmp.as_slice());
            }
        }
    }
}

pub fn day22() -> (usize, usize) {
    let input = fs::read_to_string("input/day22.txt").expect("Could not read input file");
    let mut techniques = parse_into_techniques(&input);
    let mut deck: Vec<usize> = (0..10007).collect::<Vec<usize>>();

    for technique in &techniques {
        technique.apply(&mut deck);
    }

    let part1 = deck
        .iter()
        .enumerate()
        .filter(|(_, card)| **card == 2019)
        .map(|(index, _)| index)
        .max()
        .unwrap();

    //Of course, part 2 makes the method for part 1 redundant due to size of deck and number of shuffles.
    //Each technique is O(n), which means O(n*m) ~ 10^28, which ain't doable this side of the GNAB GIB / heat death of the universe.
    //Strategy is going to be to find out what happens to index i after some arbitrary combination of the 3 techniques.
    //I'm off to do some figuring.

    //Well that was some hard figuring. I feel like I've been rederiving some results in modular arithmetic and
    //could probably have benefited from some research. Here's what I've discovered. For a deck of N cards
    // - New is equivalent to Increment(N-1) followed by C(1).
    // - Cut(m) followed by Increment(n) is equivalent to Increment(n) followed by Cut(m*n mod N).
    // - (rather more trivially) Cut(m) followed by Cut(n) is Cut((m + n) mod N), and Increment(m)
    // followed by Increment(n) is Increment((m * n) mod N).
    //
    // So the approach is:
    // - Convert all New techniques to Increment(N-1)C(1).
    // - While there are no more swaps to be made, go through in pairs, swapping C(m) I(n) for I(n) C(m*n mod N). At that point all
    //   the Increments are at the top and Cuts at the bottom.
    // - Combine the Increments to give a single Increment and the Cuts to give a single Cut.
    // - The techniques should now be just I(x) followed by C(y), where x is the combined increment and y the combined cut.
    //   x and y are likely big, but as long as I've been applying the modulo as I go, they'll still be within a i128.
    // - But there will now be trillions - M - copies of the shuffles. So I(x) followed by C(y) M times. A bunch more figuring shows
    //   this reduces to I(x^M mod N) followed by C(y*(1 + x + x^2 + ... + x^(M-1)) mod N). Clearly these will overflow, so I'll need to do a bit more
    //   figuring out how to reduce them to something manageable and invert the formula to get the old index in terms of the new index 2020.
    //   And who knows how I can deal with the power series in x!?
    // - OK - so I gave up here :( and sought some mathematical help. Two clues - geometric series and modular inverse. The latter I
    //   need because I want the value at position 2020 - so I need to start from that and work back to find out what index that value
    //   was initially at. I need to figure out the modular inverse of x so I can invert I. C is easy to invert - it's just N-y.
    // - Finally I need to figure out C^-1 I^-1 acting M times on index 2020.

    // Part 1 - convert all New to Increment(N-1) C(1)

    //Overflow is a risk. i128 should do the trick though. Max i128 is 1.7e38 while N and M are 1e14, so I'm fine because everything is
    //modulo 1e14 and worst case is I'm multiplying 2 O(1e14) numbers together.

    techniques = techniques
        .iter()
        .flat_map(|technique| {
            if matches!(technique, Technique::New) {
                //Temporarily map to vectors so if/else branches have same types
                vec![Technique::Increment(N - 1), Technique::Cut(1)] //New maps to Increment(N-1), C(1)
            } else if let Technique::Cut(n) = *technique {
                if n < 0 {
                    vec![Technique::Cut(N + n)] //C(n) = C(N-n) for n < 0; e.g. C(-2) = C(8)
                } else {
                    vec![*technique]
                }
            } else {
                vec![*technique] //Otherwise keep the element as it was
            }
        })
        .collect(); //Flatten to get rid of the vector substructure and insert the New replacements

    loop {
        let mut swapped = false; // Track whether we've done any swaps
        for i in 0..(techniques.len() - 1) {
            if let (Technique::Cut(m), Technique::Increment(n)) = (techniques[i], techniques[i + 1])
            {
                techniques[i] = Technique::Increment(n);
                techniques[i + 1] = Technique::Cut((n * m) % N);
                swapped = true; // Got C(m), I(n), swapping for I(n) C(n*m mod N)
            }
        }

        if !swapped {
            break; //No more swapped. We're now ordered
        }
    }

    //Compress all the increments and cuts into a single Increment and Cut.
    let mut increments: i128 = 1;
    let mut cuts: i128 = 0;
    for t in &techniques {
        match *t {
            Technique::Increment(n) => {
                increments *= n;
                increments %= N;
            }
            Technique::Cut(m) => {
                cuts += m;
                cuts %= N;
            }
            _ => unreachable!("Aaargh"),
        }
    }

    //We've got to the stage where we have reduced a single transformation to one Increment followd by one Cut. We know
    // i_m = I, C, I, C ... i_0, but want i_0 in terms of i_m - i.e. which index i_0 maps to the final position i_m = 2020.
    //For that we need the modular index of increments to be able to invert the transformation, i.e. i_0 = C^-1 I^-1 ...
    // C^-1 is easy; C(a)^-1 = C(N-a). But I(a)^-1 = I(a^-1), where a^-1 is the modular multiplicative inverse of a mod N (because I
    // is a modular multiplication of the index rather than an addition)
    //
    // There's probably a library for this but I'm trying to learn more about modular arithmetic so will implement euclid's extended
    // algorithm to calculate the modular inverse. See here: https://en.wikipedia.org/wiki/Modular_multiplicative_inverse
    let inv_increments = modular_inverse(increments, N);

    //C^-1 I^-1 acting M times on index 2020 will give us the original index it came from. We can read that off
    //to get the answer (x is inv_increments, y is cuts). After some scribbling
    // C^-1(y) I^-1(x) maps i -> [(i*x^M mod N) + y*(x + x^2 + ... + x^M) mod N] mod N
    // How to simplify? x + x^2 + ... +x^M is a geometric series x * (x^M - 1)/(x - 1).
    // Modular arithmetic means * (x-1)^-1 rather than divide.
    // So the whole thing boils down to:
    // [(i* x^M mod N) + (y*x mod N) * (((x^M mod N) * (x - 1)^-1) mod N - (x * y mod N * (x - 1)^-1) mod N) ] mod N
    // Phew.
    // To calculate x^M mod N, use exponentiation in powers of 2 to calculate (see https://codeforces.com/blog/entry/72527)
    // Let's use the notation above for, ahem, clarity
    let x = inv_increments;
    let x_min_1_inv = modular_inverse(x - 1, N);
    let x_to_m_mod_n = pow_mod(x, M, N); //Yes, M and N are swapped.
    let i: i128 = 2020;
    let y = cuts;
    //Fingers crossed
    let part2 = ((i * x_to_m_mod_n) % N + (((x * y) % N) * ((x_to_m_mod_n * x_min_1_inv) % N))
        - (((x * y) % N) * x_min_1_inv) % N)
        % N;

    //Woohoo!!! I should really tidy this all up to make it vaguely understandable, but nah. It was pretty warty - code should reflect that.
    //So what help did I need?
    // - Understanding how to simplify x + x^2 + ... + x^M as a geometric series (this was just a basic simple math failure to spot it *was* one - I could remember the trick for
    //   summing the series without help once the words "geometric series" were in my head).
    // - Understanding modular multiplicative inverse, first to invert the increment operation, and then to handle the denominator on my geometric series (I didn't realise it was
    //   applicable to that second case initially)
    // - Understanding how to calculate both the modular inverse (Extended Euclidean algorithm, though later saw Fermat's Little Theorem gives a neat solution) and exponentiation
    //   using powers of 2.
    // Thanks to Reddit for the initial hint about mod inverse and geometric series, https://codeforces.com/blog/entry/72527 for the exponentiation (and Chris Patterson, whose solution
    // pointed me at that), Wikipedia and Khan Academy for general background on modular arithmetic.
    //
    // Other than that I figured out how to compress New, Increment and Cut into a single Increment and Cut myself, and how to express that in terms of transformation of the indices. It was
    // just converting that into something computable I needed the help with. I think it's probably reasonable that I didn't derive modular inverses, euclid's algorithm, exponentiation, from
    // first principles!

    (part1, part2 as usize)
}

//Use iterative version of the algorithm to calculate x^n mod m
fn pow_mod(x: i128, n: i128, m: i128) -> i128 {
    let mut y: i128 = 1;
    let mut x = x;
    let mut n = n;
    while n > 0 {
        if n % 2 != 0 {
            //Odd
            y = (y * x) % m;
        }
        n /= 2;
        x = (x * x) % m;
    }

    y
}

// Calculate the modular multiplicative inverse of a mod N. Use Extended Euclid algorithm to get Bezout t coefficient.
// See here for theory - https://en.wikipedia.org/wiki/Extended_Euclidean_algorithm
fn modular_inverse(a: i128, n: i128) -> i128 {
    let mut old_t: i128 = 0;
    let mut t: i128 = 1;
    let mut old_r = n;
    let mut r = a;

    //Extended Euclidean algorithm, but without bothering with s.
    while r > 0 {
        let q = old_r / r;
        (old_r, r) = (r, old_r - q * r); //r_i+1 = r_i-1 - q_i r_i. q_i is euclidean division of old_r / r.
        (old_t, t) = (t, old_t - q * t); //t_i+1 = t_i-1 - q_i t_i
    }

    // The Bezout coefficient we want is old_t. It might be negative in which case we'll return N-old_t
    if old_t > 0 {
        old_t
    } else {
        n + old_t
    }
}

fn parse_into_techniques(input: &str) -> Vec<Technique> {
    input
        .lines()
        .map(|line| {
            if line.starts_with("deal into new stack") {
                Technique::New
            } else if line.starts_with("cut") {
                let n = line
                    .replace("cut ", "")
                    .trim()
                    .parse::<i128>()
                    .unwrap_or_else(|_| panic!("Could not parse {} into int!", line));
                Technique::Cut(n)
            } else {
                let n = line
                    .replace("deal with increment ", "")
                    .trim()
                    .parse::<i128>()
                    .unwrap_or_else(|_| panic!("Could not parse {} into int!", line));
                Technique::Increment(n)
            }
        })
        .collect::<Vec<Technique>>()
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_simple_decks() {
        let mut deck: Vec<usize> = (0..10).collect::<Vec<usize>>();
        let mut techniques = parse_into_techniques(
            "deal with increment 7
deal into new stack
deal into new stack",
        );
        for technique in techniques {
            technique.apply(&mut deck);
        }

        assert_eq!(deck, vec![0, 3, 6, 9, 2, 5, 8, 1, 4, 7]);

        deck = (0..10).collect::<Vec<usize>>();
        techniques = parse_into_techniques(
            "cut 6
deal with increment 7
deal into new stack",
        );
        for technique in techniques {
            technique.apply(&mut deck);
        }

        assert_eq!(deck, vec![3, 0, 7, 4, 1, 8, 5, 2, 9, 6]);

        deck = (0..10).collect::<Vec<usize>>();
        techniques = parse_into_techniques(
            "deal with increment 7
deal with increment 9
cut -2",
        );
        for technique in techniques {
            technique.apply(&mut deck);
        }

        assert_eq!(deck, vec![6, 3, 0, 7, 4, 1, 8, 5, 2, 9]);

        deck = (0..10).collect::<Vec<usize>>();
        techniques = parse_into_techniques(
            "deal into new stack
cut -2
deal with increment 7
cut 8
cut -4
deal with increment 7
cut 3
deal with increment 9
deal with increment 3
cut -1",
        );
        for technique in techniques {
            technique.apply(&mut deck);
        }

        assert_eq!(deck, vec![9, 2, 5, 8, 1, 4, 7, 0, 3, 6]);
    }
}
