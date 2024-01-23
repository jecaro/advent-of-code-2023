use itertools::Itertools;
use lib::get_args;
use std::io::stdin;
use std::{collections::HashSet, error::Error, io::BufRead, process::exit};

fn usage(prog_name: String) {
    println!("Usage: {} [-1|-2|-h]", prog_name);
    exit(0)
}

fn main() -> Result<(), Box<dyn Error>> {
    let (prog_name, args) = get_args()?;

    match args.get(0) {
        Some(arg) if arg == "-1" || arg == "-2" => {
            let factor = if arg == "-1" { 1 } else { 1_000_000 - 1 };
            let universe = stdin().lock().lines().process_results(|itr| parse(itr))??;
            let expanded = expand(&universe, factor)?;
            let result = solve(&expanded)?;

            println!("{}", result);
        }
        _ => usage(prog_name),
    }
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
struct Universe {
    width: i64,
    height: i64,
    galaxies: HashSet<(i64, i64)>,
}

fn expand(universe: &Universe, factor: i64) -> Result<Universe, Box<dyn Error>> {
    let lines_with_galaxies = universe
        .galaxies
        .iter()
        .map(|coordinates| coordinates.1)
        .collect::<HashSet<_>>();
    let lines_without_galaxies = (0..universe.height)
        .filter(|&y| !lines_with_galaxies.contains(&y))
        .collect::<HashSet<_>>();

    let columns_with_galaxies = universe
        .galaxies
        .iter()
        .map(|coordinates| coordinates.0)
        .collect::<HashSet<_>>();
    let columns_without_galaxies = (0..universe.width)
        .filter(|&x| !columns_with_galaxies.contains(&x))
        .collect::<HashSet<_>>();

    let galaxies = universe
        .galaxies
        .iter()
        .map(|(x, y)| -> Result<(i64, i64), Box<dyn Error>> {
            let x = x + factor
                * i64::try_from(columns_without_galaxies.iter().filter(|&c| c < x).count())?;
            let y = y + factor
                * i64::try_from(lines_without_galaxies.iter().filter(|&c| c < y).count())?;

            Ok((x, y))
        })
        .collect::<Result<HashSet<_>, _>>()?;

    let width = universe.width + i64::try_from(columns_without_galaxies.len())?;
    let height = universe.height + i64::try_from(lines_without_galaxies.len())?;
    Ok(Universe {
        width,
        height,
        galaxies,
    })
}

fn parse(itr: impl Iterator<Item = String>) -> Result<Universe, Box<dyn Error>> {
    let mut width = 0;
    let mut height = 0;
    let cells: HashSet<(i64, i64)> = itr
        .enumerate()
        .map(|(y, line)| -> Result<Vec<(i64, i64)>, Box<dyn Error>> {
            width = width.max(i64::try_from(line.len())?);
            height += 1;
            line.chars()
                .enumerate()
                .filter(|(_, c)| *c == '#')
                .map(|(x, _)| Ok((i64::try_from(x)?, i64::try_from(y)?)))
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect();

    Ok(Universe {
        width,
        height,
        galaxies: cells,
    })
}

fn solve(universe: &Universe) -> Result<i64, Box<dyn Error>> {
    universe
        .galaxies
        .iter()
        .combinations(2)
        .map(|pair| {
            let x = pair.get(0).ok_or("No first element")?;
            let y = pair.get(1).ok_or("No second element")?;
            Ok(distance(x, y))
        })
        .sum()
}

fn distance(a: &(i64, i64), b: &(i64, i64)) -> i64 {
    (a.0 - b.0).abs() + (a.1 - b.1).abs()
}

#[cfg(test)]
mod day11 {
    use std::{
        error::Error,
        fs::File,
        io::{BufRead, BufReader},
    };

    use itertools::Itertools;

    use crate::{expand, parse, solve, Universe};

    const EXAMPLE1: &str = "\
        ...#......\n\
        .......#..\n\
        #.........\n\
        ..........\n\
        ......#...\n\
        .#........\n\
        .........#\n\
        ..........\n\
        .......#..\n\
        #...#.....";

    const EXAMPLE1_EXPANDED: &str = "\
        ....#........\n\
        .........#...\n\
        #............\n\
        .............\n\
        .............\n\
        ........#....\n\
        .#...........\n\
        ............#\n\
        .............\n\
        .............\n\
        .........#...\n\
        #....#.......";

    fn to_string(universe: &Universe) -> String {
        (0..universe.height)
            .map(|y| {
                (0..universe.width)
                    .map(|x| {
                        if universe.galaxies.contains(&(x, y)) {
                            '#'
                        } else {
                            '.'
                        }
                    })
                    .chain("\n".chars())
                    .collect::<String>()
            })
            .collect::<String>()
            .trim()
            .to_string()
    }

    #[test]
    fn test_parse() -> Result<(), Box<dyn Error>> {
        let universe = parse(EXAMPLE1.lines().map(|s| s.to_string()))?;

        assert_eq!(EXAMPLE1, to_string(&universe));
        Ok(())
    }

    #[test]
    fn test_expand() -> Result<(), Box<dyn Error>> {
        let universe = parse(EXAMPLE1.lines().map(|s| s.to_string()))?;
        let expanded = expand(&universe, 1)?;

        assert_eq!(EXAMPLE1_EXPANDED, to_string(&expanded));
        Ok(())
    }

    #[test]
    fn test_solve1() -> Result<(), Box<dyn Error>> {
        let universe = parse(EXAMPLE1.lines().map(|s| s.to_string()))?;
        let expanded = expand(&universe, 1)?;

        assert_eq!(solve(&expanded)?, 374);
        Ok(())
    }

    #[test]
    fn test_solve2() -> Result<(), Box<dyn Error>> {
        let universe = parse(EXAMPLE1.lines().map(|s| s.to_string()))?;
        let expanded = expand(&universe, 10 - 1)?;

        assert_eq!(solve(&expanded)?, 1030);
        Ok(())
    }

    #[test]
    fn test_solve1_input() -> Result<(), Box<dyn Error>> {
        let file = File::open("input")?;
        let reader = BufReader::new(file);
        let universe = reader.lines().process_results(|itr| parse(itr))??;
        let expanded = expand(&universe, 1)?;

        assert_eq!(solve(&expanded)?, 9684228);
        Ok(())
    }

    #[test]
    fn test_solve2_input() -> Result<(), Box<dyn Error>> {
        let file = File::open("input")?;
        let reader = BufReader::new(file);
        let universe = reader.lines().process_results(|itr| parse(itr))??;
        let expanded = expand(&universe, 1_000_000 - 1)?;

        assert_eq!(solve(&expanded)?, 483844716556);
        Ok(())
    }
}
