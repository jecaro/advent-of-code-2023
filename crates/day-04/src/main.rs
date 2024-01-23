use itertools::Itertools;
use lib::{get_args, INVALID_INPUT};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    error::Error,
    io::{stdin, BufRead},
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
            let cards = stdin().lock().lines().map(|line| Card::from_str(&line?));

            let result = match arg.as_str() {
                "-1" => cards.process_results(|itr| solve1(itr))?,

                _ => cards.process_results(|itr| solve2(itr))?,
            }?;

            println!("{}", result)
        }
        _ => usage(prog_name),
    }
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
struct Card {
    id: u32,
    winning: HashSet<u32>,
    have: HashSet<u32>,
}

impl FromStr for Card {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let without_card = s.strip_prefix("Card").ok_or(INVALID_INPUT)?.trim_start();
        let (id_str, numbers) = without_card.split_once(":").ok_or(INVALID_INPUT)?;
        let id = id_str.parse::<u32>()?;

        let (winning_str, have_str) = numbers.split_once("|").ok_or(INVALID_INPUT)?;
        let winning = winning_str
            .split_whitespace()
            .map(|s| s.parse::<u32>())
            .collect::<Result<HashSet<_>, _>>()?;
        let have = have_str
            .split_whitespace()
            .map(|s| s.parse::<u32>())
            .collect::<Result<HashSet<_>, _>>()?;

        Ok(Self { id, winning, have })
    }
}

fn solve1(cards: impl Iterator<Item = Card>) -> Result<u32, Box<dyn Error>> {
    cards
        .map(|card| -> Result<u32, Box<dyn Error>> {
            let winning_in_have = u32::try_from(card.winning.intersection(&card.have).count())?;
            if winning_in_have == 0 {
                Ok(0)
            } else {
                Ok(2u32.pow(winning_in_have - 1))
            }
        })
        .sum()
}

fn solve2(cards: impl Iterator<Item = Card>) -> Result<u32, Box<dyn Error>> {
    let cards = cards.collect::<Vec<_>>();

    let mut count = 0;
    let mut queue: VecDeque<_> = (0..u32::try_from(cards.len())?).collect();
    let mut cache: HashMap<u32, u32> = HashMap::new();

    while let Some(card_id) = queue.pop_front() {
        let card = cards
            .get(usize::try_from(card_id)?)
            .ok_or(format!("Unable to find card {}", card_id))?;
        count += 1;

        let winning_in_have = if let Some(&cached) = cache.get(&card_id) {
            cached
        } else {
            let winning_in_have_ = u32::try_from(card.winning.intersection(&card.have).count())?;
            cache.insert(card_id, winning_in_have_);
            winning_in_have_
        };

        (card_id + 1..card_id + winning_in_have + 1).for_each(|id| {
            queue.push_back(id);
        });
    }

    Ok(count)
}

#[cfg(test)]
mod day04 {
    use itertools::Itertools;
    use std::{
        collections::HashSet,
        error::Error,
        fs::File,
        io::{BufRead, BufReader},
        str::FromStr,
    };

    use crate::{solve1, solve2, Card};

    const CARD1: &str = "Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53";
    fn card1() -> Card {
        Card {
            id: 1,
            winning: HashSet::from([41, 48, 83, 86, 17]),
            have: HashSet::from([83, 86, 6, 31, 17, 9, 48, 53]),
        }
    }

    const CARD2: &str = "Card 2: 13 32 20 16 61 | 61 30 68 82 17 32 24 19";
    fn card2() -> Card {
        Card {
            id: 2,
            winning: HashSet::from([13, 32, 20, 16, 61]),
            have: HashSet::from([61, 30, 68, 82, 17, 32, 24, 19]),
        }
    }

    const CARD3: &str = "Card 3:  1 21 53 59 44 | 69 82 63 72 16 21 14  1";
    fn card3() -> Card {
        Card {
            id: 3,
            winning: HashSet::from([1, 21, 53, 59, 44]),
            have: HashSet::from([69, 82, 63, 72, 16, 21, 14, 1]),
        }
    }

    const CARD4: &str = "Card 4: 41 92 73 84 69 | 59 84 76 51 58  5 54 83";
    fn card4() -> Card {
        Card {
            id: 4,
            winning: HashSet::from([41, 92, 73, 84, 69]),
            have: HashSet::from([59, 84, 76, 51, 58, 5, 54, 83]),
        }
    }

    const CARD5: &str = "Card 5: 87 83 26 28 32 | 88 30 70 12 93 22 82 36";
    fn card5() -> Card {
        Card {
            id: 5,
            winning: HashSet::from([87, 83, 26, 28, 32]),
            have: HashSet::from([88, 30, 70, 12, 93, 22, 82, 36]),
        }
    }

    const CARD6: &str = "Card 6: 31 18 13 56 72 | 74 77 10 23 35 67 36 11";
    fn card6() -> Card {
        Card {
            id: 6,
            winning: HashSet::from([31, 18, 13, 56, 72]),
            have: HashSet::from([74, 77, 10, 23, 35, 67, 36, 11]),
        }
    }

    fn cards() -> Vec<Card> {
        vec![card1(), card2(), card3(), card4(), card5(), card6()]
    }

    #[test]
    fn parse_single_card() -> Result<(), Box<dyn Error>> {
        assert_eq!(card1(), Card::from_str(CARD1)?,);
        assert_eq!(card2(), Card::from_str(CARD2)?,);
        assert_eq!(card3(), Card::from_str(CARD3)?,);
        assert_eq!(card4(), Card::from_str(CARD4)?,);
        assert_eq!(card5(), Card::from_str(CARD5)?,);
        assert_eq!(card6(), Card::from_str(CARD6)?,);
        Ok(())
    }

    #[test]
    fn parse_multiple_cards() -> Result<(), Box<dyn Error>> {
        let cards_str = format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            CARD1, CARD2, CARD3, CARD4, CARD5, CARD6
        );
        let result = cards_str
            .lines()
            .map(Card::from_str)
            .collect::<Result<Vec<_>, _>>()?;

        assert_eq!(cards(), result);
        Ok(())
    }

    #[test]
    fn example_solve1() -> Result<(), Box<dyn Error>> {
        assert_eq!(solve1(cards().into_iter())?, 13);
        Ok(())
    }

    #[test]
    fn example_solve2() -> Result<(), Box<dyn Error>> {
        assert_eq!(solve2(cards().into_iter())?, 30);
        Ok(())
    }

    #[test]
    fn input_solve1() -> Result<(), Box<dyn Error>> {
        let file = File::open("input")?;
        let reader = BufReader::new(file);
        let result = reader.lines().process_results(|itr| {
            itr.map(move |l| Card::from_str(&l))
                .process_results(|itr| solve1(itr))
        })???;

        assert_eq!(result, 23847);
        Ok(())
    }

    #[test]
    fn input_solve2() -> Result<(), Box<dyn Error>> {
        let file = File::open("input")?;
        let reader = BufReader::new(file);
        let result = reader.lines().process_results(|itr| {
            itr.map(move |l| Card::from_str(&l))
                .process_results(|itr| solve2(itr))
        })???;

        assert_eq!(result, 8570000);
        Ok(())
    }
}
