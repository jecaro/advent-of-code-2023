use itertools::process_results;
use itertools::Itertools;
use lib::get_args;
use rand::prelude::IteratorRandom;
use rand::thread_rng;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
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
            let graph = process_results(stdin().lock().lines(), |lines| parse(lines))??;
            let result = solve(&graph)?;

            println!("{}", result);
        }
        _ => usage(prog_name),
    }

    Ok(())
}

type Graph = HashMap<String, HashSet<String>>;

fn parse(itr: impl Iterator<Item = String>) -> Result<Graph, Box<dyn Error>> {
    Ok(itr
        .map(|line| -> Result<_, Box<dyn Error>> {
            let (key, values) = line.split_once(": ").ok_or("Invalid input")?;
            Ok(values
                .split(' ')
                .map(|v| (key.to_string(), v.to_string()))
                .collect::<Vec<(String, String)>>())
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .fold(HashMap::new(), |mut acc, (k, v)| {
            acc.entry(k.clone())
                .or_insert_with(HashSet::new)
                .insert(v.clone());
            acc.entry(v).or_insert_with(HashSet::new).insert(k);
            acc
        }))
}

fn most_common_edges(graph: &Graph, count: i32) -> Result<Vec<(&String, &String)>, Box<dyn Error>> {
    let mut edges: HashMap<(&String, &String), usize> = HashMap::new();

    (0..count).try_for_each(|_| -> Result<(), Box<dyn Error>> {
        // get two random keys
        let key1 = graph
            .keys()
            .choose(&mut thread_rng())
            .ok_or("Invalid key")?;
        let key2 = graph
            .keys()
            .choose(&mut thread_rng())
            .ok_or("Invalid key")?;

        // now find a path from key1 to key2
        let mut visited: HashSet<&String> = HashSet::new();
        let mut queue: VecDeque<Vec<&String>> = VecDeque::new();
        queue.push_front(vec![key1]);

        while let Some(current_path) = queue.pop_back() {
            let key = current_path.last().ok_or("Invalid path")?;

            if *key == key2 {
                current_path
                    .into_iter()
                    .tuple_windows()
                    .for_each(|(k1, k2)| {
                        let (k1, k2) = if k1 < k2 { (k1, k2) } else { (k2, k1) };
                        edges.entry((k1, k2)).and_modify(|e| *e += 1).or_insert(1);
                    });
                break;
            }

            if visited.contains(key) {
                continue;
            }

            visited.insert(key);

            graph.get(*key).ok_or("Invalid key")?.iter().for_each(|k| {
                let mut new_path = current_path.clone();
                new_path.push(k);
                queue.push_front(new_path);
            });
        }
        Ok(())
    })?;

    // take the three most common edges
    Ok(edges
        .into_iter()
        .sorted_by_key(|(_, v)| *v)
        .rev()
        .take(3)
        .map(|(e, _)| e)
        .sorted()
        .collect::<Vec<_>>())
}

fn remove_edges(graph: &Graph, edges: &Vec<(&String, &String)>) -> Graph {
    graph
        .into_iter()
        .map(|(k, v)| v.iter().map(move |v| (k, v)))
        .flatten()
        .filter(|(k, v)| !edges.contains(&(k, v)) && !edges.contains(&(v, k)))
        .fold(HashMap::new(), |mut acc, (k, v)| {
            acc.entry(k.clone())
                .or_insert_with(HashSet::new)
                .insert(v.clone());
            acc.entry(v.clone())
                .or_insert_with(HashSet::new)
                .insert(k.clone());
            acc
        })
}

fn solve(graph: &Graph) -> Result<usize, Box<dyn Error>> {
    let most_common = most_common_edges(graph, 300)?;

    // now remove those three edges from a copy of the graph
    let graph = remove_edges(graph, &most_common);

    // now find the connected components
    let node = graph.keys().next().ok_or("Invalid graph")?;

    let visited = bfs_visit(&graph, &node, HashSet::new())?;
    let count1 = visited.len();

    // another non visited node
    let node = graph
        .keys()
        .find(|k| !visited.contains(*k))
        .ok_or("Invalid graph")?;

    let visited = bfs_visit(&graph, &node, HashSet::new())?;
    let count2 = visited.len();

    assert_eq!(count1 + count2, graph.len());

    Ok(count1 * count2)
}

fn bfs_visit<'a>(
    graph: &'a HashMap<String, HashSet<String>>,
    node: &'a String,
    mut visited: HashSet<&'a String>,
) -> Result<HashSet<&'a String>, Box<dyn Error>> {
    let mut queue: VecDeque<&String> = VecDeque::new();
    queue.push_front(&node);

    while let Some(node) = queue.pop_back() {
        if visited.contains(&node) {
            continue;
        }

        visited.insert(&node);

        graph.get(node).ok_or("Invalid node")?.iter().for_each(|n| {
            queue.push_front(n);
        });
    }

    Ok(visited)
}

#[cfg(test)]
mod day25 {
    use std::{
        fs::File,
        io::{BufRead, BufReader},
    };

    use itertools::process_results;

    use crate::{parse, solve};

    const EXAMPLE: &str = "\
        jqt: rhn xhk nvd\n\
        rsh: frs pzl lsr\n\
        xhk: hfx\n\
        cmg: qnr nvd lhk bvb\n\
        rhn: xhk bvb hfx\n\
        bvb: xhk hfx\n\
        pzl: lsr hfx nvd\n\
        qnr: nvd\n\
        ntq: jqt hfx bvb xhk\n\
        nvd: lhk\n\
        lsr: lhk\n\
        rzs: qnr cmg lsr rsh\n\
        frs: qnr lhk lsr";

    #[test]
    fn test_parse() {
        let graph = parse(EXAMPLE.lines().map(|s| s.to_string())).unwrap();

        assert_eq!(graph.len(), 15);
    }

    #[test]
    fn test_solve() {
        let graph = parse(EXAMPLE.lines().map(|s| s.to_string())).unwrap();

        assert_eq!(solve(&graph).unwrap(), 54);
    }

    #[test]
    fn test_solve_input() {
        let file = File::open("input").unwrap();
        let reader = BufReader::new(file);
        let graph = process_results(reader.lines(), |itr| parse(itr))
            .unwrap()
            .unwrap();
        let result = solve(&graph).unwrap();

        assert_eq!(result, 527790);
    }
}
