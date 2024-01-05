use itertools::{process_results, Itertools};
use lib::get_args;
use std::{
    error::Error,
    io::{stdin, BufRead},
    iter::once,
    process::exit,
    str::FromStr,
};

fn usage(prog_name: String) {
    println!("Usage: {} [-1|-2|-h]", prog_name);
    exit(0)
}

fn main() -> Result<(), Box<dyn Error>> {
    let (prog_name, args) = get_args()?;

    match args.get(0) {
        Some(arg) if arg == "-1" || arg == "-2" => {
            let directions = process_results(stdin().lock().lines(), |lines| {
                if arg == "-1" {
                    parse1(lines)
                } else {
                    parse2(lines)
                }
            })??;
            let result = num_points(&draw(&directions));

            println!("{}", result);
        }
        _ => usage(prog_name),
    }

    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl FromStr for Direction {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "U" => Ok(Direction::Up),
            "D" => Ok(Direction::Down),
            "L" => Ok(Direction::Left),
            "R" => Ok(Direction::Right),
            _ => Err(format!("Invalid direction: {}", s).into()),
        }
    }
}

fn parse1(itr: impl Iterator<Item = String>) -> Result<Vec<(Direction, i64)>, Box<dyn Error>> {
    itr.map(|s| {
        let parts = s.split_whitespace().collect::<Vec<_>>();

        let dir_str = parts.get(0).ok_or("Missing direction")?;
        let dir = dir_str.parse::<Direction>()?;

        let dist_str = parts.get(1).ok_or("Missing distance")?;
        let dist = dist_str.parse::<i64>()?;

        Ok((dir, dist))
    })
    .collect::<Result<Vec<_>, Box<dyn Error>>>()
}

fn parse_color(hex: &str) -> Result<(Direction, i64), Box<dyn Error>> {
    let hex_str = hex
        .strip_prefix("(#")
        .and_then(|s| s.strip_suffix(")"))
        .ok_or("Invalid hex")?;

    let hex_dist = hex_str.get(0..5).ok_or("Invalid distance")?;
    let dist = i64::from_str_radix(hex_dist, 16)?;

    let hex_dir = hex_str
        .get(5..)
        .and_then(|s| s.chars().next())
        .ok_or("Invalid direction")?;
    let dir = match hex_dir {
        '0' => Direction::Right,
        '1' => Direction::Down,
        '2' => Direction::Left,
        '3' => Direction::Up,
        _ => return Err("Invalid direction".into()),
    };

    Ok((dir, dist))
}

fn parse2(itr: impl Iterator<Item = String>) -> Result<Vec<(Direction, i64)>, Box<dyn Error>> {
    itr.map(|s| {
        let parts = s.split_whitespace().collect::<Vec<_>>();
        let hex_str = parts.get(2).ok_or("Missing hex")?.to_string();

        parse_color(&hex_str)
    })
    .collect::<Result<Vec<_>, Box<dyn Error>>>()
}

fn draw(directions: &[(Direction, i64)]) -> Vec<(i64, i64)> {
    let mut point = (0, 0);

    directions
        .iter()
        .map(|(dir, dist)| {
            point = match dir {
                Direction::Up => (point.0, point.1 - *dist),
                Direction::Down => (point.0, point.1 + *dist),
                Direction::Left => (point.0 - *dist, point.1),
                Direction::Right => (point.0 + *dist, point.1),
            };
            point
        })
        .collect::<Vec<_>>()
}

// compute the area of a polygon using the shoelace formula
// see https://en.wikipedia.org/wiki/Shoelace_formula
fn shoelace(points: &[(i64, i64)]) -> i64 {
    points.first().map_or(0, |first| {
        points
            .iter()
            .chain(once(first))
            .tuple_windows()
            .map(|(p1, p2)| p1.0 * p2.1 - p2.0 * p1.1)
            .sum::<i64>()
            .abs()
            / 2
    })
}

fn perimeter(points: &[(i64, i64)]) -> i64 {
    points.first().map_or(0, |first| {
        points
            .iter()
            .chain(once(first))
            .tuple_windows()
            .map(|(p1, p2)| (p1.0 - p2.0).abs() + (p1.1 - p2.1).abs())
            .sum::<i64>()
    })
}

// according to the pick theorem: https://en.wikipedia.org/wiki/Pick%27s_theorem
//
// A = i + b/2 - 1
//
// where:
// - A is the area of the polygon
// - i is the number of points inside the polygon
// - b is the number of points on the boundary of the polygon
//
// we already have A from the shoelace formula and b from the perimeter function
//
// we want to compute b + i
// so from the theorem:
// i = A - b/2 + 1
// and finally:
// b + i = A + 1 + b/2
fn num_points(points: &[(i64, i64)]) -> i64 {
    let area = shoelace(points);
    let perimeter = perimeter(points);

    area + 1 + perimeter / 2
}

#[cfg(test)]
mod day18 {
    use std::{
        fs::File,
        io::{BufRead, BufReader},
    };

    use itertools::process_results;

    use crate::{draw, num_points, parse1, parse2, parse_color, perimeter, Direction};

    const EXAMPLE1: &str = "\
        R 6 (#70c710)
        D 5 (#0dc571)
        L 2 (#5713f0)
        D 2 (#d2c081)
        R 2 (#59c680)
        D 2 (#411b91)
        L 5 (#8ceee2)
        U 2 (#caa173)
        L 1 (#1b58a2)
        U 2 (#caa171)
        R 2 (#7807d2)
        U 3 (#a77fa3)
        L 2 (#015232)
        U 2 (#7a21e3)";

    #[test]
    fn test_parse1() {
        let directions = parse1(EXAMPLE1.lines().map(|s| s.to_string())).unwrap();
        let result = draw(&directions);

        assert_eq!(result.last().unwrap(), &(0, 0));
    }

    #[test]
    fn test_perimeter() {
        let directions = parse1(EXAMPLE1.lines().map(|s| s.to_string())).unwrap();
        let points = draw(&directions);
        let perimeter = perimeter(&points);

        assert_eq!(perimeter, 38);
    }

    #[test]
    fn test_num_points_parse1() {
        let directions = parse1(EXAMPLE1.lines().map(|s| s.to_string())).unwrap();
        let points = draw(&directions);
        let area = num_points(&points);

        assert_eq!(area, 62);
    }

    #[test]
    fn test_num_points_parse2() {
        let directions = parse2(EXAMPLE1.lines().map(|s| s.to_string())).unwrap();
        let points = draw(&directions);
        let area = num_points(&points);

        assert_eq!(area, 952408144115);
    }

    #[test]
    fn test_num_points_parse1_input() {
        let file = File::open("input").unwrap();
        let reader = BufReader::new(file);
        let directions = process_results(reader.lines(), |itr| parse1(itr))
            .unwrap()
            .unwrap();
        let points = draw(&directions);
        let area = num_points(&points);

        assert_eq!(area, 47527);
    }

    #[test]
    fn test_num_points_parse2_input() {
        let file = File::open("input").unwrap();
        let reader = BufReader::new(file);
        let directions = process_results(reader.lines(), |itr| parse2(itr))
            .unwrap()
            .unwrap();
        let points = draw(&directions);
        let area = num_points(&points);

        assert_eq!(area, 52240187443190);
    }

    #[test]
    fn test_parse_color() {
        assert_eq!(
            parse_color("(#70c710)").unwrap(),
            (Direction::Right, 461937)
        );
        assert_eq!(parse_color("(#0dc571)").unwrap(), (Direction::Down, 56407));
        assert_eq!(
            parse_color("(#5713f0)").unwrap(),
            (Direction::Right, 356671)
        );
        assert_eq!(parse_color("(#d2c081)").unwrap(), (Direction::Down, 863240));
        assert_eq!(
            parse_color("(#59c680)").unwrap(),
            (Direction::Right, 367720)
        );
        assert_eq!(parse_color("(#411b91)").unwrap(), (Direction::Down, 266681));
        assert_eq!(parse_color("(#8ceee2)").unwrap(), (Direction::Left, 577262));
        assert_eq!(parse_color("(#caa173)").unwrap(), (Direction::Up, 829975));
        assert_eq!(parse_color("(#1b58a2)").unwrap(), (Direction::Left, 112010));
        assert_eq!(parse_color("(#caa171)").unwrap(), (Direction::Down, 829975));
        assert_eq!(parse_color("(#7807d2)").unwrap(), (Direction::Left, 491645));
        assert_eq!(parse_color("(#a77fa3)").unwrap(), (Direction::Up, 686074));
        assert_eq!(parse_color("(#015232)").unwrap(), (Direction::Left, 5411));
        assert_eq!(parse_color("(#7a21e3)").unwrap(), (Direction::Up, 500254));
    }
}
