use itertools::Itertools;
use lib::{get_args, INVALID_INPUT};
use std::{
    error::Error,
    io::{stdin, BufRead},
    iter::zip,
    process::exit,
};

// t: time of the race
// m: max distance
// h: time to hold the button
// s: time to sail
// d: distance sailed
// v: sailing speed, v = h
//
// We need to find all h such that:
// h + s = t and d > m
// s x v > m
// (t - h) x h > m
// -h^2 + t x h - m > 0
//
// That's a quadratic equation with:
// delta = t^2 - 4 x m
// x1 = (t - sqrt(t^2 - 4 x m)) / 2
// x2 = (t + sqrt(t^2 - 4 x m)) / 2
// The solutions are the integer x such that x1 < x < x2

fn usage(prog_name: String) {
    println!("Usage: {} [-1|-2|-h]", prog_name);
    exit(0)
}

fn main() -> Result<(), Box<dyn Error>> {
    let (prog_name, args) = get_args()?;

    match args.get(0) {
        Some(arg) if arg == "-1" => {
            let input = stdin()
                .lock()
                .lines()
                .process_results(|itr| parse_races(itr))??;

            let result = solve(input.into_iter())?;

            println!("{}", result)
        }
        Some(arg) if arg == "-2" => {
            let input = stdin()
                .lock()
                .lines()
                .process_results(|itr| parse_race(itr))??;

            let result = solve_race(input)?;

            println!("{}", result)
        }
        _ => usage(prog_name),
    }
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
struct Race {
    time: u64,
    distance: u64,
}

fn parse_line1(s: String, header: String) -> Result<Vec<u64>, Box<dyn Error>> {
    let without_header = s
        .strip_prefix(&header)
        .ok_or::<Box<dyn Error>>(INVALID_INPUT.into())?;

    without_header
        .split_whitespace()
        .map(|s| s.parse::<u64>())
        .collect::<Result<Vec<u64>, _>>()
        .map_err(|e| e.into())
}

fn parse_races(itr: impl Iterator<Item = String>) -> Result<Vec<Race>, Box<dyn Error>> {
    let mut itr = itr;

    let first_line = itr.next().ok_or::<Box<dyn Error>>(INVALID_INPUT.into())?;
    let times = parse_line1(first_line, "Time:".into())?;

    let second_line = itr.next().ok_or::<Box<dyn Error>>("".into())?;
    let distances = parse_line1(second_line, "Distance:".into())?;

    Ok(zip(times, distances)
        .map(|(time, distance)| Race { time, distance })
        .collect())
}

fn parse_line2(s: String, header: String) -> Result<u64, Box<dyn Error>> {
    let without_header = s
        .strip_prefix(&header)
        .ok_or::<Box<dyn Error>>(INVALID_INPUT.into())?;

    without_header
        .chars()
        .filter(|c| c.is_digit(10))
        .collect::<String>()
        .parse::<u64>()
        .map_err(|e| e.into())
}

fn parse_race(itr: impl Iterator<Item = String>) -> Result<Race, Box<dyn Error>> {
    let mut itr = itr;

    let first_line = itr.next().ok_or::<Box<dyn Error>>(INVALID_INPUT.into())?;
    let time = parse_line2(first_line, "Time:".into())?;

    let second_line = itr.next().ok_or::<Box<dyn Error>>("".into())?;
    let distance = parse_line2(second_line, "Distance:".into())?;

    Ok(Race { time, distance })
}

fn solve(races: impl Iterator<Item = Race>) -> Result<u64, Box<dyn Error>> {
    races.map(solve_race).product()
}

// As is safe to use in this case. It's the only way to cast a float to an integer.
fn solve_race(input: Race) -> Result<u64, Box<dyn Error>> {
    let x1 = (input.time as f64
        - ((input.time as f64).powi(2) - 4.0 * input.distance as f64).sqrt())
        / 2.0;

    let x2 = (input.time as f64
        + ((input.time as f64).powi(2) - 4.0 * input.distance as f64).sqrt())
        / 2.0;

    u64::try_from(((x1 + 1.).floor() as u64..=(x2 - 1.).ceil() as u64).count())
        .map_err(|e| e.into())
}

#[cfg(test)]
mod day06 {
    use std::{
        error::Error,
        fs::File,
        io::{BufRead, BufReader},
    };

    use itertools::Itertools;

    use crate::{parse_race, parse_races, solve, solve_race, Race};

    const EXAMPLE: &str = "\
        Time:      7  15   30\n\
        Distance:  9  40  200";
    fn race1() -> Race {
        Race {
            time: 7,
            distance: 9,
        }
    }

    fn race2() -> Race {
        Race {
            time: 15,
            distance: 40,
        }
    }

    fn race3() -> Race {
        Race {
            time: 30,
            distance: 200,
        }
    }

    fn example1() -> Vec<Race> {
        vec![race1(), race2(), race3()]
    }

    fn example2() -> Race {
        Race {
            time: 71530,
            distance: 940200,
        }
    }

    #[test]
    fn parse_races_() -> Result<(), Box<dyn Error>> {
        assert_eq!(
            parse_races(EXAMPLE.lines().map(|s| s.to_string()))?,
            example1()
        );
        Ok(())
    }

    #[test]
    fn parse_race_() -> Result<(), Box<dyn Error>> {
        assert_eq!(
            parse_race(EXAMPLE.lines().map(|s| s.to_string()))?,
            example2()
        );
        Ok(())
    }

    #[test]
    fn solve_race_() -> Result<(), Box<dyn Error>> {
        assert_eq!(solve_race(race1())?, 4);
        assert_eq!(solve_race(race2())?, 8);
        assert_eq!(solve_race(race3())?, 9);
        Ok(())
    }

    #[test]
    fn solve_race2() -> Result<(), Box<dyn Error>> {
        assert_eq!(solve_race(example2())?, 71503);
        Ok(())
    }

    #[test]
    fn input_solve1() -> Result<(), Box<dyn Error>> {
        let file = File::open("input")?;
        let reader = BufReader::new(file);
        let input = reader.lines().process_results(|itr| parse_races(itr))??;

        assert_eq!(solve(input.into_iter())?, 170000);
        Ok(())
    }

    #[test]
    fn input_solve2() -> Result<(), Box<dyn Error>> {
        let file = File::open("input")?;
        let reader = BufReader::new(file);
        let input = reader.lines().process_results(|itr| parse_race(itr))??;

        assert_eq!(solve_race(input)?, 20537782);
        Ok(())
    }
}
