use itertools::{process_results, Itertools};
use lib::{get_args, INVALID_INPUT};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::{
    error::Error,
    io::{self, BufRead},
    process::exit,
    str::FromStr,
};

fn usage(prog_name: String) {
    println!("Usage: {} [-1|-2_1|-2_2|-h]", prog_name);
    exit(0)
}

fn main() -> Result<(), Box<dyn Error>> {
    let (prog_name, args) = get_args()?;

    match args.get(0) {
        Some(arg) if arg == "-1" || arg == "-2_1" || arg == "-2_2" => {
            let input = process_results(io::stdin().lock().lines(), |itr| parse_input(itr))??;
            let solve: fn(_) -> Result<u32, Box<dyn Error>> = match arg.as_str() {
                "-1" => solve1,
                "-2_1" => solve2_brut_force,
                _ => solve2_brut_force_reverse,
            };

            let result = solve(input)?;

            println!("{}", result)
        }
        _ => usage(prog_name),
    }
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
struct Input {
    seeds: Vec<Seed>,
    garden_maps: Vec<GardenMap>,
}

#[derive(Debug, PartialEq, Eq)]
struct Seed {
    from: u32,
    len: u32,
}

#[derive(Debug, PartialEq, Eq)]
struct GardenMap {
    from: String,
    to: String,
    garden_ranges: Vec<GardenRange>,
}

#[derive(Debug, PartialEq, Eq)]
struct GardenRange {
    destination: u32,
    source: u32,
    length: u32,
}

fn parse_seeds(s: &str) -> Result<Vec<Seed>, Box<dyn Error>> {
    s.strip_prefix("seeds:")
        .ok_or(INVALID_INPUT)?
        .split_whitespace()
        .map(|s| s.parse::<u32>())
        .chunks(2)
        .into_iter()
        .map(|seed| {
            if let [from, len] = seed.collect::<Result<Vec<_>, _>>()?[..] {
                Ok(Seed { from, len })
            } else {
                Err(INVALID_INPUT.into())
            }
        })
        .collect::<Result<Vec<_>, _>>()
}

fn parse_input(itr: impl Iterator<Item = String>) -> Result<Input, Box<dyn Error>> {
    let mut chunks = itr.batching(|itr| {
        let non_empty_lines = itr.take_while(|line| line != "");

        non_empty_lines.reduce(|acc, line| acc + "\n" + &line)
    });

    let first_chunk = chunks.next().ok_or(INVALID_INPUT)?;
    let seeds = parse_seeds(&first_chunk)?;

    let garden_maps = chunks
        .map(|chunk| chunk.parse::<GardenMap>())
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Input { seeds, garden_maps })
}

impl FromStr for GardenMap {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        let first_line = lines.next();

        let words = first_line
            .ok_or(INVALID_INPUT)?
            .split_whitespace()
            .collect::<Vec<_>>();
        let (from, to) = words
            .get(0)
            .ok_or(INVALID_INPUT)?
            .split_once("-to-")
            .ok_or(INVALID_INPUT)?;

        let garden_ranges = lines
            .map(|line| GardenRange::from_str(line))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            from: from.to_string(),
            to: to.to_string(),
            garden_ranges,
        })
    }
}

impl FromStr for GardenRange {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let words = s
            .split_whitespace()
            .map(|s| s.parse::<u32>())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            destination: *words.get(0).ok_or(INVALID_INPUT)?,
            source: *words.get(1).ok_or(INVALID_INPUT)?,
            length: *words.get(2).ok_or(INVALID_INPUT)?,
        })
    }
}

fn solve1(input: Input) -> Result<u32, Box<dyn Error>> {
    input
        .seeds
        .iter()
        .flat_map(|seed| [seed.from, seed.len])
        .map(|seed| {
            input.garden_maps.iter().fold(seed, |acc, garden_map| {
                // go through all the garden ranges and stop when we find the one that contains the
                // seed
                let mapped = garden_map.garden_ranges.iter().find_map(|garden_range| {
                    if acc >= garden_range.source
                        && (acc - garden_range.source) < garden_range.length
                    {
                        let offset = acc - garden_range.source;
                        Some(garden_range.destination + offset)
                    } else {
                        None
                    }
                });

                mapped.unwrap_or(acc)
            })
        })
        .min()
        .ok_or("Empty vector".into())
}

fn solve2_brut_force(input: Input) -> Result<u32, Box<dyn Error>> {
    input
        .seeds
        .into_par_iter()
        .flat_map(|seed| (seed.from..seed.from + seed.len))
        .map(|seed| {
            input.garden_maps.iter().fold(seed, |acc, garden_map| {
                // go through all the garden ranges and stop when we find the one that contains the
                // seed
                let mapped = garden_map.garden_ranges.iter().find_map(|garden_range| {
                    if acc >= garden_range.source
                        && (acc - garden_range.source) < garden_range.length
                    {
                        let offset = acc - garden_range.source;
                        Some(garden_range.destination + offset)
                    } else {
                        None
                    }
                });

                mapped.unwrap_or(acc)
            })
        })
        .min()
        .ok_or("Empty vector".into())
}

fn solve2_brut_force_reverse(input: Input) -> Result<u32, Box<dyn Error>> {
    (0..)
        .into_iter()
        .find(|location| {
            let soil = input
                .garden_maps
                .iter()
                .rev()
                .fold(location.clone(), |acc, garden_map| {
                    garden_map
                        .garden_ranges
                        .iter()
                        .find_map(|garden_range| {
                            if acc >= garden_range.destination
                                && (acc - garden_range.destination) < garden_range.length
                            {
                                let offset = acc - garden_range.destination;
                                Some(garden_range.source + offset)
                            } else {
                                None
                            }
                        })
                        .unwrap_or(acc)
                });
            let seed = input.seeds.iter().find(|seed_range| {
                soil >= seed_range.from && (soil - seed_range.from) < seed_range.len
            });
            seed.is_some()
        })
        .ok_or("Not found".into())
}

#[cfg(test)]
mod day05 {
    use std::{
        fs::File,
        io::{BufRead, BufReader},
        str::FromStr,
    };

    use itertools::process_results;

    use crate::{
        parse_input, parse_seeds, solve1, solve2_brut_force, solve2_brut_force_reverse, GardenMap,
        GardenRange, Input, Seed,
    };

    const SEEDS: &str = "seeds: 79 14 55 13";
    fn seeds() -> Vec<Seed> {
        vec![Seed { from: 79, len: 14 }, Seed { from: 55, len: 13 }]
    }

    const GARDEN_MAP1: &str = "\
        seed-to-soil map:\n\
        50 98 2\n\
        52 50 48";
    fn garden_map1() -> GardenMap {
        GardenMap {
            from: "seed".to_string(),
            to: "soil".to_string(),
            garden_ranges: vec![
                GardenRange {
                    destination: 50,
                    source: 98,
                    length: 2,
                },
                GardenRange {
                    destination: 52,
                    source: 50,
                    length: 48,
                },
            ],
        }
    }

    const GARDEN_MAP2: &str = "\
        soil-to-fertilizer map:\n\
        0 15 37\n\
        37 52 2\n\
        39 0 15";
    fn garden_map2() -> GardenMap {
        GardenMap {
            from: "soil".to_string(),
            to: "fertilizer".to_string(),
            garden_ranges: vec![
                GardenRange {
                    destination: 0,
                    source: 15,
                    length: 37,
                },
                GardenRange {
                    destination: 37,
                    source: 52,
                    length: 2,
                },
                GardenRange {
                    destination: 39,
                    source: 0,
                    length: 15,
                },
            ],
        }
    }

    const GARDEN_MAP3: &str = "\
        fertilizer-to-water map:\n\
        49 53 8\n\
        0 11 42\n\
        42 0 7\n\
        57 7 4";
    fn garden_map3() -> GardenMap {
        GardenMap {
            from: "fertilizer".to_string(),
            to: "water".to_string(),
            garden_ranges: vec![
                GardenRange {
                    destination: 49,
                    source: 53,
                    length: 8,
                },
                GardenRange {
                    destination: 0,
                    source: 11,
                    length: 42,
                },
                GardenRange {
                    destination: 42,
                    source: 0,
                    length: 7,
                },
                GardenRange {
                    destination: 57,
                    source: 7,
                    length: 4,
                },
            ],
        }
    }

    const GARDEN_MAP4: &str = "\
        water-to-light map:\n\
        88 18 7\n\
        18 25 70";
    fn garden_map4() -> GardenMap {
        GardenMap {
            from: "water".to_string(),
            to: "light".to_string(),
            garden_ranges: vec![
                GardenRange {
                    destination: 88,
                    source: 18,
                    length: 7,
                },
                GardenRange {
                    destination: 18,
                    source: 25,
                    length: 70,
                },
            ],
        }
    }

    const GARDEN_MAP5: &str = "\
        light-to-temperature map:\n\
        45 77 23\n\
        81 45 19\n\
        68 64 13";
    fn garden_map5() -> GardenMap {
        GardenMap {
            from: "light".to_string(),
            to: "temperature".to_string(),
            garden_ranges: vec![
                GardenRange {
                    destination: 45,
                    source: 77,
                    length: 23,
                },
                GardenRange {
                    destination: 81,
                    source: 45,
                    length: 19,
                },
                GardenRange {
                    destination: 68,
                    source: 64,
                    length: 13,
                },
            ],
        }
    }

    const GARDEN_MAP6: &str = "\
        temperature-to-humidity map:\n\
        0 69 1\n\
        1 0 69";
    fn garden_map6() -> GardenMap {
        GardenMap {
            from: "temperature".to_string(),
            to: "humidity".to_string(),
            garden_ranges: vec![
                GardenRange {
                    destination: 0,
                    source: 69,
                    length: 1,
                },
                GardenRange {
                    destination: 1,
                    source: 0,
                    length: 69,
                },
            ],
        }
    }

    const GARDEN_MAP7: &str = "\
        humidity-to-location map:\n\
        60 56 37\n\
        56 93 4";
    fn garden_map7() -> GardenMap {
        GardenMap {
            from: "humidity".to_string(),
            to: "location".to_string(),
            garden_ranges: vec![
                GardenRange {
                    destination: 60,
                    source: 56,
                    length: 37,
                },
                GardenRange {
                    destination: 56,
                    source: 93,
                    length: 4,
                },
            ],
        }
    }

    fn input1() -> Input {
        Input {
            seeds: seeds(),
            garden_maps: vec![
                garden_map1(),
                garden_map2(),
                garden_map3(),
                garden_map4(),
                garden_map5(),
                garden_map6(),
                garden_map7(),
            ],
        }
    }

    fn input_str() -> String {
        format!(
            "{}\n\n{}\n\n{}\n\n{}\n\n{}\n\n{}\n\n{}\n\n{}",
            SEEDS,
            GARDEN_MAP1,
            GARDEN_MAP2,
            GARDEN_MAP3,
            GARDEN_MAP4,
            GARDEN_MAP5,
            GARDEN_MAP6,
            GARDEN_MAP7,
        )
    }

    #[test]
    fn parse_seeds_() {
        assert_eq!(seeds(), parse_seeds(SEEDS).unwrap());
    }

    #[test]
    fn parse_single_garden_map() {
        assert_eq!(garden_map1(), GardenMap::from_str(GARDEN_MAP1).unwrap());
        assert_eq!(garden_map2(), GardenMap::from_str(GARDEN_MAP2).unwrap());
        assert_eq!(garden_map3(), GardenMap::from_str(GARDEN_MAP3).unwrap());
        assert_eq!(garden_map4(), GardenMap::from_str(GARDEN_MAP4).unwrap());
        assert_eq!(garden_map5(), GardenMap::from_str(GARDEN_MAP5).unwrap());
        assert_eq!(garden_map6(), GardenMap::from_str(GARDEN_MAP6).unwrap());
        assert_eq!(garden_map7(), GardenMap::from_str(GARDEN_MAP7).unwrap());
    }

    #[test]
    fn parse_input_() {
        assert_eq!(
            input1(),
            parse_input(input_str().lines().map(|s| s.to_string())).unwrap()
        );
    }

    #[test]
    fn example_solve1() {
        assert_eq!(solve1(input1()).unwrap(), 35);
    }

    #[test]
    fn example_solve2_brut_force() {
        assert_eq!(solve2_brut_force(input1()).unwrap(), 46);
    }

    #[test]
    fn example_solve2_brut_force_reverse() {
        assert_eq!(solve2_brut_force_reverse(input1()).unwrap(), 46);
    }

    #[test]
    fn input_solve1() {
        let file = File::open("input").unwrap();
        let reader = BufReader::new(file);
        let input = process_results(reader.lines(), |itr| parse_input(itr))
            .unwrap()
            .unwrap();

        assert_eq!(solve1(input).unwrap(), 382895070);
    }

    #[test]
    fn input_solve2_brut_force_reverse() {
        let file = File::open("input").unwrap();
        let reader = BufReader::new(file);
        let input = process_results(reader.lines(), |itr| parse_input(itr))
            .unwrap()
            .unwrap();

        assert_eq!(solve2_brut_force_reverse(input).unwrap(), 17729182);
    }

    // This takes too much time for tests
    // #[test]
    // fn input_solve2() {
    //     let file = File::open("input").unwrap();
    //     let reader = BufReader::new(file);
    //     let input = process_results(reader.lines(), |itr| parse_input(itr))
    //         .unwrap()
    //         .unwrap();

    //     assert_eq!(solve2_brut_force(input).unwrap(), 17729182);
    // }
}
