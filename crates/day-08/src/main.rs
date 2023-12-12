use itertools::FoldWhile::{Continue, Done};
use itertools::{process_results, Itertools};
use lib::{get_args, INVALID_INPUT};
use num::integer::lcm;
use std::io::BufRead;
use std::{collections::HashMap, error::Error, io, process::exit};

fn usage(prog_name: String) {
    println!("Usage: {} [-1|-2|-h]", prog_name);
    exit(0)
}

fn main() -> Result<(), Box<dyn Error>> {
    let (prog_name, args) = get_args()?;

    match args.get(0) {
        Some(arg) if arg == "-1" || arg == "-2" => {
            let solve = match arg.as_str() {
                "-1" => |path, nodes| solve1(path, "AAA".to_string(), nodes),
                _ => solve2,
            };

            let input = io::stdin().lock().lines();
            let (path, nodes) = process_results(input, |itr| parse_input(itr))??;
            let result = solve(path, nodes)?;

            println!("{}", result)
        }
        _ => usage(prog_name),
    }
    Ok(())
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum Direction {
    Left,
    Right,
}

type Path = Vec<Direction>;
type Label = String;

#[derive(Debug, PartialEq, Eq, Clone)]
struct Directions {
    left: Label,
    right: Label,
}

type Node = (Label, Directions);

fn parse_path(s: &str) -> Result<Path, Box<dyn Error>> {
    s.chars()
        .map(|c| match c {
            'L' => Ok(Direction::Left),
            'R' => Ok(Direction::Right),
            _ => Err("Invalid direction".into()),
        })
        .collect()
}

fn parse_line(s: &str) -> Result<Node, Box<dyn Error>> {
    let without_whitespaces = s
        .chars()
        .filter(|c| !(*c == '(' || *c == ')' || c.is_whitespace()))
        .collect::<String>();

    let (label, directions_str) = without_whitespaces.split_once('=').ok_or(INVALID_INPUT)?;
    let (left, right) = directions_str.split_once(',').ok_or(INVALID_INPUT)?;

    Ok((
        label.to_string(),
        Directions {
            left: left.to_string(),
            right: right.to_string(),
        },
    ))
}

fn parse_input(lines: impl Iterator<Item = String>) -> Result<(Path, Vec<Node>), Box<dyn Error>> {
    let mut lines = lines;
    let path = parse_path(&lines.next().ok_or(INVALID_INPUT)?)?;

    lines.next();

    let nodes = lines
        .map(|line| parse_line(&line))
        .collect::<Result<Vec<_>, _>>()?;

    Ok((path, nodes))
}

fn solve1(path: Path, start_node: String, nodes: Vec<Node>) -> Result<u64, Box<dyn Error>> {
    let map: HashMap<_, _> = nodes.into_iter().collect();

    path.iter()
        .cycle()
        .fold_while(
            Ok((start_node, 0)),
            |acc: Result<(String, u64), Box<dyn Error>>, current| match acc {
                e @ Err(_) => Done(e),
                Ok((label, count)) => {
                    if label.ends_with('Z') {
                        Done(Ok((label, count)))
                    } else {
                        match map.get(&label) {
                            Some(directions) => {
                                let next_node = if *current == Direction::Left {
                                    directions.left.clone()
                                } else {
                                    directions.right.clone()
                                };
                                Continue(Ok((next_node, count + 1)))
                            }
                            None => Done(Err("Unable to find the label into the map".into())),
                        }
                    }
                }
            },
        )
        .into_inner()
        .map(|(_, count)| count)
}

fn solve2(path: Path, nodes: Vec<Node>) -> Result<u64, Box<dyn Error>> {
    nodes
        .iter()
        .filter(|(label, _)| label.ends_with('A'))
        .map(|(node, _)| solve1(path.clone(), node.to_string(), nodes.clone()))
        .reduce(|x, y| Ok(lcm(x?, y?)))
        .ok_or("Empty node list")?
}

#[cfg(test)]
mod day08 {
    use std::{
        fs::File,
        io::{BufRead, BufReader},
    };

    use itertools::process_results;

    use crate::{parse_input, solve1, solve2, Direction, Directions, Node, Path};

    const EXAMPLE1: &str = "\
        RL\n\
        \n\
        AAA = (BBB, CCC)\n\
        BBB = (DDD, EEE)\n\
        CCC = (ZZZ, GGG)\n\
        DDD = (DDD, DDD)\n\
        EEE = (EEE, EEE)\n\
        GGG = (GGG, GGG)\n\
        ZZZ = (ZZZ, ZZZ)";

    fn example1() -> (Path, Vec<Node>) {
        (
            vec![Direction::Right, Direction::Left],
            vec![
                (
                    "AAA".to_string(),
                    Directions {
                        left: "BBB".to_string(),
                        right: "CCC".to_string(),
                    },
                ),
                (
                    "BBB".to_string(),
                    Directions {
                        left: "DDD".to_string(),
                        right: "EEE".to_string(),
                    },
                ),
                (
                    "CCC".to_string(),
                    Directions {
                        left: "ZZZ".to_string(),
                        right: "GGG".to_string(),
                    },
                ),
                (
                    "DDD".to_string(),
                    Directions {
                        left: "DDD".to_string(),
                        right: "DDD".to_string(),
                    },
                ),
                (
                    "EEE".to_string(),
                    Directions {
                        left: "EEE".to_string(),
                        right: "EEE".to_string(),
                    },
                ),
                (
                    "GGG".to_string(),
                    Directions {
                        left: "GGG".to_string(),
                        right: "GGG".to_string(),
                    },
                ),
                (
                    "ZZZ".to_string(),
                    Directions {
                        left: "ZZZ".to_string(),
                        right: "ZZZ".to_string(),
                    },
                ),
            ],
        )
    }

    const EXAMPLE2: &str = "\
        LLR\n\
        \n\
        AAA = (BBB, BBB)\n\
        BBB = (AAA, ZZZ)\n\
        ZZZ = (ZZZ, ZZZ)";

    fn example2() -> (Path, Vec<Node>) {
        (
            vec![Direction::Left, Direction::Left, Direction::Right],
            vec![
                (
                    "AAA".to_string(),
                    Directions {
                        left: "BBB".to_string(),
                        right: "BBB".to_string(),
                    },
                ),
                (
                    "BBB".to_string(),
                    Directions {
                        left: "AAA".to_string(),
                        right: "ZZZ".to_string(),
                    },
                ),
                (
                    "ZZZ".to_string(),
                    Directions {
                        left: "ZZZ".to_string(),
                        right: "ZZZ".to_string(),
                    },
                ),
            ],
        )
    }

    const EXAMPLE3: &str = "\
        LR\n\
        \n\
        11A = (11B, XXX)\n\
        11B = (XXX, 11Z)\n\
        11Z = (11B, XXX)\n\
        22A = (22B, XXX)\n\
        22B = (22C, 22C)\n\
        22C = (22Z, 22Z)\n\
        22Z = (22B, 22B)\n\
        XXX = (XXX, XXX)";

    fn example3() -> (Path, Vec<Node>) {
        (
            vec![Direction::Left, Direction::Right],
            vec![
                (
                    "11A".to_string(),
                    Directions {
                        left: "11B".to_string(),
                        right: "XXX".to_string(),
                    },
                ),
                (
                    "11B".to_string(),
                    Directions {
                        left: "XXX".to_string(),
                        right: "11Z".to_string(),
                    },
                ),
                (
                    "11Z".to_string(),
                    Directions {
                        left: "11B".to_string(),
                        right: "XXX".to_string(),
                    },
                ),
                (
                    "22A".to_string(),
                    Directions {
                        left: "22B".to_string(),
                        right: "XXX".to_string(),
                    },
                ),
                (
                    "22B".to_string(),
                    Directions {
                        left: "22C".to_string(),
                        right: "22C".to_string(),
                    },
                ),
                (
                    "22C".to_string(),
                    Directions {
                        left: "22Z".to_string(),
                        right: "22Z".to_string(),
                    },
                ),
                (
                    "22Z".to_string(),
                    Directions {
                        left: "22B".to_string(),
                        right: "22B".to_string(),
                    },
                ),
                (
                    "XXX".to_string(),
                    Directions {
                        left: "XXX".to_string(),
                        right: "XXX".to_string(),
                    },
                ),
            ],
        )
    }

    #[test]
    fn test_parse_example1() {
        let parsed_example = parse_input(EXAMPLE1.lines().map(|line| line.to_string())).unwrap();

        assert_eq!(parsed_example, example1());
    }

    #[test]
    fn test_parse_example2() {
        let parsed_example = parse_input(EXAMPLE2.lines().map(|line| line.to_string())).unwrap();

        assert_eq!(parsed_example, example2());
    }

    #[test]
    fn test_parse_example3() {
        let parsed_example = parse_input(EXAMPLE3.lines().map(|line| line.to_string())).unwrap();

        assert_eq!(parsed_example, example3());
    }

    #[test]
    fn test_solve1_example1() {
        assert_eq!(
            solve1(example1().0, "AAA".to_string(), example1().1).unwrap(),
            2
        );
    }

    #[test]
    fn test_solve1_example2() {
        assert_eq!(
            solve1(example2().0, "AAA".to_string(), example2().1).unwrap(),
            6
        );
    }

    #[test]
    fn test_solve2_example3() {
        assert_eq!(solve2(example3().0, example3().1).unwrap(), 6);
    }

    #[test]
    fn test_solve1_input() {
        let file = File::open("input").unwrap();
        let reader = BufReader::new(file);
        let (path, nodes) = process_results(reader.lines(), |itr| parse_input(itr))
            .unwrap()
            .unwrap();

        assert_eq!(solve1(path, "AAA".to_string(), nodes).unwrap(), 16531);
    }

    #[test]
    fn test_solve2_input() {
        let file = File::open("input").unwrap();
        let reader = BufReader::new(file);
        let (path, nodes) = process_results(reader.lines(), |itr| parse_input(itr))
            .unwrap()
            .unwrap();

        assert_eq!(solve2(path, nodes).unwrap(), 24035773251517);
    }
}
