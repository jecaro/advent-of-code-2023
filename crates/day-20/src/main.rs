use itertools::Itertools;
use lib::get_args;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    error::Error,
    io::{stdin, BufRead},
    ops::{Index, IndexMut},
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
            let nodes = stdin()
                .lock()
                .lines()
                .process_results(|lines| parse(lines))??;

            let result = if arg == "-1" {
                solve1(nodes)?
            } else {
                solve2(nodes)?
            };

            println!("{}", result);
        }
        _ => usage(prog_name),
    }

    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pulse {
    High,
    Low,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FlipFlopState {
    On,
    Off,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum NodeType {
    Broadcast,
    FlipFlop { state: FlipFlopState },
    Conjunction { inputs: HashMap<String, Pulse> },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Node {
    name: String,
    node_type: NodeType,
    outputs: Vec<String>,
}

fn parse(itr: impl Iterator<Item = String>) -> Result<Vec<Node>, Box<dyn Error>> {
    itr.map(|line| {
        let (name_str, outputs_str) = line.split_once(" -> ").ok_or("Invalid line")?;

        let outputs = outputs_str
            .split(", ")
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        let node_type = match name_str.chars().next().ok_or("Invalid line")? {
            '%' => NodeType::FlipFlop {
                state: FlipFlopState::Off,
            },
            '&' => NodeType::Conjunction {
                inputs: HashMap::new(),
            },
            _ => NodeType::Broadcast,
        };

        let name_str = name_str.trim_start_matches("&").trim_start_matches("%");

        Ok(Node {
            name: name_str.to_string(),
            node_type,
            outputs,
        })
    })
    .collect()
}

fn to_map(nodes: Vec<Node>) -> HashMap<String, Node> {
    nodes
        .into_iter()
        .map(|node| (node.name.clone(), node))
        .collect()
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SendPulse {
    from: String,
    to: String,
    pulse: Pulse,
}

fn init_conjunctions(nodes: &mut HashMap<String, Node>) -> () {
    // updating the hashmap while iterating over it is not possible in Rust that's why we clone it
    nodes.clone().iter().for_each(|(name, node)| {
        node.outputs.iter().for_each(|output| {
            nodes
                .get_mut(output)
                .map(|output_node| match &mut output_node.node_type {
                    NodeType::Conjunction { inputs } => {
                        inputs.insert(name.clone(), Pulse::Low);
                    }
                    _ => {}
                });
        })
    })
}

fn init(nodes: Vec<Node>) -> HashMap<String, Node> {
    let mut nodes = to_map(nodes);
    init_conjunctions(&mut nodes);

    nodes
}

fn solve(nodes: Vec<Node>, count: i32) -> Result<PulseCount, Box<dyn Error>> {
    let mut nodes = init(nodes);

    push_button_count(&mut nodes, count)
}

fn solve1(nodes: Vec<Node>) -> Result<i64, Box<dyn Error>> {
    let result = solve(nodes, 1000)?;

    Ok(result.high * result.low)
}

fn get_parents(nodes: &HashMap<String, Node>, name: &str) -> Vec<String> {
    nodes
        .iter()
        .filter_map(|(node_name, node)| {
            node.outputs
                .contains(&name.to_string())
                .then_some(node_name.clone())
        })
        .collect()
}

fn solve2(nodes: Vec<Node>) -> Result<i64, Box<dyn Error>> {
    let mut nodes = init(nodes);

    let rx_parents = get_parents(&nodes, "rx");
    let rx_grand_parents = rx_parents
        .iter()
        .flat_map(|name| get_parents(&nodes, name))
        .collect::<HashSet<_>>();

    // we will record in this hash map the number of pushes on the button that triggers a high
    // pulse to the grand parents of rx
    let mut found_conjunctions: HashMap<String, i64> = HashMap::new();
    let mut i = 0;

    while found_conjunctions.len() != rx_grand_parents.len() {
        i += 1;
        let (_, new_found_conjunctions) = push_button(&mut nodes, &rx_grand_parents)?;

        new_found_conjunctions.iter().for_each(|name| {
            found_conjunctions.entry(name.clone()).or_insert(i);
        });
    }

    // we assume that this number of pushes happen in a cycle then the result might be the product
    // of all these cycles (or the LCM of all these numbers)
    Ok(found_conjunctions
        .iter()
        .map(|(_, count)| *count)
        .product::<i64>())
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PulseCount {
    high: i64,
    low: i64,
}

impl Default for PulseCount {
    fn default() -> Self {
        Self { high: 0, low: 0 }
    }
}

impl Index<Pulse> for PulseCount {
    type Output = i64;

    fn index(&self, pulse: Pulse) -> &Self::Output {
        match pulse {
            Pulse::High => &self.high,
            Pulse::Low => &self.low,
        }
    }
}

impl IndexMut<Pulse> for PulseCount {
    fn index_mut(&mut self, pulse: Pulse) -> &mut Self::Output {
        match pulse {
            Pulse::High => &mut self.high,
            Pulse::Low => &mut self.low,
        }
    }
}

fn push_button(
    nodes: &mut HashMap<String, Node>,
    searched_conjunctions: &HashSet<String>,
) -> Result<(PulseCount, HashSet<String>), Box<dyn Error>> {
    let mut stack = VecDeque::new();
    stack.push_back(SendPulse {
        from: "button".to_string(),
        to: "broadcaster".to_string(),
        pulse: Pulse::Low,
    });

    let mut pulse_count = PulseCount::default();
    pulse_count[Pulse::Low] += 1;

    let mut found_conjunctions = HashSet::new();

    while let Some(SendPulse { from, to, pulse }) = stack.pop_front() {
        nodes
            .get_mut(&to)
            .map_or(Ok(()), |node| -> Result<_, Box<dyn Error>> {
                match &mut node.node_type {
                    NodeType::Broadcast => {
                        node.outputs.iter().for_each(|output| {
                            stack.push_back(SendPulse {
                                from: to.clone(),
                                to: output.clone(),
                                pulse,
                            });
                        });
                        pulse_count[pulse] += node.outputs.len() as i64;
                    }
                    NodeType::FlipFlop { ref mut state } => match pulse {
                        Pulse::High => {}
                        Pulse::Low => {
                            *state = flip(state);
                            let pulse = match state {
                                FlipFlopState::On => Pulse::High,
                                FlipFlopState::Off => Pulse::Low,
                            };

                            node.outputs.iter().for_each(|output| {
                                stack.push_back(SendPulse {
                                    from: to.clone(),
                                    to: output.clone(),
                                    pulse,
                                });
                            });
                            pulse_count[pulse] += node.outputs.len() as i64;
                        }
                    },
                    NodeType::Conjunction { inputs } => {
                        *inputs.get_mut(&from).ok_or("Invalid input")? = pulse;
                        let all_high = inputs.values().all(|&p| p == Pulse::High);
                        let pulse = if all_high { Pulse::Low } else { Pulse::High };

                        node.outputs.iter().for_each(|output| {
                            stack.push_back(SendPulse {
                                from: to.clone(),
                                to: output.clone(),
                                pulse,
                            });
                        });

                        pulse_count[pulse] += node.outputs.len() as i64;

                        if (pulse == Pulse::High) && searched_conjunctions.contains(&node.name) {
                            found_conjunctions.insert(node.name.clone());
                        }
                    }
                }
                Ok(())
            })?;
    }

    Ok((pulse_count, found_conjunctions))
}

fn push_button_count(
    nodes: &mut HashMap<String, Node>,
    count: i32,
) -> Result<PulseCount, Box<dyn Error>> {
    (0..count).try_fold(
        PulseCount::default(),
        |mut acc, _| -> Result<PulseCount, Box<dyn Error>> {
            let (new_result, _) = push_button(nodes, &HashSet::new())?;
            acc.high += new_result.high;
            acc.low += new_result.low;

            Ok(acc)
        },
    )
}

fn flip(state: &FlipFlopState) -> FlipFlopState {
    match state {
        FlipFlopState::On => FlipFlopState::Off,
        FlipFlopState::Off => FlipFlopState::On,
    }
}

#[cfg(test)]
mod day20 {
    use std::{
        collections::HashMap,
        error::Error,
        fs::File,
        io::{BufRead, BufReader},
    };

    use itertools::Itertools;

    use crate::{parse, solve, solve1, solve2, FlipFlopState, Node, NodeType, PulseCount};

    const EXAMPLE1: &str = "\
        broadcaster -> a, b, c\n\
        %a -> b\n\
        %b -> c\n\
        %c -> inv\n\
        &inv -> a";

    fn example1() -> Vec<Node> {
        vec![
            Node {
                name: "broadcaster".to_string(),
                node_type: NodeType::Broadcast,
                outputs: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            },
            Node {
                name: "a".to_string(),
                node_type: NodeType::FlipFlop {
                    state: FlipFlopState::Off,
                },
                outputs: vec!["b".to_string()],
            },
            Node {
                name: "b".to_string(),
                node_type: NodeType::FlipFlop {
                    state: FlipFlopState::Off,
                },
                outputs: vec!["c".to_string()],
            },
            Node {
                name: "c".to_string(),
                node_type: NodeType::FlipFlop {
                    state: FlipFlopState::Off,
                },
                outputs: vec!["inv".to_string()],
            },
            Node {
                name: "inv".to_string(),
                node_type: NodeType::Conjunction {
                    inputs: HashMap::new(),
                },
                outputs: vec!["a".to_string()],
            },
        ]
    }

    const EXAMPLE2: &str = "\
        broadcaster -> a\n\
        %a -> inv, con\n\
        &inv -> b\n\
        %b -> con\n\
        &con -> output";

    fn example2() -> Vec<Node> {
        vec![
            Node {
                name: "broadcaster".to_string(),
                node_type: NodeType::Broadcast,
                outputs: vec!["a".to_string()],
            },
            Node {
                name: "a".to_string(),
                node_type: NodeType::FlipFlop {
                    state: FlipFlopState::Off,
                },
                outputs: vec!["inv".to_string(), "con".to_string()],
            },
            Node {
                name: "inv".to_string(),
                node_type: NodeType::Conjunction {
                    inputs: HashMap::new(),
                },
                outputs: vec!["b".to_string()],
            },
            Node {
                name: "b".to_string(),
                node_type: NodeType::FlipFlop {
                    state: FlipFlopState::Off,
                },
                outputs: vec!["con".to_string()],
            },
            Node {
                name: "con".to_string(),
                node_type: NodeType::Conjunction {
                    inputs: HashMap::new(),
                },
                outputs: vec!["output".to_string()],
            },
        ]
    }

    #[test]
    fn test_parse_example1() -> Result<(), Box<dyn Error>> {
        let result = parse(EXAMPLE1.lines().map(|s| s.to_string()))?;
        assert_eq!(result, example1());
        Ok(())
    }

    #[test]
    fn test_parse_example2() -> Result<(), Box<dyn Error>> {
        let result = parse(EXAMPLE2.lines().map(|s| s.to_string()))?;
        assert_eq!(result, example2());
        Ok(())
    }

    #[test]
    fn test_solve_example1() -> Result<(), Box<dyn Error>> {
        let result = solve(example1(), 1)?;
        assert_eq!(result, PulseCount { high: 4, low: 8 });
        Ok(())
    }

    #[test]
    fn test_solve_example2() -> Result<(), Box<dyn Error>> {
        let result = solve(example2(), 1000)?;
        assert_eq!(
            result,
            PulseCount {
                high: 2750,
                low: 4250
            }
        );
        Ok(())
    }

    #[test]
    fn test_solve1_example1() -> Result<(), Box<dyn Error>> {
        let result = solve1(example1())?;
        assert_eq!(result, 32000000);
        Ok(())
    }

    #[test]
    fn test_solve1_example2() -> Result<(), Box<dyn Error>> {
        let result = solve1(example2())?;
        assert_eq!(result, 11687500);
        Ok(())
    }

    #[test]
    fn test_solve1_input() -> Result<(), Box<dyn Error>> {
        let file = File::open("input")?;
        let reader = BufReader::new(file);
        let nodes = reader.lines().process_results(|itr| parse(itr))??;
        let result = solve1(nodes).unwrap();

        assert_eq!(result, 944750144);
        Ok(())
    }

    #[test]
    fn test_solve2_input() -> Result<(), Box<dyn Error>> {
        let file = File::open("input")?;
        let reader = BufReader::new(file);
        let nodes = reader.lines().process_results(|itr| parse(itr))??;
        let result = solve2(nodes)?;

        assert_eq!(result, 222718819437131);
        Ok(())
    }
}
