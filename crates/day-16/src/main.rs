use itertools::Itertools;
use lib::get_args;
use std::{
    collections::HashSet,
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
            let grid = stdin()
                .lock()
                .lines()
                .process_results(|lines| parse(lines))??;
            let result = if arg == "-1" {
                solve1(&grid)
            } else {
                solve2(&grid)
            }?;

            println!("{}", result);
        }
        _ => usage(prog_name),
    }
    Ok(())
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Debug, PartialEq, Eq)]
enum Contraption {
    Empty,
    VerticalSplitter,
    HorizontalSplitter,
    MirrorSlash,
    MirrorBackslash,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, PartialEq, Eq)]
struct Grid {
    width: i32,
    height: i32,
    layout: Vec<Vec<Contraption>>,
}

fn parse(itr: impl Iterator<Item = String>) -> Result<Grid, Box<dyn Error>> {
    let mut width = 0;

    let layout = itr
        .map(|line| {
            width = if width == 0 {
                Ok(line.len() as i32)
            } else if width != (line.len() as i32) {
                Err(format!("Invalid line length: {}", line.len()))
            } else {
                Ok(width)
            }?;

            line.chars()
                .map(|c| -> Result<_, Box<dyn Error>> {
                    match c {
                        '.' => Ok(Contraption::Empty),
                        '|' => Ok(Contraption::VerticalSplitter),
                        '-' => Ok(Contraption::HorizontalSplitter),
                        '/' => Ok(Contraption::MirrorSlash),
                        '\\' => Ok(Contraption::MirrorBackslash),
                        _ => Err(format!("Invalid character: {}", c).into()),
                    }
                })
                .collect()
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Grid {
        width,
        height: layout.len() as i32,
        layout,
    })
}

fn up(point: &Point) -> Point {
    Point {
        x: point.x,
        y: point.y - 1,
    }
}

fn down(point: &Point) -> Point {
    Point {
        x: point.x,
        y: point.y + 1,
    }
}

fn left(point: &Point) -> Point {
    Point {
        x: point.x - 1,
        y: point.y,
    }
}

fn right(point: &Point) -> Point {
    Point {
        x: point.x + 1,
        y: point.y,
    }
}

fn next(point: &Point, direction: &Direction) -> Point {
    match direction {
        Direction::Up => up(point),
        Direction::Down => down(point),
        Direction::Left => left(point),
        Direction::Right => right(point),
    }
}

fn solve1(grid: &Grid) -> Result<i32, Box<dyn Error>> {
    solve(grid, (Point { x: 0, y: 0 }, Direction::Right))
}

fn solve2(grid: &Grid) -> Result<i32, Box<dyn Error>> {
    let xs = 0..grid.width;
    let last_x = if grid.width > 0 {
        Ok(grid.width - 1)
    } else {
        Err("Invalid width")
    }?;

    let ys = 0..grid.height;
    let last_y = if grid.height > 0 {
        Ok(grid.height - 1)
    } else {
        Err("Invalid height")
    }?;

    let positions = xs
        .clone()
        .map(|x| (Point { x, y: 0 }, Direction::Down))
        .chain(xs.map(|x| (Point { x, y: last_y }, Direction::Up)))
        .chain(ys.clone().map(|y| (Point { x: 0, y }, Direction::Right)))
        .chain(ys.map(|y| (Point { x: last_x, y }, Direction::Left)));

    positions
        .map(|point_and_direction| solve(grid, point_and_direction))
        .process_results(|itr| itr.max())?
        .ok_or("No solution".into())
}

fn solve(grid: &Grid, start: (Point, Direction)) -> Result<i32, Box<dyn Error>> {
    let mut visited: Vec<Vec<HashSet<Direction>>> =
        vec![vec![HashSet::new(); grid.width as usize]; grid.height as usize];
    let mut stack = vec![start];

    while let Some((point, direction)) = stack.pop() {
        // discard out of bound points
        if point.x < 0 || point.x >= grid.width || point.y < 0 || point.y >= grid.height {
            continue;
        }

        let cell_visited = visited
            .get_mut(point.y as usize)
            .and_then(|row| row.get_mut(point.x as usize));

        // skip visited cells
        if cell_visited
            .as_ref()
            .map_or(false, |directions| directions.contains(&direction))
        {
            continue;
        }

        // mark cell as visited
        cell_visited.map(|directions| directions.insert(direction));

        // get the next moves
        grid.layout
            .get(point.y as usize)
            .and_then(|row| row.get(point.x as usize))
            .map(|contraption| match contraption {
                Contraption::Empty => stack.push((next(&point, &direction), direction)),
                Contraption::VerticalSplitter => match direction {
                    Direction::Up | Direction::Down => {
                        stack.push((next(&point, &direction), direction));
                    }
                    Direction::Left | Direction::Right => {
                        stack.push((up(&point), Direction::Up));
                        stack.push((down(&point), Direction::Down));
                    }
                },
                Contraption::HorizontalSplitter => match direction {
                    Direction::Up | Direction::Down => {
                        stack.push((left(&point), Direction::Left));
                        stack.push((right(&point), Direction::Right));
                    }
                    Direction::Left | Direction::Right => {
                        stack.push((next(&point, &direction), direction));
                    }
                },
                Contraption::MirrorSlash => match direction {
                    Direction::Up => {
                        stack.push((right(&point), Direction::Right));
                    }
                    Direction::Down => {
                        stack.push((left(&point), Direction::Left));
                    }
                    Direction::Left => {
                        stack.push((down(&point), Direction::Down));
                    }
                    Direction::Right => {
                        stack.push((up(&point), Direction::Up));
                    }
                },
                Contraption::MirrorBackslash => match direction {
                    Direction::Up => {
                        stack.push((left(&point), Direction::Left));
                    }
                    Direction::Down => {
                        stack.push((right(&point), Direction::Right));
                    }
                    Direction::Left => {
                        stack.push((up(&point), Direction::Up));
                    }
                    Direction::Right => {
                        stack.push((down(&point), Direction::Down));
                    }
                },
            });
    }

    Ok(visited
        .iter()
        .flatten()
        .filter(|&visited| !visited.is_empty())
        .count() as i32)
}

#[cfg(test)]
mod day16 {

    use std::{
        error::Error,
        fs::File,
        io::{BufRead, BufReader},
    };

    use itertools::Itertools;

    use crate::{parse, solve1, solve2, Contraption, Grid};

    const EXAMPLE: &str = r".|...\....
|.-.\.....
.....|-...
........|.
..........
.........\
..../.\\..
.-.-/..|..
.|....-|.\
..//.|....";

    fn example_grid() -> Grid {
        Grid {
            width: 10,
            height: 10,
            layout: vec![
                vec![
                    Contraption::Empty,
                    Contraption::VerticalSplitter,
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::MirrorBackslash,
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::Empty,
                ],
                vec![
                    Contraption::VerticalSplitter,
                    Contraption::Empty,
                    Contraption::HorizontalSplitter,
                    Contraption::Empty,
                    Contraption::MirrorBackslash,
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::Empty,
                ],
                vec![
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::VerticalSplitter,
                    Contraption::HorizontalSplitter,
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::Empty,
                ],
                vec![
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::VerticalSplitter,
                    Contraption::Empty,
                ],
                vec![
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::Empty,
                ],
                vec![
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::MirrorBackslash,
                ],
                vec![
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::MirrorSlash,
                    Contraption::Empty,
                    Contraption::MirrorBackslash,
                    Contraption::MirrorBackslash,
                    Contraption::Empty,
                    Contraption::Empty,
                ],
                vec![
                    Contraption::Empty,
                    Contraption::HorizontalSplitter,
                    Contraption::Empty,
                    Contraption::HorizontalSplitter,
                    Contraption::MirrorSlash,
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::VerticalSplitter,
                    Contraption::Empty,
                    Contraption::Empty,
                ],
                vec![
                    Contraption::Empty,
                    Contraption::VerticalSplitter,
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::HorizontalSplitter,
                    Contraption::VerticalSplitter,
                    Contraption::Empty,
                    Contraption::MirrorBackslash,
                ],
                vec![
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::MirrorSlash,
                    Contraption::MirrorSlash,
                    Contraption::Empty,
                    Contraption::VerticalSplitter,
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::Empty,
                    Contraption::Empty,
                ],
            ],
        }
    }

    #[test]
    fn test_parse() -> Result<(), Box<dyn Error>> {
        assert_eq!(
            parse(EXAMPLE.lines().map(|s| s.to_string()))?,
            example_grid()
        );
        Ok(())
    }

    #[test]
    fn test_solve1_example() -> Result<(), Box<dyn Error>> {
        assert_eq!(solve1(&example_grid())?, 46);
        Ok(())
    }

    #[test]
    fn test_solve1_input() -> Result<(), Box<dyn Error>> {
        let file = File::open("input")?;
        let reader = BufReader::new(file);
        let grid = reader.lines().process_results(|itr| parse(itr))??;
        let result = solve1(&grid)?;
        assert_eq!(result, 7046);
        Ok(())
    }

    #[test]
    fn test_solve2_example() -> Result<(), Box<dyn Error>> {
        assert_eq!(solve2(&example_grid())?, 51);
        Ok(())
    }

    #[test]
    fn test_solve2_input() -> Result<(), Box<dyn Error>> {
        let file = File::open("input")?;
        let reader = BufReader::new(file);
        let grid = reader.lines().process_results(|itr| parse(itr))??;
        let result = solve2(&grid)?;
        assert_eq!(result, 7313);
        Ok(())
    }
}
