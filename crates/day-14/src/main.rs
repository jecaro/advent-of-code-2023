use itertools::Itertools;
use lib::get_args;
use std::{
    cmp::Ordering,
    collections::HashMap,
    error::Error,
    io::{stdin, BufRead},
    process::exit,
};

fn usage(prog_name: String) {
    println!("Usage: {} [-1|-2|-h]", prog_name);
    exit(0)
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
enum Cell {
    Rounded,
    Cube,
    Empty,
}

fn main() -> Result<(), Box<dyn Error>> {
    let (prog_name, args) = get_args()?;

    match args.get(0) {
        Some(arg) if arg == "-1" || arg == "-2" => {
            let cells = stdin()
                .lock()
                .lines()
                .process_results(|itr| -> Result<_, Box<dyn Error>> { parse(itr) })??;

            let result = if arg == "-1" {
                solve1(cells)?
            } else {
                solve2(cells)?
            };

            println!("{}", result);
        }
        _ => usage(prog_name),
    }
    Ok(())
}

fn solve1(cells: Vec<Vec<Cell>>) -> Result<i32, Box<dyn Error>> {
    transpose(cells).map(|cells| count(&tilt_left(cells)))
}

fn parse(itr: impl Iterator<Item = String>) -> Result<Vec<Vec<Cell>>, Box<dyn Error>> {
    itr.map(|line| -> Result<Vec<Cell>, Box<dyn Error>> {
        line.chars()
            .map(|c| match c {
                'O' => Ok(Cell::Rounded),
                '#' => Ok(Cell::Cube),
                '.' => Ok(Cell::Empty),
                _ => Err("Invalid character".into()),
            })
            .collect::<Result<Vec<_>, _>>()
    })
    .collect::<Result<Vec<_>, _>>()
}

fn transpose(cells: Vec<Vec<Cell>>) -> Result<Vec<Vec<Cell>>, Box<dyn Error>> {
    (0..cells.len())
        .map(|i| {
            cells
                .iter()
                .map(|line| line.get(i).map(|c| c.clone()).ok_or("Vec too small".into()))
                .collect::<Result<Vec<_>, Box<dyn Error>>>()
        })
        .collect::<Result<Vec<_>, Box<dyn Error>>>()
}

fn cmp1(c1: &Cell, c2: &Cell) -> Ordering {
    match (c1, c2) {
        (Cell::Rounded, Cell::Empty) => Ordering::Less,
        _ => Ordering::Equal,
    }
}

fn cmp2(c1: &Cell, c2: &Cell) -> Ordering {
    match (c1, c2) {
        (Cell::Empty, Cell::Rounded) => Ordering::Less,
        _ => Ordering::Equal,
    }
}

fn tilt(cells: Vec<Vec<Cell>>, cmp: fn(c1: &Cell, c2: &Cell) -> Ordering) -> Vec<Vec<Cell>> {
    cells
        .into_iter()
        .map(|mut row| {
            row.as_mut_slice()
                .split_mut(|c| c == &Cell::Cube)
                .for_each(|continuous_chunk| {
                    continuous_chunk.sort_by(cmp);
                });
            row
        })
        .collect::<Vec<_>>()
}

fn tilt_left(cells: Vec<Vec<Cell>>) -> Vec<Vec<Cell>> {
    tilt(cells, cmp1)
}

fn tilt_right(cells: Vec<Vec<Cell>>) -> Vec<Vec<Cell>> {
    tilt(cells, cmp2)
}

fn cycle(cells: Vec<Vec<Cell>>) -> Result<Vec<Vec<Cell>>, Box<dyn Error>> {
    // north
    let cells = tilt_left(transpose(cells)?);
    // west
    let cells = tilt_left(transpose(cells)?);
    // south
    let cells = tilt_right(transpose(cells)?);
    // east
    let cells = tilt_right(transpose(cells)?);

    Ok(cells)
}

fn solve2(cells: Vec<Vec<Cell>>) -> Result<i32, Box<dyn Error>> {
    let mut cache: HashMap<Vec<Vec<Cell>>, usize> = HashMap::new();
    let mut states: Vec<Vec<Vec<Cell>>> = Vec::new();

    let mut current_cells = cells;

    for i in 0..100_000_000 {
        if let Some(cached) = cache.get(&current_cells) {
            let number_of_states_in_cycle = i - cached;
            let remaining_steps = 1_000_000_000 - i;
            let last_state_index = cached + remaining_steps % number_of_states_in_cycle;

            current_cells = states
                .get(last_state_index)
                .ok_or("Index out of bounds")?
                .clone();

            break;
        } else {
            let new_cells = cycle(current_cells.clone())?;

            states.push(current_cells.clone());
            cache.insert(current_cells.clone(), i);

            current_cells = new_cells;
        }
    }

    transpose(current_cells).map(|cells| count(&cells))
}

fn count(cells: &Vec<Vec<Cell>>) -> i32 {
    cells
        .iter()
        .map(|row| {
            row.iter()
                .enumerate()
                .map(|(i, c)| {
                    if c == &Cell::Rounded {
                        row.len() as i32 - i as i32
                    } else {
                        0
                    }
                })
                .sum::<i32>()
        })
        .sum::<i32>()
}

#[cfg(test)]
mod day14 {
    use std::{
        error::Error,
        fs::File,
        io::{BufRead, BufReader},
    };

    use itertools::Itertools;

    use crate::{count, parse, solve1, solve2, tilt_left, transpose, Cell};

    const EXAMPLE: &str = "\
        O....#....\n\
        O.OO#....#\n\
        .....##...\n\
        OO.#O....O\n\
        .O.....O#.\n\
        O.#..O.#.#\n\
        ..O..#O..O\n\
        .......O..\n\
        #....###..\n\
        #OO..#....";

    fn example() -> Vec<Vec<Cell>> {
        vec![
            vec![
                Cell::Rounded,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Cube,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
            ],
            vec![
                Cell::Rounded,
                Cell::Empty,
                Cell::Rounded,
                Cell::Rounded,
                Cell::Cube,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Cube,
            ],
            vec![
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Cube,
                Cell::Cube,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
            ],
            vec![
                Cell::Rounded,
                Cell::Rounded,
                Cell::Empty,
                Cell::Cube,
                Cell::Rounded,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Rounded,
            ],
            vec![
                Cell::Empty,
                Cell::Rounded,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Rounded,
                Cell::Cube,
                Cell::Empty,
            ],
            vec![
                Cell::Rounded,
                Cell::Empty,
                Cell::Cube,
                Cell::Empty,
                Cell::Empty,
                Cell::Rounded,
                Cell::Empty,
                Cell::Cube,
                Cell::Empty,
                Cell::Cube,
            ],
            vec![
                Cell::Empty,
                Cell::Empty,
                Cell::Rounded,
                Cell::Empty,
                Cell::Empty,
                Cell::Cube,
                Cell::Rounded,
                Cell::Empty,
                Cell::Empty,
                Cell::Rounded,
            ],
            vec![
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Rounded,
                Cell::Empty,
                Cell::Empty,
            ],
            vec![
                Cell::Cube,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Cube,
                Cell::Cube,
                Cell::Cube,
                Cell::Empty,
                Cell::Empty,
            ],
            vec![
                Cell::Cube,
                Cell::Rounded,
                Cell::Rounded,
                Cell::Empty,
                Cell::Empty,
                Cell::Cube,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
            ],
        ]
    }

    fn example_tilted_north() -> Vec<Vec<Cell>> {
        vec![
            vec![
                Cell::Rounded,
                Cell::Rounded,
                Cell::Rounded,
                Cell::Rounded,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Cube,
                Cell::Cube,
            ],
            vec![
                Cell::Rounded,
                Cell::Rounded,
                Cell::Rounded,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
            ],
            vec![
                Cell::Rounded,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Cube,
                Cell::Rounded,
                Cell::Rounded,
                Cell::Empty,
                Cell::Empty,
            ],
            vec![
                Cell::Rounded,
                Cell::Empty,
                Cell::Empty,
                Cell::Cube,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
            ],
            vec![
                Cell::Empty,
                Cell::Cube,
                Cell::Rounded,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
            ],
            vec![
                Cell::Cube,
                Cell::Empty,
                Cell::Cube,
                Cell::Rounded,
                Cell::Empty,
                Cell::Empty,
                Cell::Cube,
                Cell::Empty,
                Cell::Cube,
                Cell::Cube,
            ],
            vec![
                Cell::Empty,
                Cell::Empty,
                Cell::Cube,
                Cell::Rounded,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Cube,
                Cell::Empty,
            ],
            vec![
                Cell::Rounded,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Cube,
                Cell::Rounded,
                Cell::Empty,
                Cell::Cube,
                Cell::Empty,
            ],
            vec![
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Cube,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
            ],
            vec![
                Cell::Empty,
                Cell::Cube,
                Cell::Rounded,
                Cell::Empty,
                Cell::Empty,
                Cell::Cube,
                Cell::Rounded,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
            ],
        ]
    }

    #[test]
    fn test_parse() -> Result<(), Box<dyn Error>> {
        let result = parse(EXAMPLE.lines().map(|s| s.to_string()))?;
        assert_eq!(result, example());
        Ok(())
    }

    #[test]
    fn test_tilted_north() -> Result<(), Box<dyn Error>> {
        let result = tilt_left(transpose(example())?);
        assert_eq!(result, example_tilted_north());
        Ok(())
    }

    #[test]
    fn test_count() {
        let result = count(&example_tilted_north());
        assert_eq!(result, 136);
    }

    #[test]
    fn test_solve1_input() -> Result<(), Box<dyn Error>> {
        let file = File::open("input")?;
        let reader = BufReader::new(file);
        let cells = reader.lines().process_results(|itr| parse(itr))??;
        let result = solve1(cells)?;

        assert_eq!(result, 110821);
        Ok(())
    }

    #[test]
    fn test_solve2_example() -> Result<(), Box<dyn Error>> {
        let result = solve2(example())?;

        assert_eq!(result, 64);
        Ok(())
    }

    #[test]
    fn test_solve2_input() -> Result<(), Box<dyn Error>> {
        let file = File::open("input")?;
        let reader = BufReader::new(file);
        let cells = reader.lines().process_results(|itr| parse(itr))??;
        let result = solve2(cells)?;

        assert_eq!(result, 83516);
        Ok(())
    }
}
