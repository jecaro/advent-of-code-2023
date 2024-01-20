use itertools::intersperse;
use itertools::Itertools;
use lib::{get_args, INVALID_INPUT};
use std::collections::HashMap;
use std::{
    error::Error,
    io::{stdin, BufRead},
    iter::repeat,
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
            let result = stdin().lock().lines().process_results(|itr| {
                itr.map(|line| parse_line(line))
                    .process_results(|itr| match arg.as_str() {
                        "-1" => solve1(itr),
                        _ => solve2(itr),
                    })
            })??;

            println!("{}", result);
        }
        _ => usage(prog_name),
    }
    Ok(())
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum Spring {
    Operational,
    Damaged,
    Unknown,
}

#[derive(Debug, PartialEq, Eq)]
struct InputLine {
    springs: Vec<Spring>,
    damaged: Vec<i64>,
}

fn repeat_five(input_line: &InputLine) -> InputLine {
    let springs: Vec<Spring> = intersperse(
        repeat(input_line.springs.clone()).take(5),
        vec![Spring::Unknown; 1],
    )
    .flatten()
    .collect();

    let damaged = repeat(&input_line.damaged)
        .take(5)
        .flatten()
        .cloned()
        .collect();

    InputLine { springs, damaged }
}

#[allow(dead_code)]
fn display(springs: &[Spring]) -> String {
    springs
        .iter()
        .map(|s| match s {
            Spring::Operational => '.',
            Spring::Damaged => '#',
            Spring::Unknown => '?',
        })
        .collect()
}

fn solve1(itr: impl Iterator<Item = InputLine>) -> i64 {
    itr.map(|line| combinations2(&line)).sum()
}

fn solve2(itr: impl Iterator<Item = InputLine>) -> i64 {
    itr.map(|line| repeat_five(&line))
        .map(|line| combinations2(&line))
        .sum()
}

fn check(springs: &[Spring], damaged_count: &[i64]) -> bool {
    let damaged_count_in_springs = springs
        .split(|s| *s != Spring::Damaged)
        .filter(|s| !s.is_empty())
        .map(|s| s.len() as i64)
        .collect::<Vec<_>>();

    damaged_count_in_springs == *damaged_count
}

fn char_to_spring(c: char) -> Result<Spring, Box<dyn Error>> {
    match c {
        '.' => Ok(Spring::Operational),
        '#' => Ok(Spring::Damaged),
        '?' => Ok(Spring::Unknown),
        _ => Err(INVALID_INPUT.into()),
    }
}

#[allow(dead_code)]
fn combinations1(input_line: &InputLine) -> i64 {
    let damaged_count_in_springs = input_line
        .springs
        .iter()
        .filter(|s| **s == Spring::Damaged)
        .count() as i64;
    let number_to_fit = input_line.damaged.iter().sum::<i64>() - damaged_count_in_springs;

    let unknown_refs = input_line.springs.iter().enumerate().filter_map(|(i, s)| {
        if *s == Spring::Unknown {
            Some(i)
        } else {
            None
        }
    });

    unknown_refs
        .combinations(number_to_fit as usize)
        .filter(|replacements| {
            let mut trial = input_line.springs.iter().cloned().collect::<Vec<_>>();
            replacements
                .iter()
                .for_each(|i| trial[*i] = Spring::Damaged);

            check(&trial, &input_line.damaged)
        })
        .count() as i64
}

fn combinations2(input_line: &InputLine) -> i64 {
    let mut results: HashMap<Parameters, i64> = HashMap::new();

    combinations_rec(
        &mut results,
        Parameters {
            springs: input_line.springs.clone(),
            damaged: input_line.damaged.clone(),
            current: None,
        },
    )
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct Parameters {
    springs: Vec<Spring>,
    damaged: Vec<i64>,
    current: Option<i64>,
}

fn combinations_rec(memoized: &mut HashMap<Parameters, i64>, parameters: Parameters) -> i64 {
    if let Some(result) = memoized.get(&parameters) {
        *result
    } else {
        let Parameters {
            ref springs,
            ref damaged,
            current,
        } = parameters;

        let result = match springs.get(0) {
            Some(Spring::Damaged) => combinations_rec(
                memoized,
                Parameters {
                    springs: springs[1..].to_vec(),
                    damaged: damaged.clone(),
                    current: current.map_or(Some(1), |c| Some(c + 1)),
                },
            ),

            Some(Spring::Unknown) => {
                let mut springs_operational = springs[1..].to_vec();
                springs_operational.insert(0, Spring::Operational);

                let mut springs_damaged = springs[1..].to_vec();
                springs_damaged.insert(0, Spring::Damaged);

                combinations_rec(
                    memoized,
                    Parameters {
                        springs: springs_operational,
                        damaged: damaged.clone(),
                        current,
                    },
                ) + combinations_rec(
                    memoized,
                    Parameters {
                        springs: springs_damaged,
                        damaged: damaged.clone(),
                        current,
                    },
                )
            }

            Some(Spring::Operational) => match (damaged.get(0), current) {
                (Some(count1), Some(count2)) => {
                    if *count1 == count2 {
                        combinations_rec(
                            memoized,
                            Parameters {
                                springs: springs[1..].to_vec(),
                                damaged: damaged[1..].to_vec(),
                                current: None,
                            },
                        )
                    } else {
                        0
                    }
                }
                (_, None) => combinations_rec(
                    memoized,
                    Parameters {
                        springs: springs[1..].to_vec(),
                        damaged: damaged.clone(),
                        current: None,
                    },
                ),
                _ => 0,
            },

            None => match (damaged.get(0), current) {
                (Some(count1), Some(count2)) => {
                    if *count1 == count2 && damaged.len() == 1 {
                        1
                    } else {
                        0
                    }
                }
                (None, None) => 1,
                _ => 0,
            },
        };

        memoized.insert(parameters, result);

        result
    }
}

fn parse_line(line: String) -> Result<InputLine, Box<dyn Error>> {
    let (springs_str, damaged_str) = line.split_once(" ").ok_or("Invalid input")?;
    let springs = springs_str
        .chars()
        .map(|c| char_to_spring(c))
        .collect::<Result<_, _>>()?;
    let damaged = damaged_str
        .split(",")
        .map(|s| s.parse::<i64>())
        .collect::<Result<Vec<_>, _>>()?;

    Ok(InputLine { springs, damaged })
}

#[cfg(test)]
mod day12 {
    use std::{
        error::Error,
        fs::File,
        io::{BufRead, BufReader},
    };

    use itertools::Itertools;

    use crate::{
        combinations1, combinations2, parse_line, repeat_five, solve1, solve2, InputLine, Spring,
    };

    const EXAMPLE1: &str = "\
        #.#.### 1,1,3\n\
        .#...#....###. 1,1,3\n\
        .#.###.#.###### 1,3,1,6\n\
        ####.#...#... 4,1,1\n\
        #....######..#####. 1,6,5\n\
        .###.##....# 3,2,1";

    fn example1() -> Vec<InputLine> {
        vec![
            InputLine {
                springs: vec![
                    Spring::Damaged,
                    Spring::Operational,
                    Spring::Damaged,
                    Spring::Operational,
                    Spring::Damaged,
                    Spring::Damaged,
                    Spring::Damaged,
                ],
                damaged: vec![1, 1, 3],
            },
            InputLine {
                springs: vec![
                    Spring::Operational,
                    Spring::Damaged,
                    Spring::Operational,
                    Spring::Operational,
                    Spring::Operational,
                    Spring::Damaged,
                    Spring::Operational,
                    Spring::Operational,
                    Spring::Operational,
                    Spring::Operational,
                    Spring::Damaged,
                    Spring::Damaged,
                    Spring::Damaged,
                    Spring::Operational,
                ],
                damaged: vec![1, 1, 3],
            },
            InputLine {
                springs: vec![
                    Spring::Operational,
                    Spring::Damaged,
                    Spring::Operational,
                    Spring::Damaged,
                    Spring::Damaged,
                    Spring::Damaged,
                    Spring::Operational,
                    Spring::Damaged,
                    Spring::Operational,
                    Spring::Damaged,
                    Spring::Damaged,
                    Spring::Damaged,
                    Spring::Damaged,
                    Spring::Damaged,
                    Spring::Damaged,
                ],
                damaged: vec![1, 3, 1, 6],
            },
            InputLine {
                springs: vec![
                    Spring::Damaged,
                    Spring::Damaged,
                    Spring::Damaged,
                    Spring::Damaged,
                    Spring::Operational,
                    Spring::Damaged,
                    Spring::Operational,
                    Spring::Operational,
                    Spring::Operational,
                    Spring::Damaged,
                    Spring::Operational,
                    Spring::Operational,
                    Spring::Operational,
                ],
                damaged: vec![4, 1, 1],
            },
            InputLine {
                springs: vec![
                    Spring::Damaged,
                    Spring::Operational,
                    Spring::Operational,
                    Spring::Operational,
                    Spring::Operational,
                    Spring::Damaged,
                    Spring::Damaged,
                    Spring::Damaged,
                    Spring::Damaged,
                    Spring::Damaged,
                    Spring::Damaged,
                    Spring::Operational,
                    Spring::Operational,
                    Spring::Damaged,
                    Spring::Damaged,
                    Spring::Damaged,
                    Spring::Damaged,
                    Spring::Damaged,
                    Spring::Operational,
                ],
                damaged: vec![1, 6, 5],
            },
            InputLine {
                springs: vec![
                    Spring::Operational,
                    Spring::Damaged,
                    Spring::Damaged,
                    Spring::Damaged,
                    Spring::Operational,
                    Spring::Damaged,
                    Spring::Damaged,
                    Spring::Operational,
                    Spring::Operational,
                    Spring::Operational,
                    Spring::Operational,
                    Spring::Damaged,
                ],
                damaged: vec![3, 2, 1],
            },
        ]
    }

    const LINE1: &str = "???.### 1,1,3";
    fn line1() -> InputLine {
        InputLine {
            springs: vec![
                Spring::Unknown,
                Spring::Unknown,
                Spring::Unknown,
                Spring::Operational,
                Spring::Damaged,
                Spring::Damaged,
                Spring::Damaged,
            ],
            damaged: vec![1, 1, 3],
        }
    }

    const LINE2: &str = ".??..??...?##. 1,1,3";
    fn line2() -> InputLine {
        InputLine {
            springs: vec![
                Spring::Operational,
                Spring::Unknown,
                Spring::Unknown,
                Spring::Operational,
                Spring::Operational,
                Spring::Unknown,
                Spring::Unknown,
                Spring::Operational,
                Spring::Operational,
                Spring::Operational,
                Spring::Unknown,
                Spring::Damaged,
                Spring::Damaged,
                Spring::Operational,
            ],
            damaged: vec![1, 1, 3],
        }
    }

    const LINE3: &str = "?#?#?#?#?#?#?#? 1,3,1,6";
    fn line3() -> InputLine {
        InputLine {
            springs: vec![
                Spring::Unknown,
                Spring::Damaged,
                Spring::Unknown,
                Spring::Damaged,
                Spring::Unknown,
                Spring::Damaged,
                Spring::Unknown,
                Spring::Damaged,
                Spring::Unknown,
                Spring::Damaged,
                Spring::Unknown,
                Spring::Damaged,
                Spring::Unknown,
                Spring::Damaged,
                Spring::Unknown,
            ],
            damaged: vec![1, 3, 1, 6],
        }
    }

    const LINE4: &str = "????.#...#... 4,1,1";
    fn line4() -> InputLine {
        InputLine {
            springs: vec![
                Spring::Unknown,
                Spring::Unknown,
                Spring::Unknown,
                Spring::Unknown,
                Spring::Operational,
                Spring::Damaged,
                Spring::Operational,
                Spring::Operational,
                Spring::Operational,
                Spring::Damaged,
                Spring::Operational,
                Spring::Operational,
                Spring::Operational,
            ],
            damaged: vec![4, 1, 1],
        }
    }

    const LINE5: &str = "????.######..#####. 1,6,5";
    fn line5() -> InputLine {
        InputLine {
            springs: vec![
                Spring::Unknown,
                Spring::Unknown,
                Spring::Unknown,
                Spring::Unknown,
                Spring::Operational,
                Spring::Damaged,
                Spring::Damaged,
                Spring::Damaged,
                Spring::Damaged,
                Spring::Damaged,
                Spring::Damaged,
                Spring::Operational,
                Spring::Operational,
                Spring::Damaged,
                Spring::Damaged,
                Spring::Damaged,
                Spring::Damaged,
                Spring::Damaged,
                Spring::Operational,
            ],
            damaged: vec![1, 6, 5],
        }
    }

    const LINE6: &str = "?###???????? 3,2,1";
    fn line6() -> InputLine {
        InputLine {
            springs: vec![
                Spring::Unknown,
                Spring::Damaged,
                Spring::Damaged,
                Spring::Damaged,
                Spring::Unknown,
                Spring::Unknown,
                Spring::Unknown,
                Spring::Unknown,
                Spring::Unknown,
                Spring::Unknown,
                Spring::Unknown,
                Spring::Unknown,
            ],
            damaged: vec![3, 2, 1],
        }
    }

    fn example2_str() -> String {
        format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            LINE1, LINE2, LINE3, LINE4, LINE5, LINE6
        )
    }

    fn example2() -> Vec<InputLine> {
        vec![line1(), line2(), line3(), line4(), line5(), line6()]
    }

    #[test]
    fn test_parse_example1() -> Result<(), Box<dyn Error>> {
        let parsed = EXAMPLE1
            .lines()
            .map(|line| parse_line(line.to_string()))
            .collect::<Result<Vec<_>, _>>()?;
        assert_eq!(parsed, example1());
        Ok(())
    }

    #[test]
    fn test_parse_example2() -> Result<(), Box<dyn Error>> {
        let parsed = example2_str()
            .lines()
            .map(|line| parse_line(line.to_string()))
            .collect::<Result<Vec<_>, _>>()?;
        assert_eq!(parsed, example2());
        Ok(())
    }

    #[test]
    fn test_combinations1_line1() {
        let input = line1();
        assert_eq!(combinations1(&input), 1);
    }

    #[test]
    fn test_combinations2_line1() {
        let input = line1();
        assert_eq!(combinations2(&input), 1);
    }

    #[test]
    fn test_combinations2_repeat_line1() {
        let input = repeat_five(&line1());
        assert_eq!(combinations2(&input), 1);
    }

    #[test]
    fn test_combinations1_line2() {
        let input = line2();
        assert_eq!(combinations1(&input), 4);
    }

    #[test]
    fn test_combinations2_line2() {
        let input = line2();
        assert_eq!(combinations2(&input), 4);
    }

    #[test]
    fn test_combinations2_repeat_line2() {
        let input = repeat_five(&line2());
        assert_eq!(combinations2(&input), 16384);
    }

    #[test]
    fn test_combinations1_line3() {
        let input = line3();
        assert_eq!(combinations1(&input), 1);
    }

    #[test]
    fn test_combinations2_line3() {
        let input = line3();
        assert_eq!(combinations2(&input), 1);
    }

    #[test]
    fn test_combinations2_repeat_line3() {
        let input = repeat_five(&line3());
        assert_eq!(combinations2(&input), 1);
    }

    #[test]
    fn test_combinations1_line4() {
        let input = line4();
        assert_eq!(combinations1(&input), 1);
    }

    #[test]
    fn test_combinations2_line4() {
        let input = line4();
        assert_eq!(combinations2(&input), 1);
    }

    #[test]
    fn test_combinations2_repeat_line4() {
        let input = repeat_five(&line4());
        assert_eq!(combinations2(&input), 16);
    }

    #[test]
    fn test_combinations1_line5() {
        let input = line5();
        assert_eq!(combinations1(&input), 4);
    }

    #[test]
    fn test_combinations2_line5() {
        let input = line5();
        assert_eq!(combinations2(&input), 4);
    }

    #[test]
    fn test_combinations2_repeat_line5() {
        let input = repeat_five(&line5());
        assert_eq!(combinations2(&input), 2500);
    }

    #[test]
    fn test_combinations1_line6() {
        let input = line6();
        assert_eq!(combinations1(&input), 10);
    }

    #[test]
    fn test_combinations2_line6() {
        let input = line6();
        assert_eq!(combinations2(&input), 10);
    }

    #[test]
    fn test_combinations2_repeat_line6() {
        let input = repeat_five(&line6());
        assert_eq!(combinations2(&input), 506250);
    }

    #[test]
    fn test_combinations1_input() -> Result<(), Box<dyn Error>> {
        let file = File::open("input")?;
        let reader = BufReader::new(file);
        let result = reader.lines().process_results(|itr| {
            itr.map(|line| parse_line(line))
                .process_results(|itr| solve1(itr))
        })??;

        assert_eq!(result, 7047);
        Ok(())
    }

    #[test]
    fn test_combinations2_input() -> Result<(), Box<dyn Error>> {
        let file = File::open("input")?;
        let reader = BufReader::new(file);
        let result = reader.lines().process_results(|itr| {
            itr.map(|line| parse_line(line))
                .process_results(|itr| solve2(itr))
        })??;

        assert_eq!(result, 17391848518844);
        Ok(())
    }
}
