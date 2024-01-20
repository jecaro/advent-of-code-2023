use itertools::Itertools;
use lib::get_args;
use std::{
    collections::{HashMap, HashSet, VecDeque},
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
            let result = if arg == "-1" {
                let map = stdin()
                    .lock()
                    .lines()
                    .process_results(|lines| parse(lines))??;

                solve1(&map)?
            } else {
                let map = stdin()
                    .lock()
                    .lines()
                    .process_results(|lines| parse(lines.map(|line| remove_slopes(&line))))??;

                solve2(&map)?
            };

            println!("{}", result);
        }
        _ => usage(prog_name),
    }

    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tile {
    Path,
    Forest,
    SlopeNorth,
    SlopeSouth,
    SlopeEast,
    SlopeWest,
}

impl TryFrom<char> for Tile {
    type Error = Box<dyn Error>;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '.' => Ok(Tile::Path),
            '#' => Ok(Tile::Forest),
            '>' => Ok(Tile::SlopeEast),
            '<' => Ok(Tile::SlopeWest),
            '^' => Ok(Tile::SlopeNorth),
            'v' => Ok(Tile::SlopeSouth),
            _ => Err(format!("Invalid tile: {}", c).into()),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Map {
    tiles: Vec<Vec<Tile>>,
    width: usize,
    height: usize,
}

fn parse(itr: impl Iterator<Item = String>) -> Result<Map, Box<dyn Error>> {
    let mut height = 0;
    let mut width = 0;

    let tiles = itr
        .map(|line| {
            height += 1;

            if width == 0 {
                width = line.len();
            } else if width != line.len() {
                Err::<_, Box<dyn Error>>(format!("Invalid line length: {}", line.len()).into())?;
            }

            line.chars()
                .map(|c| Tile::try_from(c))
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Map {
        tiles,
        width,
        height,
    })
}

fn remove_slopes(str: &str) -> String {
    str.chars()
        .map(|c| match c {
            '>' => '.',
            '<' => '.',
            '^' => '.',
            'v' => '.',
            _ => c,
        })
        .collect::<String>()
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Step {
    current: Position,
    visited: HashSet<Position>,
}

fn top(Position { x, y }: &Position) -> Position {
    Position { x: *x, y: y - 1 }
}

fn bottom(Position { x, y }: &Position) -> Position {
    Position { x: *x, y: y + 1 }
}

fn left(Position { x, y }: &Position) -> Position {
    Position { x: x - 1, y: *y }
}

fn right(Position { x, y }: &Position) -> Position {
    Position { x: x + 1, y: *y }
}

fn get_adjacent_positions(map: &Map, from: &Position) -> Result<Vec<Position>, Box<dyn Error>> {
    let from_tile = map_get(map, &from).ok_or("Invalid from position")?;

    let next_possible_positions = match from_tile {
        Tile::SlopeNorth => vec![top(&from)],
        Tile::SlopeSouth => vec![bottom(&from)],
        Tile::SlopeEast => vec![right(&from)],
        Tile::SlopeWest => vec![left(&from)],
        _ => vec![top(&from), bottom(&from), left(&from), right(&from)],
    };

    Ok(next_possible_positions
        .into_iter()
        .filter(|next| on_map_and_not_forest(map, next))
        .collect())
}

fn map_get(map: &Map, Position { x, y }: &Position) -> Option<Tile> {
    (*x >= 0 && *y >= 0)
        .then_some(
            map.tiles
                .get(*y as usize)
                .and_then(|row| row.get(*x as usize))
                .copied(),
        )
        .flatten()
}

fn on_map_and_not_forest(map: &Map, position: &Position) -> bool {
    let tile = map_get(map, position);

    tile.is_some() && tile != Some(Tile::Forest)
}

type Graph = HashMap<Position, Vec<(Position, usize)>>;

fn solve2(map: &Map) -> Result<usize, Box<dyn Error>> {
    // create a compressed graph

    // find all vertices
    let start = Position { x: 1, y: 0 };
    let end = Position {
        x: map.width as i32 - 2,
        y: map.height as i32 - 1,
    };
    let vertices = (0..map.width)
        .map(|x| {
            let start = &start;
            let end = &end;
            (0..map.height).filter_map(move |y| -> Option<Result<Position, Box<dyn Error>>> {
                let position = Position {
                    x: x as i32,
                    y: y as i32,
                };
                if position == *start || position == *end {
                    Some(Ok(position))
                } else {
                    match is_junction(map, &position) {
                        Ok(true) => Some(Ok(position)),
                        Ok(false) => None,
                        Err(e) => Some(Err(e)),
                    }
                }
            })
        })
        .flatten()
        .collect::<Result<HashSet<_>, _>>()?;

    // build the graph
    let mut graph = Graph::new();
    vertices
        .iter()
        .try_for_each(|vertex| -> Result<_, Box<dyn Error>> {
            let mut stack: Vec<(Position, usize)> = vec![(vertex.clone(), 0)];
            let mut visited: HashSet<Position> = HashSet::new();

            while let Some((current, distance)) = stack.pop() {
                visited.insert(current.clone());
                if current != *vertex && vertices.contains(&current) {
                    // add the edge
                    graph
                        .entry(vertex.clone())
                        .or_default()
                        .push((current.clone(), distance));

                    continue;
                }

                let next_positions = get_adjacent_positions(map, &current)?;

                next_positions
                    .into_iter()
                    .filter(|next| !visited.contains(next))
                    .for_each(|next| stack.push((next, distance + 1)));
            }
            Ok(())
        })?;

    // DFS the graph to find the longest path from start to end
    let mut queue: VecDeque<(&Position, HashSet<&Position>, usize)> = VecDeque::new();
    queue.push_front((&start, HashSet::from([&start]), 0));
    let mut paths = Vec::new();

    while let Some((current, visited, distance)) = queue.pop_front() {
        if *current == end {
            paths.push(distance);
            continue;
        }

        let next_positions = graph.get(&current).ok_or("Invalid current position")?;

        next_positions
            .into_iter()
            .filter(|(next, _)| !visited.contains(next))
            .for_each(|(next_position, next_distance)| {
                let mut new_visited = visited.clone();
                new_visited.insert(current);
                queue.push_back((next_position, new_visited, distance + next_distance))
            });
    }

    paths.into_iter().max().ok_or("No path found".into())
}

fn is_junction(map: &Map, position: &Position) -> Result<bool, Box<dyn Error>> {
    Ok(get_adjacent_positions(map, position)?.len() > 2)
}

fn solve1(map: &Map) -> Result<usize, Box<dyn Error>> {
    let start = Position { x: 1, y: 0 };
    let end = Position {
        x: map.width as i32 - 2,
        y: map.height as i32 - 1,
    };

    let mut stack: Vec<Step> = Vec::new();
    stack.push(Step {
        current: start,
        visited: HashSet::new(),
    });
    let mut paths: Vec<usize> = Vec::new();

    while let Some(Step { current, visited }) = stack.pop() {
        if current == end {
            paths.push(visited.len());
            continue;
        }

        let next_positions = get_adjacent_positions(map, &current)?;

        next_positions
            .into_iter()
            .filter(|next| !visited.contains(next))
            .for_each(|next| {
                let mut visited = visited.clone();
                visited.insert(current.clone());

                stack.push(Step {
                    current: next,
                    visited,
                });
            });
    }

    paths.into_iter().max().ok_or("No path found".into())
}

#[cfg(test)]
mod day23 {
    use std::{
        error::Error,
        fs::File,
        io::{BufRead, BufReader},
    };

    use itertools::Itertools;

    use crate::{parse, remove_slopes, solve1, solve2};

    const EXAMPLE: &str = "\
        #.#####################\n\
        #.......#########...###\n\
        #######.#########.#.###\n\
        ###.....#.>.>.###.#.###\n\
        ###v#####.#v#.###.#.###\n\
        ###.>...#.#.#.....#...#\n\
        ###v###.#.#.#########.#\n\
        ###...#.#.#.......#...#\n\
        #####.#.#.#######.#.###\n\
        #.....#.#.#.......#...#\n\
        #.#####.#.#.#########v#\n\
        #.#...#...#...###...>.#\n\
        #.#.#v#######v###.###v#\n\
        #...#.>.#...>.>.#.###.#\n\
        #####v#.#.###v#.#.###.#\n\
        #.....#...#...#.#.#...#\n\
        #.#########.###.#.#.###\n\
        #...###...#...#...#.###\n\
        ###.###.#.###v#####v###\n\
        #...#...#.#.>.>.#.>.###\n\
        #.###.###.#.###.#.#v###\n\
        #.....###...###...#...#\n\
        #####################.#";

    #[test]
    fn test_parse() -> Result<(), Box<dyn Error>> {
        let map = parse(EXAMPLE.lines().map(|s| s.to_string()))?;

        assert_eq!(map.width, 23);
        assert_eq!(map.height, 23);

        Ok(())
    }

    #[test]
    fn test_solve1() -> Result<(), Box<dyn Error>> {
        let map = parse(EXAMPLE.lines().map(|s| s.to_string()))?;

        assert_eq!(solve1(&map)?, 94);

        Ok(())
    }

    #[test]
    fn test_solve1_noslopes() -> Result<(), Box<dyn Error>> {
        let map = parse(EXAMPLE.lines().map(remove_slopes))?;

        assert_eq!(solve1(&map)?, 154);

        Ok(())
    }

    #[test]
    fn test_solve2() -> Result<(), Box<dyn Error>> {
        let map = parse(EXAMPLE.lines().map(remove_slopes))?;

        assert_eq!(solve2(&map)?, 154);

        Ok(())
    }

    #[test]
    fn test_solve1_input() -> Result<(), Box<dyn Error>> {
        let file = File::open("input")?;
        let reader = BufReader::new(file);
        let map = reader.lines().process_results(|itr| parse(itr))??;
        let result = solve1(&map)?;

        assert_eq!(result, 1966);

        Ok(())
    }

    // too slow for running in tests
    // #[test]
    // fn test_solve2_input() -> Result<(), Box<dyn Error>> {
    //     let file = File::open("input")?;
    //     let reader = BufReader::new(file);
    //     let map = reader
    //         .lines()
    //         .process_results(|itr| parse(itr.map(|line| remove_slopes(&line))))??;
    //     let result = solve2(&map)?;

    //     assert_eq!(result, 6286);

    //     Ok(())
    // }
}
