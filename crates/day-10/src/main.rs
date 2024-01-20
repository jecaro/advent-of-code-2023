use itertools::Itertools;
use lib::get_args;
use std::{
    cell::RefCell,
    collections::HashSet,
    error::Error,
    io::{stdin, BufRead},
    process::exit,
    rc::Rc,
};

fn usage(prog_name: String) {
    println!("Usage: {} [-1|-2|-h]", prog_name);
    exit(0)
}

fn main() -> Result<(), Box<dyn Error>> {
    let (prog_name, args) = get_args()?;

    match args.get(0) {
        Some(arg) if arg == "-1" || arg == "-2" => {
            let solve = if arg == "-1" { solve1 } else { solve2 };
            let maze = stdin()
                .lock()
                .lines()
                .process_results(|itr| parse_maze(itr))??;
            let result = solve(maze)?;

            println!("{}", result);
        }
        _ => usage(prog_name),
    }
    Ok(())
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Tile {
    NorthSouth,
    EastWest,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
    Start,
    Ground,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Direction {
    North,
    East,
    South,
    West,
}

type Maze = Vec<Vec<Tile>>;

#[derive(Debug, PartialEq, Eq, Clone)]
struct Tree {
    position: Coordinates,
    children: Vec<Rc<RefCell<Tree>>>,
}

// Tree is a recursive structure. The default Drop implementation is recursive too. It can lead to
// stack overflows. So we need an iterative implementation.
impl Drop for Tree {
    fn drop(&mut self) {
        let mut stack: Vec<Rc<RefCell<Tree>>> = Vec::new();
        for child in self.children.clone() {
            stack.push(child);
        }
        while let Some(current) = stack.pop() {
            for child in current.borrow().children.iter() {
                stack.push(child.clone());
            }
            current.borrow_mut().children.clear();
        }
    }
}

type Coordinates = (i32, i32);

fn parse_char(c: char) -> Result<Tile, Box<dyn Error>> {
    match c {
        '|' => Ok(Tile::NorthSouth),
        '-' => Ok(Tile::EastWest),
        'L' => Ok(Tile::NorthEast),
        'J' => Ok(Tile::NorthWest),
        'F' => Ok(Tile::SouthEast),
        '7' => Ok(Tile::SouthWest),
        'S' => Ok(Tile::Start),
        '.' => Ok(Tile::Ground),
        _ => Err(format!("Invalid character: {}", c).into()),
    }
}

fn parse_maze(itr: impl Iterator<Item = String>) -> Result<Maze, Box<dyn Error>> {
    itr.map(|line| line.chars().map(|c| parse_char(c)).collect())
        .collect()
}

fn offset(direction: &Direction) -> Coordinates {
    match direction {
        Direction::North => (0, -1),
        Direction::East => (1, 0),
        Direction::South => (0, 1),
        Direction::West => (-1, 0),
    }
}

fn all_direction() -> Vec<Direction> {
    vec![
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ]
}

fn valid_to(maze: &Maze, position: Coordinates, direction: Direction) -> Option<Coordinates> {
    let offset = offset(&direction);
    let new_coordinates = (position.0 + offset.0, position.1 + offset.1);

    let destination_tile = maze
        .get(new_coordinates.1 as usize)
        .and_then(|row| row.get(new_coordinates.0 as usize))?;

    let valid = (direction == Direction::North
        && (*destination_tile == Tile::NorthSouth
            || *destination_tile == Tile::SouthEast
            || *destination_tile == Tile::SouthWest))
        || (direction == Direction::South
            && (*destination_tile == Tile::NorthSouth
                || *destination_tile == Tile::NorthEast
                || *destination_tile == Tile::NorthWest))
        || (direction == Direction::East
            && (*destination_tile == Tile::EastWest
                || *destination_tile == Tile::NorthWest
                || *destination_tile == Tile::SouthWest))
        || (direction == Direction::West
            && (*destination_tile == Tile::EastWest
                || *destination_tile == Tile::NorthEast
                || *destination_tile == Tile::SouthEast));

    valid.then(|| new_coordinates)
}

fn valid_from(
    maze: &Maze,
    position: Coordinates,
    direction: Direction,
) -> Result<bool, Box<dyn Error>> {
    let tile = maze
        .get(position.1 as usize)
        .and_then(|row| row.get(position.0 as usize))
        .ok_or("Invalid coordinates")?;

    let valid = match tile {
        Tile::NorthSouth => direction == Direction::North || direction == Direction::South,
        Tile::EastWest => direction == Direction::East || direction == Direction::West,
        Tile::NorthEast => direction == Direction::North || direction == Direction::East,
        Tile::NorthWest => direction == Direction::North || direction == Direction::West,
        Tile::SouthEast => direction == Direction::South || direction == Direction::East,
        Tile::SouthWest => direction == Direction::South || direction == Direction::West,
        Tile::Start => true,
        Tile::Ground => false,
    };

    Ok(valid)
}

fn opposite(direction: Direction) -> Direction {
    match direction {
        Direction::North => Direction::South,
        Direction::East => Direction::West,
        Direction::South => Direction::North,
        Direction::West => Direction::East,
    }
}

fn next(
    maze: &Maze,
    last_direction: Option<Direction>,
    position: Coordinates,
) -> Result<Vec<(Direction, Coordinates)>, Box<dyn Error>> {
    all_direction()
        .iter()
        .filter(|direction| Some(opposite(**direction)) != last_direction)
        .map(|direction| -> Result<_, Box<dyn Error>> {
            let from = valid_from(maze, position, *direction)?;
            let to = valid_to(maze, position, *direction);
            let direction_and_to = to.map(|to| (*direction, to));
            Ok(if from { direction_and_to } else { None })
        })
        .process_results(|itr| itr.filter_map(|r| r).collect::<Vec<_>>())
}

fn create_tree(maze: &Maze) -> Result<Rc<RefCell<Tree>>, Box<dyn Error>> {
    let start = maze
        .iter()
        .enumerate()
        .find_map(|(y, row)| {
            row.iter().enumerate().find_map(|(x, tile)| {
                if *tile == Tile::Start {
                    Some((x as i32, y as i32))
                } else {
                    None
                }
            })
        })
        .ok_or("No start tile found")?;

    let mut stack: Vec<(Option<Direction>, Rc<RefCell<Tree>>)> = Vec::new();
    let tree = Rc::new(RefCell::new(Tree {
        position: start,
        children: Vec::new(),
    }));
    stack.push((None, tree.clone()));

    while let Some((last_direction, current)) = stack.pop() {
        let next_coordinates = next(maze, last_direction, current.borrow().position)?;

        for (direction, coordinate) in next_coordinates {
            let child = Rc::new(RefCell::new(Tree {
                position: coordinate,
                children: Vec::new(),
            }));

            current.borrow_mut().children.push(child.clone());
            stack.push((Some(direction), child));
            break;
        }
    }

    Ok(tree)
}

fn paths(tree: Rc<RefCell<Tree>>) -> Vec<Vec<Rc<RefCell<Tree>>>> {
    let mut stack = vec![vec![tree.clone()]];
    let mut paths = Vec::new();

    while let Some(mut path) = stack.pop() {
        if let Some(last) = path.last().cloned() {
            if last.borrow().children.is_empty() {
                paths.push(path);
            } else {
                last.borrow().children.iter().for_each(|child| {
                    path.push(child.clone());
                    stack.push(path.clone());
                });
            }
        }
    }

    paths
}

fn longuest_path(tree: Rc<RefCell<Tree>>) -> Result<Vec<Rc<RefCell<Tree>>>, Box<dyn Error>> {
    paths(tree)
        .into_iter()
        .max_by_key(|path| path.len())
        .ok_or("No path found".into())
}

fn solve1(maze: Maze) -> Result<u32, Box<dyn Error>> {
    let tree = create_tree(&maze)?;

    let longuest_path = longuest_path(tree.clone())?;

    Ok(longuest_path.len() as u32 / 2)
}

fn get_start_replacement(path: &[Rc<RefCell<Tree>>]) -> Result<Tile, Box<dyn Error>> {
    let first_coord = path.first().ok_or("Invalid path")?.borrow().position;
    let second_coord = path.get(1).ok_or("Invalid path")?.borrow().position;
    let last_coord = path.last().ok_or("Invalid path")?.borrow().position;
    let first_second = (
        second_coord.0 - first_coord.0,
        second_coord.1 - first_coord.1,
    );
    let first_last = (last_coord.0 - first_coord.0, last_coord.1 - first_coord.1);

    match (first_second, first_last) {
        ((-1, 0), (1, 0)) => Ok(Tile::EastWest),
        ((1, 0), (-1, 0)) => Ok(Tile::EastWest),

        ((0, -1), (0, 1)) => Ok(Tile::NorthEast),
        ((0, 1), (0, -1)) => Ok(Tile::NorthEast),

        ((0, -1), (-1, 0)) => Ok(Tile::NorthWest),
        ((0, -1), (1, 0)) => Ok(Tile::NorthEast),
        ((0, 1), (-1, 0)) => Ok(Tile::SouthWest),
        ((0, 1), (1, 0)) => Ok(Tile::SouthEast),

        ((-1, 0), (0, -1)) => Ok(Tile::NorthWest),
        ((-1, 0), (0, 1)) => Ok(Tile::NorthEast),
        ((1, 0), (0, -1)) => Ok(Tile::SouthWest),
        ((1, 0), (0, 1)) => Ok(Tile::SouthEast),

        _ => Err("Invalid first and last tiles".into()),
    }
}

fn solve2(maze: Maze) -> Result<u32, Box<dyn Error>> {
    let tree = create_tree(&maze)?;

    let path = longuest_path(tree.clone())?;

    // To make it easier to handle the start tile, we replace it by the proper tile
    let new_start = get_start_replacement(&path)?;
    let maze = maze.iter().map(|row| {
        row.iter().map(|tile| {
            if *tile == Tile::Start {
                new_start
            } else {
                *tile
            }
        })
    });

    // put all the coordinates in a set
    let coordinates: HashSet<(i32, i32)> =
        HashSet::from_iter(path.iter().map(|node| node.borrow().position));

    Ok(maze
        // scan all the lines
        .enumerate()
        .map(|(y, line)| -> u32 {
            // fold the chars
            line.enumerate()
                .fold(
                    // in the state we store:
                    // - the number of tiles inside the path
                    // - if we are inside the path
                    // - the tile starting a wall NorthEast or SouthEast
                    (0, false, None),
                    |(count, inside, first_tile): (u32, bool, Option<Tile>), (x, tile)| {
                        // we are on a wall
                        if coordinates.contains(&(x as i32, y as i32)) {
                            match (first_tile, tile) {
                                (None, Tile::NorthSouth) => (count, !inside, None),

                                (None, Tile::NorthEast) => (count, inside, Some(Tile::NorthEast)),
                                (None, Tile::SouthEast) => (count, inside, Some(Tile::SouthEast)),

                                (Some(Tile::NorthEast), Tile::SouthWest) => (count, !inside, None),
                                (Some(Tile::NorthEast), Tile::NorthWest) => (count, inside, None),

                                (Some(Tile::SouthEast), Tile::NorthWest) => (count, !inside, None),
                                (Some(Tile::SouthEast), Tile::SouthWest) => (count, inside, None),

                                _ => (count, inside, first_tile),
                            }
                        // not on a wall
                        } else {
                            (if inside { count + 1 } else { count }, inside, None)
                        }
                    },
                )
                .0
        })
        .sum())
}

#[cfg(test)]
mod day10 {
    use std::{
        error::Error,
        fs::File,
        io::{BufRead, BufReader},
    };

    use itertools::Itertools;

    use crate::{parse_maze, solve1, solve2, Maze, Tile};

    const EXAMPLE1: &str = "\
        -L|F7\n\
        7S-7|\n\
        L|7||\n\
        -L-J|\n\
        L|-JF";

    fn example1() -> Maze {
        vec![
            vec![
                Tile::EastWest,
                Tile::NorthEast,
                Tile::NorthSouth,
                Tile::SouthEast,
                Tile::SouthWest,
            ],
            vec![
                Tile::SouthWest,
                Tile::Start,
                Tile::EastWest,
                Tile::SouthWest,
                Tile::NorthSouth,
            ],
            vec![
                Tile::NorthEast,
                Tile::NorthSouth,
                Tile::SouthWest,
                Tile::NorthSouth,
                Tile::NorthSouth,
            ],
            vec![
                Tile::EastWest,
                Tile::NorthEast,
                Tile::EastWest,
                Tile::NorthWest,
                Tile::NorthSouth,
            ],
            vec![
                Tile::NorthEast,
                Tile::NorthSouth,
                Tile::EastWest,
                Tile::NorthWest,
                Tile::SouthEast,
            ],
        ]
    }

    const EXAMPLE2: &str = "\
        7-F7-\n\
        .FJ|7\n\
        SJLL7\n\
        |F--J\n\
        LJ.LJ";

    fn example2() -> Maze {
        vec![
            vec![
                Tile::SouthWest,
                Tile::EastWest,
                Tile::SouthEast,
                Tile::SouthWest,
                Tile::EastWest,
            ],
            vec![
                Tile::Ground,
                Tile::SouthEast,
                Tile::NorthWest,
                Tile::NorthSouth,
                Tile::SouthWest,
            ],
            vec![
                Tile::Start,
                Tile::NorthWest,
                Tile::NorthEast,
                Tile::NorthEast,
                Tile::SouthWest,
            ],
            vec![
                Tile::NorthSouth,
                Tile::SouthEast,
                Tile::EastWest,
                Tile::EastWest,
                Tile::NorthWest,
            ],
            vec![
                Tile::NorthEast,
                Tile::NorthWest,
                Tile::Ground,
                Tile::NorthEast,
                Tile::NorthWest,
            ],
        ]
    }

    const EXAMPLE3: &str = "\
        ...........\n\
        .S-------7.\n\
        .|F-----7|.\n\
        .||.....||.\n\
        .||.....||.\n\
        .|L-7.F-J|.\n\
        .|..|.|..|.\n\
        .L--J.L--J.\n\
        ...........";

    const EXAMPLE4: &str = "\
        .F----7F7F7F7F-7....\n\
        .|F--7||||||||FJ....\n\
        .||.FJ||||||||L7....\n\
        FJL7L7LJLJ||LJ.L-7..\n\
        L--J.L7...LJS7F-7L7.\n\
        ....F-J..F7FJ|L7L7L7\n\
        ....L7.F7||L7|.L7L7|\n\
        .....|FJLJ|FJ|F7|.LJ\n\
        ....FJL-7.||.||||...\n\
        ....L---J.LJ.LJLJ...";

    const EXAMPLE5: &str = "\
        FF7FSF7F7F7F7F7F---7\n\
        L|LJ||||||||||||F--J\n\
        FL-7LJLJ||||||LJL-77\n\
        F--JF--7||LJLJ7F7FJ-\n\
        L---JF-JLJ.||-FJLJJ7\n\
        |F|F-JF---7F7-L7L|7|\n\
        |FFJF7L7F-JF7|JL---7\n\
        7-L-JL7||F7|L7F-7F7|\n\
        L.L7LFJ|||||FJL7||LJ\n\
        L7JLJL-JLJLJL--JLJ.L";

    #[test]
    fn test_parse_maze() -> Result<(), Box<dyn Error>> {
        let result = parse_maze(EXAMPLE1.lines().map(|s| s.to_string()))?;
        assert_eq!(result, example1());

        let result = parse_maze(EXAMPLE2.lines().map(|s| s.to_string()))?;
        assert_eq!(result, example2());
        Ok(())
    }

    #[test]
    fn test_solve1_example1() -> Result<(), Box<dyn Error>> {
        assert_eq!(solve1(example1())?, 4);
        Ok(())
    }

    #[test]
    fn test_solve1_example2() -> Result<(), Box<dyn Error>> {
        assert_eq!(solve1(example2())?, 8);
        Ok(())
    }

    #[test]
    fn test_solve2_example3() -> Result<(), Box<dyn Error>> {
        let maze = parse_maze(EXAMPLE3.lines().map(|s| s.to_string()))?;
        assert_eq!(solve2(maze)?, 4);
        Ok(())
    }

    #[test]
    fn test_solve2_example4() -> Result<(), Box<dyn Error>> {
        let maze = parse_maze(EXAMPLE4.lines().map(|s| s.to_string()))?;
        assert_eq!(solve2(maze)?, 8);
        Ok(())
    }

    #[test]
    fn test_solve2_example5() -> Result<(), Box<dyn Error>> {
        let maze = parse_maze(EXAMPLE5.lines().map(|s| s.to_string()))?;
        assert_eq!(solve2(maze)?, 10);
        Ok(())
    }

    #[test]
    fn test_solve1_input() -> Result<(), Box<dyn Error>> {
        let file = File::open("input")?;
        let reader = BufReader::new(file);
        let maze = reader.lines().process_results(|itr| parse_maze(itr))??;

        assert_eq!(solve1(maze)?, 6890);
        Ok(())
    }

    #[test]
    fn test_solve2_input() -> Result<(), Box<dyn Error>> {
        let file = File::open("input")?;
        let reader = BufReader::new(file);
        let maze = reader.lines().process_results(|itr| parse_maze(itr))??;

        assert_eq!(solve2(maze)?, 453);
        Ok(())
    }
}
