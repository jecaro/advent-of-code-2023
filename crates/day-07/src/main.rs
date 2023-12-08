use lib::{get_args, INVALID_INPUT};
use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    error::Error,
    io::{self, BufRead},
    iter::zip,
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
            let solve = match arg.as_str() {
                "-1" => solve1,
                _ => solve2,
            };

            let input = io::stdin().lock().lines().collect::<Result<Vec<_>, _>>()?;
            let cards = input
                .iter()
                .map(|x| parse_hand_and_bid(x))
                .collect::<Result<Vec<_>, _>>()?;

            let result = solve(cards);

            println!("{}", result)
        }
        _ => usage(prog_name),
    }
    Ok(())
}

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Clone, Copy, Hash)]
enum Card {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Height,
    Nine,
    T,
    J,
    Q,
    K,
    A,
}

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Clone, Copy)]
enum Type {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    FullHouse,
    FourOfAKind,
    FiveOfAKind,
}

type Hand = [Card; 5];

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Clone)]
struct HandAndBid {
    hand: Hand,
    bid: u32,
}

fn parse_card(c: char) -> Result<Card, Box<dyn Error>> {
    match c {
        '2' => Ok(Card::Two),
        '3' => Ok(Card::Three),
        '4' => Ok(Card::Four),
        '5' => Ok(Card::Five),
        '6' => Ok(Card::Six),
        '7' => Ok(Card::Seven),
        '8' => Ok(Card::Height),
        '9' => Ok(Card::Nine),
        'T' => Ok(Card::T),
        'J' => Ok(Card::J),
        'Q' => Ok(Card::Q),
        'K' => Ok(Card::K),
        'A' => Ok(Card::A),
        _ => Err(INVALID_INPUT.into()),
    }
}

fn parse_hand(s: &str) -> Result<Hand, Box<dyn Error>> {
    s.chars()
        .take(5)
        .map(parse_card)
        .collect::<Result<Vec<Card>, Box<dyn Error>>>()?
        .as_slice()
        .try_into()
        .map(|x: &Hand| *x)
        .map_err(|e| e.into())
}

fn type1(hand: &Hand) -> Type {
    let cards_counts = hand.iter().fold(HashMap::new(), |mut acc, x| {
        *acc.entry(x).or_insert(0) += 1;
        acc
    });

    let mut counts = cards_counts.values().collect::<Vec<_>>();
    counts.sort();
    match counts.as_slice() {
        [1, 1, 1, 1, 1] => Type::HighCard,
        [1, 1, 1, 2] => Type::OnePair,
        [1, 2, 2] => Type::TwoPair,
        [1, 1, 3] => Type::ThreeOfAKind,
        [2, 3] => Type::FullHouse,
        [1, 4] => Type::FourOfAKind,
        [5] => Type::FiveOfAKind,
        _ => unreachable!(),
    }
}

fn type2(hand: &Hand) -> Type {
    let non_jocker_cards = hand
        .iter()
        .cloned()
        .filter(|card| *card != Card::J)
        .collect::<HashSet<Card>>();

    let joker_use = non_jocker_cards.into_iter().map(|x| {
        let new_hand = hand.map(|y| if y == Card::J { x } else { y });
        type1(&new_hand)
    });

    [type1(hand)]
        .iter()
        .cloned()
        .chain(joker_use)
        .max()
        .unwrap()
}

fn parse_hand_and_bid(s: &str) -> Result<HandAndBid, Box<dyn Error>> {
    let (hand_str, bid_str) = s.split_once(' ').ok_or(INVALID_INPUT)?;
    let hand = parse_hand(hand_str)?;
    let bid = bid_str.parse::<u32>()?;

    Ok(HandAndBid { hand, bid })
}

fn compare_hands(
    hand1: &Hand,
    hand2: &Hand,
    type_: fn(&Hand) -> Type,
    cmp: fn(&Card, &Card) -> Ordering,
) -> Ordering {
    let type1_ = type_(hand1);
    let type2_ = type_(hand2);

    if type1_ == type2_ {
        zip(hand1.iter(), hand2.iter())
            .find_map(|(x, y)| match cmp(x, y) {
                Ordering::Equal => None,
                x => Some(x),
            })
            .unwrap_or(Ordering::Equal)
    } else {
        type1_.cmp(&type2_)
    }
}

fn card_level(card: &Card) -> u32 {
    match card {
        Card::J => 1,
        Card::Two => 2,
        Card::Three => 3,
        Card::Four => 4,
        Card::Five => 5,
        Card::Six => 6,
        Card::Seven => 7,
        Card::Height => 8,
        Card::Nine => 9,
        Card::T => 10,
        Card::Q => 11,
        Card::K => 12,
        Card::A => 13,
    }
}

fn cmp2(card1: &Card, card2: &Card) -> Ordering {
    card_level(card1).cmp(&card_level(card2))
}

fn solve1(mut hand_and_bids: Vec<HandAndBid>) -> u32 {
    hand_and_bids.sort_by(|x, y| compare_hands(&x.hand, &y.hand, type1, |x, y| x.cmp(y)));

    zip(hand_and_bids.iter(), 1..).map(|(x, y)| x.bid * y).sum()
}

fn solve2(mut hand_and_bids: Vec<HandAndBid>) -> u32 {
    hand_and_bids.sort_by(|x, y| compare_hands(&x.hand, &y.hand, type2, cmp2));

    zip(hand_and_bids.iter(), 1..).map(|(x, y)| x.bid * y).sum()
}

#[cfg(test)]
mod day07 {

    use std::{
        fs::File,
        io::{BufRead, BufReader},
    };

    use crate::{parse_hand_and_bid, solve1, solve2, Card, HandAndBid};

    const EXAMPLE: &str = "\
        32T3K 765\n\
        T55J5 684\n\
        KK677 28\n\
        KTJJT 220\n\
        QQQJA 483";

    fn example() -> Vec<HandAndBid> {
        vec![
            HandAndBid {
                hand: [Card::Three, Card::Two, Card::T, Card::Three, Card::K],
                bid: 765,
            },
            HandAndBid {
                hand: [Card::T, Card::Five, Card::Five, Card::J, Card::Five],
                bid: 684,
            },
            HandAndBid {
                hand: [Card::K, Card::K, Card::Six, Card::Seven, Card::Seven],
                bid: 28,
            },
            HandAndBid {
                hand: [Card::K, Card::T, Card::J, Card::J, Card::T],
                bid: 220,
            },
            HandAndBid {
                hand: [Card::Q, Card::Q, Card::Q, Card::J, Card::A],
                bid: 483,
            },
        ]
    }

    #[test]
    fn parse_example() {
        let parsed_example = EXAMPLE
            .lines()
            .map(parse_hand_and_bid)
            .collect::<Result<Vec<HandAndBid>, _>>()
            .unwrap();
        assert_eq!(parsed_example, example());
    }

    #[test]
    fn solve1_example() {
        assert_eq!(solve1(example()), 6440);
    }

    #[test]
    fn solve2_example() {
        assert_eq!(solve2(example()), 5905);
    }

    #[test]
    fn input_solve1() {
        let file = File::open("input").unwrap();
        let reader = BufReader::new(file);
        let cards = reader
            .lines()
            .map(|x| parse_hand_and_bid(&x?))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(solve1(cards), 249483956);
    }

    #[test]
    fn input_solve2() {
        let file = File::open("input").unwrap();
        let reader = BufReader::new(file);
        let cards = reader
            .lines()
            .map(|x| parse_hand_and_bid(&x?))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(solve2(cards), 252137472);
    }
}
