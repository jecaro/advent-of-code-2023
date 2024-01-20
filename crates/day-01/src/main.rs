use itertools::Itertools;
use lib::get_args;
use std::{
    collections::HashMap,
    error::Error,
    io::{stdin, BufRead},
    process::exit,
};

fn numbers() -> HashMap<String, char> {
    let values = [
        ("one", '1'),
        ("two", '2'),
        ("three", '3'),
        ("four", '4'),
        ("five", '5'),
        ("six", '6'),
        ("seven", '7'),
        ("eight", '8'),
        ("nine", '9'),
    ];
    values
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_owned()))
        .collect()
}

fn first_last(s: &str) -> Result<u32, Box<dyn Error>> {
    let first = s.chars().next().ok_or(Into::<Box<dyn Error>>::into(
        "Cant get the first char of {s}",
    ))?;
    let last = s
        .chars()
        .last()
        .ok_or(Into::<Box<dyn Error>>::into(format!(
            "Cant get the last char of {s}"
        )))?;

    let number = format!("{}{}", first, last);

    number.parse::<u32>().map_err(Into::into)
}

fn solve1(itr: impl Iterator<Item = String>) -> Result<u32, Box<dyn Error>> {
    itr.map(|s| (s.chars().filter(|c| c.is_numeric()).collect::<String>()))
        .map(|s| first_last(&s))
        .sum()
}

fn solve2(itr: impl Iterator<Item = String>) -> Result<u32, Box<dyn Error>> {
    let table = numbers();
    itr.map(|s| {
        // loop over the chars
        s.chars()
            .enumerate()
            .map(|(i, c)| {
                // loop over the table
                table
                    .iter()
                    // if the string starting at i matches a key, return the replacing char
                    .find_map(|(key, value)| s[i..].starts_with(key).then(|| value))
                    // otherwise return the original char
                    .map_or(c, |value| value.to_owned())
            })
            // keep only chars that convert to numeric
            .filter(|c| c.is_numeric())
            .collect::<String>()
    })
    // now take the first and last numeric char
    .map(|s| first_last(&s))
    // and get the sum
    .sum()
}

fn usage(prog_name: String) {
    println!("Usage: {} [-1|-2|-h]", prog_name);
    exit(0)
}

fn main() -> Result<(), Box<dyn Error>> {
    let (prog_name, args) = get_args()?;

    match args.get(0) {
        Some(arg) if arg == "-1" || arg == "-2" => {
            let result = stdin().lock().lines().process_results(|itr| {
                let solve: fn(_) -> Result<u32, Box<dyn Error>> = match arg.as_str() {
                    "-1" => solve1,
                    _ => solve2,
                };
                solve(itr)
            })??;

            println!("{}", result)
        }
        _ => usage(prog_name),
    }
    Ok(())
}

#[cfg(test)]
mod day01 {
    use itertools::Itertools;
    use std::error::Error;
    use std::fs::File;
    use std::io::BufRead;
    use std::io::BufReader;

    use crate::solve1;
    use crate::solve2;

    const INPUT1: &str = "\
        1abc2\n\
        pqr3stu8vwx\n\
        a1b2c3d4e5f\n\
        treb7uchet";

    const INPUT2: &str = "\
        two1nine\n\
        eightwothree\n\
        abcone2threexyz\n\
        xtwone3four\n\
        4nineeightseven2\n\
        zoneight234\n\
        7pqrstsixteen";

    #[test]
    fn example1_solve1() -> Result<(), Box<dyn Error>> {
        assert_eq!(solve1(INPUT1.lines().map(|s| s.to_string()))?, 142);
        Ok(())
    }

    #[test]
    fn example1_solve2() -> Result<(), Box<dyn Error>> {
        assert_eq!(solve2(INPUT1.lines().map(|s| s.to_string()))?, 142);
        Ok(())
    }

    #[test]
    fn input_solve1() -> Result<(), Box<dyn Error>> {
        let file = File::open("input")?;
        let reader = BufReader::new(file);
        let result = reader.lines().process_results(|itr| solve1(itr))??;

        assert_eq!(result, 56397);
        Ok(())
    }

    #[test]
    fn input_solve2() -> Result<(), Box<dyn Error>> {
        let file = File::open("input")?;
        let reader = BufReader::new(file);
        let result = reader.lines().process_results(|itr| solve2(itr))??;

        assert_eq!(result, 55701);
        Ok(())
    }

    #[test]
    fn example2_solve2() -> Result<(), Box<dyn Error>> {
        assert_eq!(solve2(INPUT2.lines().map(|s| s.to_string()))?, 281);
        Ok(())
    }
}
