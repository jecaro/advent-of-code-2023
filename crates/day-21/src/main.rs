use itertools::Itertools;
use lib::get_args;
use std::{
    collections::{HashMap, HashSet},
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
            let (grid, start) = stdin()
                .lock()
                .lines()
                .process_results(|lines| parse(lines))??;

            let result = if arg == "-1" {
                solve1(&grid, &start) as i64
            } else {
                solve2(&grid, &start)?
            };

            println!("{}", result);
        }
        _ => usage(prog_name),
    }

    Ok(())
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
struct Coordinates {
    x: i32,
    y: i32,
}

struct Grid {
    rocks: HashSet<Coordinates>,
    width: usize,
    height: usize,
}

fn valid1(grid: &Grid, c: &Coordinates) -> bool {
    c.x < grid.width as i32 && c.y < grid.height as i32 && !grid.rocks.contains(c)
}

fn valid2(grid: &Grid, c: &Coordinates) -> bool {
    let c_mod = Coordinates {
        x: i32::rem_euclid(c.x, grid.width as i32),
        y: i32::rem_euclid(c.y, grid.height as i32),
    };

    !grid.rocks.contains(&c_mod)
}

type ValidFn = fn(grid: &Grid, c: &Coordinates) -> bool;

fn advance(grid: &Grid, current: &HashSet<Coordinates>, valid: ValidFn) -> HashSet<Coordinates> {
    let mut next = HashSet::new();

    current.iter().for_each(|c| {
        vec![(0, 1), (0, -1), (1, 0), (-1, 0)]
            .iter()
            .for_each(|(dx, dy)| {
                let new_c = Coordinates {
                    x: c.x as i32 + dx,
                    y: c.y as i32 + dy,
                };

                if valid(grid, &new_c) {
                    next.insert(new_c);
                }
            })
    });

    next
}

fn advance_count(grid: &Grid, start: &Coordinates, count: i32, valid: ValidFn) -> usize {
    let mut current = HashSet::new();
    current.insert(start.clone());

    (0..count).for_each(|_| {
        current = advance(grid, &current, valid);
    });

    current.len()
}

fn solve1(grid: &Grid, start: &Coordinates) -> usize {
    advance_count(grid, start, 64, valid1)
}

// Solution found here:
// https://github.com/derailed-dash/Advent-of-Code/blob/master/src/AoC_2023/Dazbo's_Advent_of_Code_2023.ipynb
fn solve2(grid: &Grid, start: &Coordinates) -> Result<i64, Box<dyn Error>> {
    const NO_VALUE: &str = "No value";

    let mut current = HashSet::new();
    current.insert(start.clone());

    let mut steps = HashMap::new();
    let xs = (0..3).map(|i| 65 + 131 * i).collect::<Vec<_>>();
    let max_value = xs.iter().max().ok_or("No max value")?;

    (1..=*max_value).for_each(|i| {
        current = advance(grid, &current, valid2);

        if xs.contains(&i) {
            steps.insert(i, current.len());
        }
    });
    let p0 = *steps.get(xs.get(0).ok_or(NO_VALUE)?).ok_or(NO_VALUE)? as i64;
    let p1 = *steps.get(xs.get(1).ok_or(NO_VALUE)?).ok_or(NO_VALUE)? as i64;
    let p2 = *steps.get(xs.get(2).ok_or(NO_VALUE)?).ok_or(NO_VALUE)? as i64;

    let c = p0;
    let b = (4 * p1 - 3 * p0 - p2) / 2;
    let a = p1 - p0 - b;

    let x = (26501365 - grid.width as i64 / 2) / grid.width as i64;

    Ok(a * x * x + b * x + c)
}

fn parse(lines: impl Iterator<Item = String>) -> Result<(Grid, Coordinates), Box<dyn Error>> {
    let mut rocks = HashSet::new();
    let mut start = None;
    let mut width = 0;
    let mut height = 0;

    lines
        .enumerate()
        .try_for_each(|(y, line)| -> Result<(), Box<dyn Error>> {
            if width == 0 {
                width = line.len();
            } else if width != line.len() {
                Err::<_, Box<dyn Error>>("Inconsistent line length".into())?;
            }
            height += 1;

            line.chars()
                .enumerate()
                .try_for_each(|(x, c)| -> Result<(), Box<dyn Error>> {
                    let x = x as i32;
                    let y = y as i32;
                    match c {
                        '#' => {
                            rocks.insert(Coordinates { x, y });
                        }
                        'S' => {
                            if let Some(_) = start {
                                Err::<_, Box<dyn Error>>("Multiple starts found".into())?;
                            } else {
                                start = Some(Coordinates { x, y });
                            }
                        }
                        _ => (),
                    };
                    Ok(())
                })
        })?;

    Ok((
        Grid {
            rocks,
            width,
            height,
        },
        start.ok_or("No start found")?,
    ))
}

#[cfg(test)]
mod day21 {
    use std::{
        error::Error,
        fs::File,
        io::{BufRead, BufReader},
    };

    use itertools::Itertools;

    use crate::{advance_count, parse, solve1, solve2, valid1, valid2, Coordinates};

    const EXAMPLE: &str = "\
        ...........\n\
        .....###.#.\n\
        .###.##..#.\n\
        ..#.#...#..\n\
        ....#.#....\n\
        .##..S####.\n\
        .##..#...#.\n\
        .......##..\n\
        .##.#.####.\n\
        .##..##.##.\n\
        ...........";

    #[test]
    fn test_parse() -> Result<(), Box<dyn Error>> {
        let (grid, start) = parse(EXAMPLE.lines().map(|s| s.to_string()))?;

        assert_eq!(grid.width, 11);
        assert_eq!(grid.height, 11);
        assert_eq!(grid.rocks.len(), 40);
        assert_eq!(start, Coordinates { x: 5, y: 5 });

        Ok(())
    }

    #[test]
    fn test_advance_count_valid1() -> Result<(), Box<dyn Error>> {
        let (grid, start) = parse(EXAMPLE.lines().map(|s| s.to_string()))?;

        let result = advance_count(&grid, &start, 6, valid1);
        assert_eq!(result, 16);

        Ok(())
    }

    #[test]
    fn test_advance_count_valid2() -> Result<(), Box<dyn Error>> {
        let (grid, start) = parse(EXAMPLE.lines().map(|s| s.to_string()))?;

        let result = advance_count(&grid, &start, 6, valid2);
        assert_eq!(result, 16);

        let result = advance_count(&grid, &start, 10, valid2);
        assert_eq!(result, 50);

        let result = advance_count(&grid, &start, 50, valid2);
        assert_eq!(result, 1594);

        // those are too slow to run in tests

        // let result = advance_count1(&grid, &start, 100, valid2);
        // assert_eq!(result, 6536);

        // let result = advance_count(&grid, &start, 500, valid2);
        // assert_eq!(result, 167004);

        // let result = advance_count(&grid, &start, 1000, valid2);
        // assert_eq!(result, 668697);

        // let result = advance_count(&grid, &start, 5000, valid2);
        // assert_eq!(result, 16733044);

        Ok(())
    }

    #[test]
    fn test_solve1_input() -> Result<(), Box<dyn Error>> {
        let file = File::open("input")?;
        let reader = BufReader::new(file);
        let (grid, start) = reader.lines().process_results(|itr| parse(itr))??;

        let result = solve1(&grid, &start);
        assert_eq!(result, 3758);

        Ok(())
    }

    #[test]
    fn test_solve2_input() -> Result<(), Box<dyn Error>> {
        let file = File::open("input")?;
        let reader = BufReader::new(file);
        let (grid, start) = reader.lines().process_results(|itr| parse(itr))??;

        let result = solve2(&grid, &start)?;
        assert_eq!(result, 621494544278648);

        Ok(())
    }
}
