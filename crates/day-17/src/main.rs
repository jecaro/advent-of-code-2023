use itertools::Itertools;
use lib::get_args;
use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap},
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
            let graph = stdin()
                .lock()
                .lines()
                .process_results(|lines| parse(lines))??;

            let result = if arg == "-1" {
                solve1(graph)
            } else {
                solve2(graph)
            }?;

            println!("{}", result);
        }
        _ => usage(prog_name),
    }

    Ok(())
}

fn parse(itr: impl Iterator<Item = String>) -> Result<Graph, Box<dyn Error>> {
    let mut width = 0;
    let graph: Vec<Vec<_>> = itr
        .map(|line| -> Result<Vec<u32>, String> {
            if width == 0 {
                width = line.len();
            } else if width != line.len() {
                return Err("Invalid line length".to_string());
            }
            line.chars()
                .map(|c| c.to_digit(10).ok_or("Invalid digit".to_string()))
                .collect()
        })
        .process_results(|itr| itr.collect())?;
    let height = graph.len();

    Ok(Graph {
        graph,
        width,
        height,
    })
}

struct Graph {
    graph: Vec<Vec<u32>>,
    width: usize,
    height: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Vertex {
    x: usize,
    y: usize,
    orientation: Orientation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct QueueElement {
    vertex: Vertex,
    dist: u32,
}

impl PartialOrd for QueueElement {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.dist.cmp(&other.dist).reverse())
    }
}

impl Ord for QueueElement {
    fn cmp(&self, other: &Self) -> Ordering {
        self.dist.cmp(&other.dist).reverse()
    }
}

fn graph_get(
    Graph {
        graph,
        width: _,
        height: _,
    }: &Graph,
    x: usize,
    y: usize,
) -> Option<u32> {
    graph.get(y).and_then(|row| row.get(x)).copied()
}

fn solve1(graph: Graph) -> Result<u32, Box<dyn Error>> {
    solve(graph, get_neighbors1)
}

fn solve2(graph: Graph) -> Result<u32, Box<dyn Error>> {
    solve(graph, get_neighbors2)
}

type GetNeighbors = fn(&Graph, Vertex) -> Vec<(Vertex, u32)>;

fn get_neighbors1(graph: &Graph, vertex: Vertex) -> Vec<(Vertex, u32)> {
    // a range and a reverse range are not the same type, therefore they cannot be part of the same
    // array
    [(1 as i32..=3).collect(), (-3 as i32..=-1).rev().collect()]
        .iter()
        .flat_map(|range: &Vec<i32>| -> Vec<(Vertex, u32)> {
            let mut dist: u32 = 0;
            range
                .iter()
                .filter_map(|offset| {
                    move_(vertex, graph.width, graph.height, *offset).and_then(|next| {
                        dist += graph_get(graph, next.x, next.y)?;

                        Some((next, dist))
                    })
                })
                .collect::<Vec<(Vertex, u32)>>()
        })
        .collect()
}

fn get_neighbors2(graph: &Graph, vertex: Vertex) -> Vec<(Vertex, u32)> {
    // we start at 1 and -1 to rightly compute the distance on the way
    [(1 as i32..=10).collect(), (-10 as i32..=-1).rev().collect()]
        .iter()
        .flat_map(|range: &Vec<i32>| -> Vec<(Vertex, u32)> {
            let mut dist: u32 = 0;
            range
                .iter()
                .filter_map(|offset| {
                    move_(vertex, graph.width, graph.height, *offset).and_then(|next| {
                        dist += graph_get(graph, next.x, next.y)?;

                        // discard vertices that are too close
                        (*offset > 3 || *offset < -3).then_some((next, dist))
                    })
                })
                .collect::<Vec<(Vertex, u32)>>()
        })
        .collect()
}

fn move_(vertex: Vertex, width: usize, height: usize, d: i32) -> Option<Vertex> {
    match vertex.orientation {
        Orientation::Horizontal => {
            let nx = vertex.x as i32 + d;
            (nx >= 0 && nx < width as i32).then_some(Vertex {
                x: nx as usize,
                y: vertex.y,
                orientation: Orientation::Vertical,
            })
        }
        Orientation::Vertical => {
            let ny = vertex.y as i32 + d;
            (ny >= 0 && ny < height as i32).then_some(Vertex {
                x: vertex.x,
                y: ny as usize,
                orientation: Orientation::Horizontal,
            })
        }
    }
}

fn solve(graph: Graph, neighbors: GetNeighbors) -> Result<u32, Box<dyn Error>> {
    let mut queue: BinaryHeap<QueueElement> = BinaryHeap::new();
    queue.push(QueueElement {
        vertex: Vertex {
            x: 0,
            y: 0,
            orientation: Orientation::Horizontal,
        },
        dist: 0,
    });
    queue.push(QueueElement {
        vertex: Vertex {
            x: 0,
            y: 0,
            orientation: Orientation::Vertical,
        },
        dist: 0,
    });

    let mut distances: HashMap<Vertex, u32> = HashMap::new();
    distances.insert(
        Vertex {
            x: 0,
            y: 0,
            orientation: Orientation::Vertical,
        },
        0,
    );
    distances.insert(
        Vertex {
            x: 0,
            y: 0,
            orientation: Orientation::Horizontal,
        },
        0,
    );

    // track of the previous vertices for debugging
    let mut prev: HashMap<Vertex, Vertex> = HashMap::new();

    let mut result: Option<u32> = None;

    while let Some(QueueElement {
        vertex: current,
        dist,
    }) = queue.pop()
    {
        if current.x == graph.width - 1 && current.y == graph.height - 1 {
            result = result.map_or(Some(dist), |result| {
                Some(if dist < result { dist } else { result })
            });
        }

        neighbors(&graph, current)
            .iter()
            .for_each(|(neighbor, relative_dist)| {
                let dist = dist + relative_dist;
                let prev_dist = distances.get(neighbor).unwrap_or(&u32::MAX);
                if dist < *prev_dist {
                    queue.push(QueueElement {
                        vertex: *neighbor,
                        dist,
                    });
                    distances.insert(*neighbor, dist);
                    prev.insert(*neighbor, current);
                }
            });
    }

    result.ok_or("No path found".into())
}

#[cfg(test)]
mod day17 {
    use std::{
        error::Error,
        fs::File,
        io::{BufRead, BufReader},
    };

    use itertools::Itertools;

    use crate::{parse, solve1, solve2};

    const EXAMPLE1: &str = "\
        2413432311323\n\
        3215453535623\n\
        3255245654254\n\
        3446585845452\n\
        4546657867536\n\
        1438598798454\n\
        4457876987766\n\
        3637877979653\n\
        4654967986887\n\
        4564679986453\n\
        1224686865563\n\
        2546548887735\n\
        4322674655533";

    const EXAMPLE2: &str = "\
        111111111111\n\
        999999999991\n\
        999999999991\n\
        999999999991\n\
        999999999991";

    #[test]
    fn test_solve1_example1() -> Result<(), Box<dyn Error>> {
        let graph = parse(EXAMPLE1.lines().map(|s| s.to_string()))?;
        assert_eq!(solve1(graph)?, 102);
        Ok(())
    }

    #[test]
    fn test_solve2_example1() -> Result<(), Box<dyn Error>> {
        let graph = parse(EXAMPLE1.lines().map(|s| s.to_string()))?;
        assert_eq!(solve2(graph)?, 94);
        Ok(())
    }

    #[test]
    fn test_solve2_example2() -> Result<(), Box<dyn Error>> {
        let graph = parse(EXAMPLE2.lines().map(|s| s.to_string()))?;
        assert_eq!(solve2(graph)?, 71);
        Ok(())
    }

    #[test]
    fn test_solve1_input() -> Result<(), Box<dyn Error>> {
        let file = File::open("input")?;
        let reader = BufReader::new(file);
        let graph = reader.lines().process_results(|itr| parse(itr))??;
        let result = solve1(graph)?;
        assert_eq!(result, 722);
        Ok(())
    }

    #[test]
    fn test_solve2_input() -> Result<(), Box<dyn Error>> {
        let file = File::open("input")?;
        let reader = BufReader::new(file);
        let graph = reader.lines().process_results(|itr| parse(itr))??;
        let result = solve2(graph)?;
        assert_eq!(result, 894);
        Ok(())
    }
}
