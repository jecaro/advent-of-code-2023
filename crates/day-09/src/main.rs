use itertools::Itertools;
use lib::get_args;
use std::{
    error::Error,
    io::{stdin, BufRead},
    process::exit,
};

fn usage(prog_name: String) {
    println!("Usage: {} [-1|-2|-h]", prog_name);
    exit(0)
}

fn main() -> Result<(), Box<dyn Error>> {
    let (prog_name, args) = get_args()?;

    match args.get(0) {
        Some(arg) if arg == "-1" || arg == "-2" => {
            let solve_line = match arg.as_str() {
                "-1" => solve_line1,
                _ => solve_line2,
            };

            let result = stdin()
                .lock()
                .lines()
                .process_results(|itr| solve(itr, solve_line))??;

            println!("{}", result)
        }
        _ => usage(prog_name),
    }
    Ok(())
}

fn parse_line(line: String) -> Result<Vec<i32>, Box<dyn Error>> {
    line.split_whitespace()
        .map(|s| s.parse::<i32>().map_err(|e| e.into()))
        .collect()
}

fn solve_line1(numbers: Vec<i32>) -> Result<i32, Box<dyn Error>> {
    if numbers.iter().all(|n| *n == 0) {
        return Ok(0);
    } else {
        let offsets: Vec<_> = numbers
            .windows(2)
            .map(|w| {
                let x0 = w.get(0).ok_or("No first element")?;
                let x1 = w.get(1).ok_or("No second element")?;
                Ok(x1 - x0)
            })
            .collect::<Result<_, Box<dyn Error>>>()?;
        let offsets_result = solve_line1(offsets)?;
        let last_number = numbers.last().ok_or("No last number")?;
        Ok(last_number + offsets_result)
    }
}

fn solve_line2(numbers: Vec<i32>) -> Result<i32, Box<dyn Error>> {
    let numbers: Vec<_> = numbers.into_iter().rev().collect();
    solve_line1(numbers)
}

fn solve(
    itr: impl Iterator<Item = String>,
    solve_line: fn(Vec<i32>) -> Result<i32, Box<dyn Error>>,
) -> Result<i32, Box<dyn Error>> {
    itr.map(|line| {
        let parsed_lined = parse_line(line)?;
        solve_line(parsed_lined)
    })
    .sum()
}

#[cfg(test)]
mod day09 {
    use std::{
        error::Error,
        fs::File,
        io::{BufRead, BufReader},
    };

    use itertools::Itertools;

    use crate::{parse_line, solve, solve_line1, solve_line2};

    const LINE1: &str = "0 3 6 9 12 15";
    fn line1() -> Vec<i32> {
        vec![0, 3, 6, 9, 12, 15]
    }
    const LINE2: &str = "1 3 6 10 15 21";
    fn line2() -> Vec<i32> {
        vec![1, 3, 6, 10, 15, 21]
    }
    const LINE3: &str = "10 13 16 21 30 45";
    fn line3() -> Vec<i32> {
        vec![10, 13, 16, 21, 30, 45]
    }

    fn example() -> Vec<String> {
        vec![LINE1.to_string(), LINE2.to_string(), LINE3.to_string()]
    }

    #[test]
    fn test_parse_lines() -> Result<(), Box<dyn Error>> {
        let parsed_line1 = parse_line(LINE1.to_string())?;
        assert_eq!(parsed_line1, line1());

        let parsed_line2 = parse_line(LINE2.to_string())?;
        assert_eq!(parsed_line2, line2());

        let parsed_line3 = parse_line(LINE3.to_string())?;
        assert_eq!(parsed_line3, line3());

        Ok(())
    }

    #[test]
    fn test_solve1_line1() -> Result<(), Box<dyn Error>> {
        assert_eq!(solve_line1(line1())?, 18);

        Ok(())
    }

    #[test]
    fn test_solve2_line1() -> Result<(), Box<dyn Error>> {
        assert_eq!(solve_line2(line1())?, -3);

        Ok(())
    }

    #[test]
    fn test_solve1_line2() -> Result<(), Box<dyn Error>> {
        assert_eq!(solve_line1(line2())?, 28);

        Ok(())
    }

    #[test]
    fn test_solve2_line2() -> Result<(), Box<dyn Error>> {
        assert_eq!(solve_line2(line2())?, 0);

        Ok(())
    }

    #[test]
    fn test_solve1_line3() -> Result<(), Box<dyn Error>> {
        assert_eq!(solve_line1(line3())?, 68);

        Ok(())
    }

    #[test]
    fn test_solve2_line3() -> Result<(), Box<dyn Error>> {
        assert_eq!(solve_line2(line3())?, 5);

        Ok(())
    }

    #[test]
    fn test_solve1_lines() -> Result<(), Box<dyn Error>> {
        let result = solve(example().into_iter(), solve_line1)?;
        assert_eq!(result, 114);

        Ok(())
    }

    #[test]
    fn test_solve2_lines() -> Result<(), Box<dyn Error>> {
        let result = solve(example().into_iter(), solve_line2)?;
        assert_eq!(result, 2);

        Ok(())
    }

    #[test]
    fn test_solve1_input() -> Result<(), Box<dyn Error>> {
        let file = File::open("input")?;
        let reader = BufReader::new(file);
        let result = reader
            .lines()
            .process_results(|itr| solve(itr, solve_line1))??;
        assert_eq!(result, 1969958987);

        Ok(())
    }

    #[test]
    fn test_solve2_input() -> Result<(), Box<dyn Error>> {
        let file = File::open("input")?;
        let reader = BufReader::new(file);
        let result = reader
            .lines()
            .process_results(|itr| solve(itr, solve_line2))??;

        assert_eq!(result, 1068);
        Ok(())
    }
}
