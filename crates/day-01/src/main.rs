use lib::get_args;
use std::{
    collections::HashMap,
    error::Error,
    io::{self, stdin, BufRead},
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

fn solve1(itr: impl Iterator<Item = io::Result<String>>) -> Result<u32, Box<dyn Error>> {
    itr.map(|s| Ok::<String, Box<dyn Error>>(s?.chars().filter(|c| c.is_numeric()).collect()))
        .map(|s| first_last(&s?))
        .sum()
}

fn solve2(itr: impl Iterator<Item = io::Result<String>>) -> Result<u32, Box<dyn Error>> {
    let table = numbers();
    itr.map(|s| {
        let s_ = s?;
        // loop over the chars
        Ok::<String, Box<dyn Error>>(
            s_.chars()
                .enumerate()
                .map(|(i, c)| {
                    // loop over the table
                    table
                        .iter()
                        // if the string starting at i matches a key, return the replacing char
                        .find_map(|(key, value)| s_[i..].starts_with(key).then(|| value))
                        // otherwise return the original char
                        .map_or(c, |value| value.to_owned())
                })
                // keep only chars that convert to numeric
                .filter(|c| c.is_numeric())
                .collect::<String>(),
        )
    })
    // now take the first and last numeric char
    .map(|s| first_last(&s?))
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
            let solve = match arg.as_str() {
                "-1" => solve1,
                _ => solve2,
            };

            let result = solve(stdin().lock().lines())?;

            println!("{}", result)
        }
        _ => usage(prog_name),
    }
    Ok(())
}

#[cfg(test)]
mod day01 {
    use std::fs::File;
    use std::io;
    use std::io::BufRead;
    use std::io::BufReader;

    use crate::solve1;
    use crate::solve2;

    fn input1() -> Vec<io::Result<String>> {
        vec!["1abc2", "pqr3stu8vwx", "a1b2c3d4e5f", "treb7uchet"]
            .iter()
            .map(|s| Ok(s.to_string()))
            .collect()
    }

    #[test]
    fn example1_solve1() {
        assert_eq!(solve1(input1().into_iter()).unwrap(), 142);
    }

    #[test]
    fn example1_solve2() {
        assert_eq!(solve2(input1().into_iter()).unwrap(), 142);
    }

    #[test]
    fn input_solve1() {
        let file = File::open("input").unwrap();
        let reader = BufReader::new(file);

        assert_eq!(solve1(reader.lines()).unwrap(), 56397);
    }

    #[test]
    fn input_solve2() {
        let file = File::open("input").unwrap();
        let reader = BufReader::new(file);

        assert_eq!(solve2(reader.lines()).unwrap(), 55701);
    }

    #[test]
    fn example2_solve2() {
        let input: Vec<_> = vec![
            "two1nine",
            "eightwothree",
            "abcone2threexyz",
            "xtwone3four",
            "4nineeightseven2",
            "zoneight234",
            "7pqrstsixteen",
        ]
        .iter()
        .map(|s| Ok(s.to_string()))
        .collect();

        assert_eq!(solve2(input.into_iter()).unwrap(), 281);
    }
}
