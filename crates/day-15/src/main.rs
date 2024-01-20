use lib::get_args;
use std::{
    array::from_fn,
    collections::HashMap,
    error::Error,
    io::{read_to_string, stdin},
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
            let input = read_to_string(stdin())?;
            let result = if arg == "-1" {
                solve1(&input)
            } else {
                solve2(&input)?
            };

            println!("{}", result);
        }
        _ => usage(prog_name),
    }
    Ok(())
}

fn hash(s: &str) -> u8 {
    s.chars().filter(|c| *c != '\n').fold(0, |acc, c| {
        let acc = acc + c as u64;
        let acc = acc * 17;
        let acc = acc % 256;
        acc
    }) as u8
}

fn solve1(s: &str) -> u64 {
    s.split(',').map(|x| hash(x) as u64).sum()
}

#[derive(Debug, PartialEq, Eq)]
struct Step {
    label: String,
    operation: Operation,
}

#[derive(Debug, PartialEq, Eq)]
enum Operation {
    Remove,
    Focal(u64),
}

impl FromStr for Operation {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "-" => Ok(Operation::Remove),
            _ => {
                let s = s.trim_start_matches('=');
                Ok(Operation::Focal(s.parse::<u64>()?))
            }
        }
    }
}

impl FromStr for Step {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.find(|c| c == '=' || c == '-') {
            None => Err("Missing '=' or '-'")?,
            Some(index) => {
                let (label, operation) = s.split_at(index);
                Ok(Step {
                    label: label.to_string(),
                    operation: operation.parse::<Operation>()?,
                })
            }
        }
    }
}

#[derive(Debug)]
struct SlotAndFocal {
    slot: u64,
    focal: u64,
}

fn solve2(s: &str) -> Result<u64, Box<dyn Error>> {
    let mut lenses: [HashMap<String, SlotAndFocal>; 256] = from_fn(|_| HashMap::new());

    s.chars()
        .filter(|c| *c != '\n')
        .collect::<String>()
        .split(',')
        .try_for_each(|s| -> Result<_, Box<dyn Error>> {
            let step = s.parse::<Step>()?;
            let hash = hash(&step.label);
            lenses
                .get_mut(hash as usize)
                .map(|lens| match step.operation {
                    Operation::Remove => {
                        match lens.get(&step.label) {
                            None => {}
                            Some(slot_and_focal) => {
                                let slot = slot_and_focal.slot;
                                lens.iter_mut()
                                    .filter(|(_, slot_and_focal)| slot_and_focal.slot > slot)
                                    .for_each(|(_, slot_and_focal)| {
                                        slot_and_focal.slot -= 1;
                                    });
                            }
                        }
                        lens.remove(&step.label);
                    }
                    Operation::Focal(focal) => {
                        let new_slot = lens.len() as u64;
                        lens.entry(step.label)
                            .and_modify(|slot_and_focal| {
                                slot_and_focal.focal = focal;
                            })
                            .or_insert(SlotAndFocal {
                                slot: new_slot,
                                focal,
                            });
                    }
                });
            Ok(())
        })?;

    Ok(lenses
        .iter()
        .enumerate()
        .map(|(box_, lens)| {
            lens.iter()
                .map(|(_, SlotAndFocal { slot, focal })| {
                    let box_ = box_ as u64 + 1;
                    let slot = *slot + 1;
                    box_ * slot * focal
                })
                .sum::<u64>()
        })
        .sum())
}

#[cfg(test)]
mod day15 {
    use std::{error::Error, fs::read_to_string};

    use crate::{hash, solve1, solve2, Operation, Step};

    const EXAMPLE: &str = "rn=1,cm-,qp=3,cm=2,qp-,pc=4,ot=9,ab=5,pc-,pc=6,ot=7";

    #[test]
    fn test_hash_simple() {
        assert_eq!(hash("rn=1"), 30);
        assert_eq!(hash("cm-"), 253);
        assert_eq!(hash("qp=3"), 97);
        assert_eq!(hash("cm=2"), 47);
        assert_eq!(hash("qp-"), 14);
        assert_eq!(hash("pc=4"), 180);
        assert_eq!(hash("ot=9"), 9);
        assert_eq!(hash("ab=5"), 197);
        assert_eq!(hash("pc-"), 48);
        assert_eq!(hash("pc=6"), 214);
        assert_eq!(hash("ot=7"), 231);
    }

    #[test]
    fn test_solve1_example() {
        assert_eq!(solve1(EXAMPLE), 1320);
    }

    #[test]
    fn test_solve1_input() -> Result<(), Box<dyn Error>> {
        let input = read_to_string("input")?;
        assert_eq!(solve1(&input), 507769);
        Ok(())
    }

    #[test]
    fn test_solve2_example() -> Result<(), Box<dyn Error>> {
        assert_eq!(solve2(EXAMPLE)?, 145);
        Ok(())
    }
    #[test]
    fn test_solve2_input() -> Result<(), Box<dyn Error>> {
        let input = read_to_string("input")?;
        assert_eq!(solve2(&input)?, 269747);
        Ok(())
    }

    #[test]
    fn test_parse() -> Result<(), Box<dyn Error>> {
        assert_eq!(
            "rn=1".parse::<Step>()?,
            Step {
                label: "rn".to_string(),
                operation: Operation::Focal(1)
            }
        );
        assert_eq!(
            "cm-".parse::<Step>()?,
            Step {
                label: "cm".to_string(),
                operation: Operation::Remove
            }
        );
        assert_eq!(
            "qp=3".parse::<Step>()?,
            Step {
                label: "qp".to_string(),
                operation: Operation::Focal(3)
            }
        );
        assert_eq!(
            "cm=2".parse::<Step>()?,
            Step {
                label: "cm".to_string(),
                operation: Operation::Focal(2)
            }
        );
        assert_eq!(
            "qp-".parse::<Step>()?,
            Step {
                label: "qp".to_string(),
                operation: Operation::Remove
            }
        );
        assert_eq!(
            "pc=4".parse::<Step>()?,
            Step {
                label: "pc".to_string(),
                operation: Operation::Focal(4)
            }
        );
        assert_eq!(
            "ot=9".parse::<Step>()?,
            Step {
                label: "ot".to_string(),
                operation: Operation::Focal(9)
            }
        );
        assert_eq!(
            "ab=5".parse::<Step>()?,
            Step {
                label: "ab".to_string(),
                operation: Operation::Focal(5)
            }
        );
        assert_eq!(
            "pc-".parse::<Step>()?,
            Step {
                label: "pc".to_string(),
                operation: Operation::Remove
            }
        );
        assert_eq!(
            "pc=6".parse::<Step>()?,
            Step {
                label: "pc".to_string(),
                operation: Operation::Focal(6)
            }
        );
        assert_eq!(
            "ot=7".parse::<Step>()?,
            Step {
                label: "ot".to_string(),
                operation: Operation::Focal(7)
            }
        );
        Ok(())
    }
}
