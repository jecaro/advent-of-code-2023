use lib::{get_args, INVALID_INPUT};
use std::{
    error::Error,
    io::{self, BufRead},
    process::exit,
    str::FromStr,
};

const BAG: Cubes = Cubes {
    red: 12,
    green: 13,
    blue: 14,
};

fn usage(prog_name: String) {
    println!("Usage: {} [-1|-2|-h]", prog_name);
    exit(0)
}

fn main() -> Result<(), Box<dyn Error>> {
    let (prog_name, args) = get_args()?;

    let games = io::stdin()
        .lock()
        .lines()
        .map(|line| Game::from_str(&line?));
    match args.get(0) {
        Some(arg) if arg == "-1" => {
            let result = solve1(&BAG, games);

            println!("{}", result?);
        }
        Some(arg) if arg == "-2" => {
            let result = solve2(games);

            println!("{}", result?);
        }
        _ => usage(prog_name),
    }
    Ok(())
}

#[derive(Debug, Default, PartialEq, Eq)]
struct Cubes {
    blue: u32,
    green: u32,
    red: u32,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct Game {
    id: u32,
    draws: Vec<Cubes>,
}

fn min(draws: &[Cubes]) -> u32 {
    let min = draws.iter().fold(Cubes::default(), |acc, draw| Cubes {
        blue: acc.blue.max(draw.blue),
        green: acc.green.max(draw.green),
        red: acc.red.max(draw.red),
    });

    power(&min)
}

fn solve2(
    games: impl Iterator<Item = Result<Game, Box<dyn Error>>>,
) -> Result<u32, Box<dyn Error>> {
    games
        .map(|game| -> Result<u32, Box<dyn Error>> { Ok(min(game?.draws.as_slice())) })
        .sum()
}

fn power(cube: &Cubes) -> u32 {
    cube.blue * cube.green * cube.red
}

fn draw_possible(bag: &Cubes, draw: &Cubes) -> bool {
    bag.blue >= draw.blue && bag.green >= draw.green && bag.red >= draw.red
}

fn game_possible(bag: &Cubes, game: &Game) -> bool {
    game.draws.iter().all(|draw| draw_possible(bag, draw))
}

fn solve1(
    bag: &Cubes,
    games: impl Iterator<Item = Result<Game, Box<dyn Error>>>,
) -> Result<u32, Box<dyn Error>> {
    games
        .filter_map(|game| match game {
            Ok(game) => {
                if game_possible(bag, &game) {
                    Some(Ok(game.id))
                } else {
                    None
                }
            }
            Err(e) => Some(Err(e)),
        })
        .sum()
}

impl FromStr for Game {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let without_game = s.strip_prefix("Game ").ok_or(INVALID_INPUT)?;
        let (id_str, draw_str) = without_game.split_once(":").ok_or(INVALID_INPUT)?;

        let id = id_str.parse::<u32>()?;
        let draws = draw_str
            .split(";")
            .map(|draw_str| Cubes::from_str(draw_str))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Game { id, draws })
    }
}

impl FromStr for Cubes {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut draw = Cubes::default();
        for count_color_str in s.split(",") {
            let (count_str, color_str) = count_color_str
                .trim()
                .split_once(" ")
                .ok_or(INVALID_INPUT)?;

            let count = count_str.parse::<u32>()?;
            match color_str {
                "blue" => draw.blue = count,
                "green" => draw.green = count,
                "red" => draw.red = count,
                _ => return Err(INVALID_INPUT.into()),
            }
        }

        Ok(draw)
    }
}

#[cfg(test)]
mod day02 {
    use std::{
        fs::File,
        io::{BufRead, BufReader},
        str::FromStr,
    };

    use crate::{solve1, solve2, Cubes, Game, BAG};

    const GAME_1_STR: &str = "Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green";
    fn game_1() -> Game {
        Game {
            id: 1,
            draws: vec![
                Cubes {
                    blue: 3,
                    green: 0,
                    red: 4,
                },
                Cubes {
                    blue: 6,
                    green: 2,
                    red: 1,
                },
                Cubes {
                    blue: 0,
                    green: 2,
                    red: 0,
                },
            ],
        }
    }

    const GAME_2_STR: &str = "Game 2: 1 blue, 2 green; 3 green, 4 blue, 1 red; 1 green, 1 blue";
    fn game_2() -> Game {
        Game {
            id: 2,
            draws: vec![
                Cubes {
                    blue: 1,
                    green: 2,
                    red: 0,
                },
                Cubes {
                    blue: 4,
                    green: 3,
                    red: 1,
                },
                Cubes {
                    blue: 1,
                    green: 1,
                    red: 0,
                },
            ],
        }
    }

    const GAME_3_STR: &str =
        "Game 3: 8 green, 6 blue, 20 red; 5 blue, 4 red, 13 green; 5 green, 1 red";
    fn game_3() -> Game {
        Game {
            id: 3,
            draws: vec![
                Cubes {
                    blue: 6,
                    green: 8,
                    red: 20,
                },
                Cubes {
                    blue: 5,
                    green: 13,
                    red: 4,
                },
                Cubes {
                    blue: 0,
                    green: 5,
                    red: 1,
                },
            ],
        }
    }

    const GAME_4_STR: &str =
        "Game 4: 1 green, 3 red, 6 blue; 3 green, 6 red; 3 green, 15 blue, 14 red";
    fn game_4() -> Game {
        Game {
            id: 4,
            draws: vec![
                Cubes {
                    blue: 6,
                    green: 1,
                    red: 3,
                },
                Cubes {
                    blue: 0,
                    green: 3,
                    red: 6,
                },
                Cubes {
                    blue: 15,
                    green: 3,
                    red: 14,
                },
            ],
        }
    }

    const GAME_5_STR: &str = "Game 5: 6 red, 1 blue, 3 green; 2 blue, 1 red, 2 green";
    fn game_5() -> Game {
        Game {
            id: 5,
            draws: vec![
                Cubes {
                    blue: 1,
                    green: 3,
                    red: 6,
                },
                Cubes {
                    blue: 2,
                    green: 2,
                    red: 1,
                },
            ],
        }
    }

    fn games() -> Vec<Game> {
        vec![game_1(), game_2(), game_3(), game_4(), game_5()]
    }

    #[test]
    fn parse_single_game() {
        assert_eq!(game_1(), Game::from_str(GAME_1_STR).unwrap(),);
        assert_eq!(game_2(), Game::from_str(GAME_2_STR).unwrap(),);
        assert_eq!(game_3(), Game::from_str(GAME_3_STR).unwrap(),);
        assert_eq!(game_4(), Game::from_str(GAME_4_STR).unwrap(),);
        assert_eq!(game_5(), Game::from_str(GAME_5_STR).unwrap(),);
    }

    #[test]
    fn parse_multiple_games() {
        let games_str = format!(
            "{}\n{}\n{}\n{}\n{}",
            GAME_1_STR, GAME_2_STR, GAME_3_STR, GAME_4_STR, GAME_5_STR
        );

        assert_eq!(
            games(),
            games_str
                .lines()
                .map(Game::from_str)
                .collect::<Result<Vec<_>, _>>()
                .unwrap(),
        );
    }

    #[test]
    fn example_solve1() {
        assert_eq!(solve1(&BAG, games().into_iter().map(Ok)).unwrap(), 8);
    }

    #[test]
    fn example_solve2() {
        assert_eq!(solve2(games().into_iter().map(Ok)).unwrap(), 2286);
    }

    #[test]
    fn input_solve1() {
        let file = File::open("input").unwrap();
        let reader = BufReader::new(file);
        let games = reader.lines().map(|l| Game::from_str(&l?));

        assert_eq!(solve1(&BAG, games).unwrap(), 2439);
    }

    #[test]
    fn input_solve2() {
        let file = File::open("input").unwrap();
        let reader = BufReader::new(file);
        let games = reader.lines().map(|l| Game::from_str(&l?));

        assert_eq!(solve2(games).unwrap(), 63711);
    }
}
