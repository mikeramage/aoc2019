use crate::utils;

///Day 1 solution
pub fn day1() -> (usize, usize) {
    let input: Vec<usize> = utils::parse_input("input/day1.txt");
    (
        input.iter().map(|x| fuel_from_mass(*x)).sum(),
        input.iter().map(|x| total_fuel(*x)).sum(),
    )
}

fn total_fuel(mass: usize) -> usize {
    let mut total_fuel = 0;
    let mut current_fuel = fuel_from_mass(mass);
    while current_fuel > 0 {
        total_fuel += current_fuel;
        current_fuel = fuel_from_mass(current_fuel);
    }

    total_fuel
}

fn fuel_from_mass(mass: usize) -> usize {
    (mass / 3).saturating_sub(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuel_from_mass() {
        assert_eq!(6, fuel_from_mass(24));
        assert_eq!(0, fuel_from_mass(1));
        assert_eq!(0, fuel_from_mass(3));
        assert_eq!(15, fuel_from_mass(53));
        assert_eq!(0, fuel_from_mass(0));
    }

    #[test]
    fn test_total_fuel() {
        assert_eq!(2, total_fuel(14));
        assert_eq!(966, total_fuel(1969));
        assert_eq!(50346, total_fuel(100756));
    }
}
