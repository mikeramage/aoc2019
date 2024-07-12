use crate::utils;
use regex::Regex;
use std::collections::HashMap;

pub fn day14() -> (usize, usize) {
    let input: Vec<String> = utils::parse_input("input/day14.txt");
    let reactions: HashMap<String, (usize, HashMap<String, usize>)> = parse_reactions(&input);
    let mut spares: HashMap<String, usize> = HashMap::new();
    let part1 = ore_required(&reactions, &mut spares);

    // parse each line of input to a reaction - reactants and product - and store in suitable data structure
    // from FUEL reaction work back to calculate how many of each reactant is required, and the reactant's reactant
    // and so on back to ORE.
    //
    // suitable data structure will require chemical and number - perhaps a map for each product
    // strongly suggests a recursive algorithm.
    //
    // Going to start with a hashmap of product to (num produced, Map of reactant to num required). product and reactant
    // are strings, num produced, required are usizes. If that doesn't work we'll revisit.
    spares = HashMap::new();
    let part2 = total_fuel_possible(&reactions, &mut spares);

    (part1, part2)
}

fn parse_reactions(input: &[String]) -> HashMap<String, (usize, HashMap<String, usize>)> {
    let re = Regex::new(r"(?<reactants>(\d+ \w+,?\s?)+) => (?<produced>\d+) (?<product>\w+)")
        .unwrap_or_else(|err| panic!("Bad regex: {err}"));
    // input.iter().map(|reaction| )
    let reactions: HashMap<String, (usize, HashMap<String, usize>)> =
        HashMap::from_iter(input.iter().map(|reaction| {
            let caps = re.captures(reaction).unwrap();
            // let mut reactants: HashMap<String, usize> = HashMap::new();
            // for reactant in caps["reactants"].trim().split(',') {
            //     let reactant = reactant.trim();
            //     let mut num_chemical = reactant.split(' ');
            //     let num_required = num_chemical.next().unwrap().parse::<usize>().unwrap();
            //     let chemical = String::from(num_chemical.next().unwrap());
            //     reactants.insert(chemical, num_required);
            // }
            let reactants = HashMap::from_iter(caps["reactants"].trim().split(',').map(|s| {
                s.trim()
                    .split(' ')
                    .collect::<Vec<&str>>()
                    .chunks(2)
                    .map(|c| (String::from(c[1]), c[0].parse::<usize>().unwrap()))
                    .collect::<Vec<(String, usize)>>()[0]
                    .clone()
            }));
            (
                (caps["product"]).to_string(),
                (caps["produced"].parse::<usize>().unwrap(), reactants),
            )
        }));
    reactions
}

fn ore_required(
    reactions: &HashMap<String, (usize, HashMap<String, usize>)>,
    spares: &mut HashMap<String, usize>,
) -> usize {
    let fuel = reactions.get("FUEL").unwrap();
    let num_produced = fuel.0;
    let mut fuel_reaction = fuel.1.clone();
    assert_eq!(1, num_produced); //FUEL only comes in 1.
    let mut fully_reduced = false;

    while !fully_reduced {
        //Reduce one level
        let mut temp: HashMap<String, usize> = HashMap::new();
        fully_reduced = true;
        for (reactant, num_required) in &fuel_reaction {
            if reactant == "ORE" {
                temp.entry(reactant.clone())
                    .and_modify(|x| *x += *num_required)
                    .or_insert_with(|| *num_required);
                continue; //Can't reduce further
            }
            fully_reduced = false; //If we get here, we must have at least one reactant not reduced to ore.
            let (batch_size, sub_reaction) = reactions.get(reactant).unwrap();
            let mut actual_required = *num_required;
            if let Some(num_spares) = spares.get(reactant) {
                if num_spares >= num_required {
                    actual_required = 0;
                    spares
                        .entry(reactant.clone())
                        .and_modify(|x| *x -= num_required);
                } else {
                    actual_required = num_required - num_spares;
                    spares.remove(reactant.as_str());
                }
            }
            let num_batches = actual_required.div_ceil(*batch_size);
            if num_batches * batch_size > actual_required {
                spares.insert(
                    reactant.clone(),
                    (num_batches * batch_size) - actual_required,
                );
            }
            for (sub_reactant, sub_required) in sub_reaction {
                let total_subreactants_required = num_batches * sub_required;
                temp.entry(sub_reactant.clone())
                    .and_modify(|x| *x += total_subreactants_required)
                    .or_insert_with(|| total_subreactants_required);
            }
        }

        fuel_reaction = temp;
    }

    //By this stage we've fully reduced the fuel reaction. Just pull out the amount of ore required.
    *fuel_reaction.get("ORE").unwrap()
}

#[allow(unused_variables)]
fn total_fuel_possible(
    reactions: &HashMap<String, (usize, HashMap<String, usize>)>,
    spares: &mut HashMap<String, usize>,
) -> usize {
    // let fuel = 0;
    //This naive algorithm takes too long, especially for the unit tests (though it works for the
    // main day14 after 2 and a half minutes on releast build). Presumably the set of spares repeats
    // let mut ore_remaining: usize = 1_000_000_000_000;
    // let mut fuel = 0;
    // loop {
    //     let ore_required = ore_required(&reactions, spares);
    //     if ore_remaining > ore_required {
    //         fuel += 1;
    //         ore_remaining -= ore_required;
    //     } else {
    //         break;
    //     }
    // }

    // This works for all but the final unit test and the main day14 - no repeats.
    //
    // let mut ore_remaining: usize = 1_000_000_000_000;
    // let mut fuel: usize;
    // let mut list_of_spares: Vec<HashMap<String, usize>> = vec![];
    // let mut list_of_ore_required: Vec<usize> = vec![];

    // loop {
    //     let ore_required = ore_required(&reactions, spares);

    //     if list_of_spares.contains(&spares.clone()) {
    //         //Got a repeat
    //         break;
    //     } else {
    //         list_of_spares.push(spares.clone());
    //         list_of_ore_required.push(ore_required);
    //     }
    // }

    // let ore_required_to_reach_repeat = list_of_ore_required.iter().sum::<usize>();
    // let num_repeats = ore_remaining / ore_required_to_reach_repeat;
    // fuel = num_repeats * list_of_ore_required.len();
    // ore_remaining -= ore_required_to_reach_repeat * num_repeats;
    // 'outer: loop {
    //     for ore_required in &list_of_ore_required {
    //         if ore_remaining > *ore_required {
    //             fuel += 1;
    //             ore_remaining -= ore_required;
    //         } else {
    //             break 'outer;
    //         }
    //     }
    // }
    //
    // fuel
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ore_required() {
        let mut input: Vec<String> = "10 ORE => 10 A
1 ORE => 1 B
7 A, 1 B => 1 C
7 A, 1 C => 1 D
7 A, 1 D => 1 E
7 A, 1 E => 1 FUEL"
            .split('\n')
            .map(|s| String::from(s))
            .collect();

        let mut reactions = parse_reactions(&input);
        let mut spares: HashMap<String, usize> = HashMap::new();
        assert_eq!(31, ore_required(&reactions, &mut spares));

        input = "9 ORE => 2 A
8 ORE => 3 B
7 ORE => 5 C
3 A, 4 B => 1 AB
5 B, 7 C => 1 BC
4 C, 1 A => 1 CA
2 AB, 3 BC, 4 CA => 1 FUEL"
            .split('\n')
            .map(|s| String::from(s))
            .collect();
        reactions = parse_reactions(&input);
        spares = HashMap::new();
        assert_eq!(165, ore_required(&reactions, &mut spares));

        input = "157 ORE => 5 NZVS
165 ORE => 6 DCFZ
44 XJWVT, 5 KHKGT, 1 QDVJ, 29 NZVS, 9 GPVTF, 48 HKGWZ => 1 FUEL
12 HKGWZ, 1 GPVTF, 8 PSHF => 9 QDVJ
179 ORE => 7 PSHF
177 ORE => 5 HKGWZ
7 DCFZ, 7 PSHF => 2 XJWVT
165 ORE => 2 GPVTF
3 DCFZ, 7 NZVS, 5 HKGWZ, 10 PSHF => 8 KHKGT"
            .split('\n')
            .map(|s| String::from(s))
            .collect();
        reactions = parse_reactions(&input);
        spares = HashMap::new();
        assert_eq!(13312, ore_required(&reactions, &mut spares));
        // spares = HashMap::new();
        // assert_eq!(82892753, total_fuel_possible(&reactions, &mut spares));

        input = "2 VPVL, 7 FWMGM, 2 CXFTF, 11 MNCFX => 1 STKFG
17 NVRVD, 3 JNWZP => 8 VPVL
53 STKFG, 6 MNCFX, 46 VJHF, 81 HVMC, 68 CXFTF, 25 GNMV => 1 FUEL
22 VJHF, 37 MNCFX => 5 FWMGM
139 ORE => 4 NVRVD
144 ORE => 7 JNWZP
5 MNCFX, 7 RFSQX, 2 FWMGM, 2 VPVL, 19 CXFTF => 3 HVMC
5 VJHF, 7 MNCFX, 9 VPVL, 37 CXFTF => 6 GNMV
145 ORE => 6 MNCFX
1 NVRVD => 8 CXFTF
1 VJHF, 6 MNCFX => 4 RFSQX
176 ORE => 6 VJHF"
            .split('\n')
            .map(|s| String::from(s))
            .collect();
        reactions = parse_reactions(&input);
        spares = HashMap::new();
        assert_eq!(180697, ore_required(&reactions, &mut spares));
        // spares = HashMap::new();
        // assert_eq!(5586022, total_fuel_possible(&reactions, &mut spares));

        input = "171 ORE => 8 CNZTR
7 ZLQW, 3 BMBT, 9 XCVML, 26 XMNCP, 1 WPTQ, 2 MZWV, 1 RJRHP => 4 PLWSL
114 ORE => 4 BHXH
14 VRPVC => 6 BMBT
6 BHXH, 18 KTJDG, 12 WPTQ, 7 PLWSL, 31 FHTLT, 37 ZDVW => 1 FUEL
6 WPTQ, 2 BMBT, 8 ZLQW, 18 KTJDG, 1 XMNCP, 6 MZWV, 1 RJRHP => 6 FHTLT
15 XDBXC, 2 LTCX, 1 VRPVC => 6 ZLQW
13 WPTQ, 10 LTCX, 3 RJRHP, 14 XMNCP, 2 MZWV, 1 ZLQW => 1 ZDVW
5 BMBT => 4 WPTQ
189 ORE => 9 KTJDG
1 MZWV, 17 XDBXC, 3 XCVML => 2 XMNCP
12 VRPVC, 27 CNZTR => 2 XDBXC
15 KTJDG, 12 BHXH => 5 XCVML
3 BHXH, 2 VRPVC => 7 MZWV
121 ORE => 7 VRPVC
7 XCVML => 6 RJRHP
5 BHXH, 4 VRPVC => 5 LTCX"
            .split('\n')
            .map(|s| String::from(s))
            .collect();
        reactions = parse_reactions(&input);
        spares = HashMap::new();
        assert_eq!(2210736, ore_required(&reactions, &mut spares));
        // spares = HashMap::new();
        // assert_eq!(460664, total_fuel_possible(&reactions, &mut spares));
    }
}
