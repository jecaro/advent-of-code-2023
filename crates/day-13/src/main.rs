use itertools::Itertools;
use lib::get_args;
use std::{
    convert::identity,
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
            let result = stdin()
                .lock()
                .lines()
                .process_results(|itr| -> Result<i32, _> {
                    let patterns = parse(itr);
                    patterns
                        .iter()
                        .map(|p| {
                            if arg == "-1" {
                                solve_pattern1(p.iter().cloned())
                            } else {
                                solve_pattern2(p.iter().cloned())
                            }
                        })
                        .sum::<Result<i32, _>>()
                })??;

            println!("{}", result);
        }
        _ => usage(prog_name),
    }
    Ok(())
}

fn get_mirror_horizontally(
    itr: impl Iterator<Item = String>,
    number_of_different_chars: usize,
) -> Result<Option<i32>, Box<dyn Error>> {
    let lines = itr.collect::<Vec<_>>();
    let before_last = if lines.len() != 0 { lines.len() - 1 } else { 0 };

    let indexes = (0..before_last).map(|i| -> Result<Option<i32>, Box<dyn Error>> {
        let start = lines.as_slice().get(0..i + 1).ok_or("No start")?;
        let end = lines.as_slice().get(i + 1..).ok_or("No end")?;

        let mirror_equal = start
            .iter()
            .rev()
            .zip(end)
            .map(|(string1, string2)| {
                // get the number of different chars
                string1
                    .chars()
                    .zip(string2.chars())
                    .filter(|(c1, c2)| c1 != c2)
                    .count()
            })
            .sum::<usize>();

        Ok((mirror_equal == number_of_different_chars).then_some(i32::try_from(i)? + 1))
    });

    indexes.process_results(|mut itr| itr.find_map(identity))
}

fn get_mirror_vertically(
    itr: impl Iterator<Item = String>,
    number_of_different_chars: usize,
) -> Result<Option<i32>, Box<dyn Error>> {
    let vect_of_strings = itr.collect::<Vec<_>>();
    match vect_of_strings.get(0) {
        None => Ok(None),
        Some(s) => {
            let transposed = (0..s.len())
                .map(|i| {
                    vect_of_strings
                        .iter()
                        .map(|s| s.chars().nth(i).ok_or("No char".into()))
                        .collect::<Result<String, Box<dyn Error>>>()
                })
                .collect::<Result<Vec<_>, Box<dyn Error>>>()?;

            get_mirror_horizontally(transposed.into_iter(), number_of_different_chars)
        }
    }
}

fn solve_pattern(
    itr: impl Iterator<Item = String> + Clone,
    number_of_different_chars: usize,
) -> Result<i32, Box<dyn Error>> {
    let vertically =
        get_mirror_vertically(itr.clone(), number_of_different_chars)?.map_or(0, identity);
    let horizontally = get_mirror_horizontally(itr, number_of_different_chars)?.map_or(0, identity);

    Ok(vertically + horizontally * 100)
}

fn solve_pattern1(itr: impl Iterator<Item = String> + Clone) -> Result<i32, Box<dyn Error>> {
    solve_pattern(itr, 0)
}

fn solve_pattern2(itr: impl Iterator<Item = String> + Clone) -> Result<i32, Box<dyn Error>> {
    solve_pattern(itr, 1)
}

fn parse(itr: impl Iterator<Item = String>) -> Vec<Vec<String>> {
    itr.group_by(|s| s.is_empty())
        .into_iter()
        .filter(|(empty, _)| !empty)
        .map(|(_, group)| group.collect())
        .collect()
}

#[cfg(test)]
mod day13 {
    use itertools::Itertools;
    use std::{
        error::Error,
        fs::File,
        io::{BufRead, BufReader},
    };

    use crate::{
        get_mirror_horizontally, get_mirror_vertically, parse, solve_pattern1, solve_pattern2,
    };

    const EXAMPLE1: &str = "\
        #.##..##.\n\
        ..#.##.#.\n\
        ##......#\n\
        ##......#\n\
        ..#.##.#.\n\
        ..##..##.\n\
        #.#.##.#.";

    const EXAMPLE2: &str = "\
        #...##..#\n\
        #....#..#\n\
        ..##..###\n\
        #####.##.\n\
        #####.##.\n\
        ..##..###\n\
        #....#..#";

    fn both_examples() -> String {
        format!("{}\n\n{}", EXAMPLE1, EXAMPLE2)
    }

    #[test]
    fn test_mirror_vertically() -> Result<(), Box<dyn Error>> {
        let result = get_mirror_vertically(EXAMPLE1.lines().map(|s| s.to_string()), 0)?
            .ok_or("No result")?;
        assert_eq!(result, 5);
        Ok(())
    }

    #[test]
    fn test_mirror_horizontally() -> Result<(), Box<dyn Error>> {
        let result = get_mirror_horizontally(EXAMPLE2.lines().map(|s| s.to_string()), 0)?
            .ok_or("No result")?;
        assert_eq!(result, 4);
        Ok(())
    }

    #[test]
    fn test_solve_pattern1_example1() -> Result<(), Box<dyn Error>> {
        let result = solve_pattern1(EXAMPLE1.lines().map(|s| s.to_string()))?;
        assert_eq!(result, 5);
        Ok(())
    }

    #[test]
    fn test_solve_pattern1_example2() -> Result<(), Box<dyn Error>> {
        let result = solve_pattern1(EXAMPLE2.lines().map(|s| s.to_string()))?;
        assert_eq!(result, 400);
        Ok(())
    }

    #[test]
    fn test_solve_pattern1_both() -> Result<(), Box<dyn Error>> {
        let patterns = parse(both_examples().lines().map(|s| s.to_string()));
        let result = patterns
            .iter()
            .map(|p| solve_pattern1(p.iter().cloned()))
            .sum::<Result<i32, _>>()?;
        assert_eq!(result, 405);
        Ok(())
    }

    #[test]
    fn test_solve_pattern2_example1() -> Result<(), Box<dyn Error>> {
        let result = solve_pattern2(EXAMPLE1.lines().map(|s| s.to_string()))?;
        assert_eq!(result, 300);
        Ok(())
    }

    #[test]
    fn test_solve_pattern2_example2() -> Result<(), Box<dyn Error>> {
        let result = solve_pattern2(EXAMPLE2.lines().map(|s| s.to_string()))?;
        assert_eq!(result, 100);
        Ok(())
    }

    #[test]
    fn test_solve_pattern2_both() -> Result<(), Box<dyn Error>> {
        let patterns = parse(both_examples().lines().map(|s| s.to_string()));
        let result = patterns
            .iter()
            .map(|p| solve_pattern2(p.iter().cloned()))
            .sum::<Result<i32, _>>()?;
        assert_eq!(result, 400);
        Ok(())
    }

    #[test]
    fn test_solve1_input() -> Result<(), Box<dyn Error>> {
        let file = File::open("input")?;
        let reader = BufReader::new(file);
        let patterns = reader.lines().process_results(|itr| parse(itr))?;
        let result = patterns
            .iter()
            .map(|p| solve_pattern1(p.iter().cloned()))
            .sum::<Result<i32, _>>()?;
        assert_eq!(result, 35232);
        Ok(())
    }

    #[test]
    fn test_solve2_input() -> Result<(), Box<dyn Error>> {
        let file = File::open("input")?;
        let reader = BufReader::new(file);
        let patterns = reader.lines().process_results(|itr| parse(itr))?;
        let result = patterns
            .iter()
            .map(|p| solve_pattern2(p.iter().cloned()))
            .sum::<Result<i32, _>>()?;
        assert_eq!(result, 37982);
        Ok(())
    }
}
