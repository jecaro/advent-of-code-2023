use itertools::{process_results, Itertools};
use lib::get_args;
use std::{
    error::Error,
    io::{self, BufRead},
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
            let result = process_results(io::stdin().lock().lines(), |itr| {
                let input = itr.map(|line| {
                    char_to_located_element(line.clone().chars()).collect::<Vec<LocatedElement>>()
                });

                let solve = match arg.as_str() {
                    "-1" => solve1,
                    _ => solve2,
                };

                solve(input)
            })?;

            println!("{}", result)
        }

        _ => usage(prog_name),
    }
    Ok(())
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum Element {
    Symbol { symbol: char },
    Number { number: i32 },
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct LocatedElement {
    element: Element,
    location: i32,
}

fn state_to_located_element(state: &(i32, String)) -> Option<LocatedElement> {
    let (location, number) = state;
    Some(LocatedElement {
        element: Element::Number {
            number: number.parse::<i32>().ok()?,
        },
        location: *location,
    })
}

fn char_to_located_element<I>(itr: I) -> impl Iterator<Item = LocatedElement>
where
    I: Iterator<Item = char>,
{
    // add a dot at the end of the iterator to loop on a two element window
    itr.chain(['.'])
        .tuple_windows::<(_, _)>()
        .enumerate()
        .scan(
            None,
            |prev_state: &mut Option<(i32, String)>, (location, (c, next))| {
                // skip dots
                if c == '.' {
                    assert!(prev_state.is_none());

                    Some(None)
                // yield a symbol
                } else if !c.is_numeric() {
                    assert!(prev_state.is_none());

                    Some(Some(LocatedElement {
                        element: Element::Symbol { symbol: c },
                        location: location as i32,
                    }))
                // c is a number
                } else {
                    // update the current state
                    match prev_state {
                        None => {
                            *prev_state = Some((location as i32, c.to_string()));
                        }
                        Some((_, s)) => {
                            s.push(c);
                        }
                    };

                    if next.is_numeric() {
                        Some(None)
                    } else {
                        let item = prev_state
                            .as_ref()
                            .map(|state| state_to_located_element(&state));
                        *prev_state = None;

                        item
                    }
                }
            },
        )
        .filter_map(|x| x)
}

fn adjacent(location: i32, number: i32, symbol_location: i32) -> bool {
    let nb_digits = number.to_string().len() as i32;

    symbol_location >= location - 1 && symbol_location <= location + nb_digits
}

fn solve1(itr: impl Iterator<Item = Vec<LocatedElement>>) -> i32 {
    // add an empty line at the beginning
    [Vec::new()]
        .into_iter()
        .chain(itr)
        // and at the end
        .chain([Vec::new()])
        .tuple_windows()
        // to have current in a middle of a three lines window
        .map(|(previous, current, next)| {
            // get all the symbols on the three lines
            let symbols = previous
                .iter()
                .chain(current.iter())
                .chain(next.iter())
                .filter(|located_element| matches!(located_element.element, Element::Symbol { .. }))
                .collect::<Vec<_>>();

            // get all the numbers on current line
            current
                .iter()
                .filter_map(|located_element| match &located_element.element {
                    Element::Number { number } => {
                        if symbols.iter().any(|symbol| {
                            adjacent(located_element.location, *number, symbol.location)
                        }) {
                            Some(number)
                        } else {
                            None
                        }
                    }
                    _ => None,
                })
                .sum::<i32>()
        })
        .sum()
}

fn solve2(itr: impl Iterator<Item = Vec<LocatedElement>>) -> i32 {
    // add an empty line at the beginning
    [Vec::new()]
        .into_iter()
        .chain(itr)
        // and at the end
        .chain([Vec::new()])
        .tuple_windows()
        // to have current in a middle of a three lines window
        .map(|(previous, current, next)| {
            // get all the numbers on the three lines
            let numbers = previous
                .iter()
                .chain(current.iter())
                .chain(next.iter())
                .filter_map(|located_element| match located_element.element {
                    Element::Number { number } => Some((located_element.location, number)),
                    _ => None,
                })
                .collect::<Vec<_>>();

            // get all the stars on current line
            current
                .iter()
                .filter_map(|located_element| match &located_element.element {
                    Element::Symbol { symbol: '*' } => {
                        // get the adjacent numbers
                        let adjacent_numbers = numbers
                            .iter()
                            .filter(|(location, number)| {
                                adjacent(*location, *number, located_element.location)
                            })
                            .collect::<Vec<_>>();

                        match adjacent_numbers.as_slice() {
                            [(_, number1), (_, number2)] => Some(number1 * number2),
                            _ => None,
                        }
                    }
                    _ => None,
                })
                .sum::<i32>()
        })
        .sum()
}

#[cfg(test)]
mod day03 {
    use std::{
        error::Error,
        fs::File,
        io::{BufRead, BufReader},
    };

    use itertools::process_results;

    use crate::{char_to_located_element, solve1, solve2, Element, LocatedElement};

    const LINE1: &str = "467..114..";
    fn line1() -> Vec<LocatedElement> {
        vec![
            LocatedElement {
                element: Element::Number { number: 467 },
                location: 0,
            },
            LocatedElement {
                element: Element::Number { number: 114 },
                location: 5,
            },
        ]
    }

    const LINE2: &str = "...*......";
    fn line2() -> Vec<LocatedElement> {
        vec![LocatedElement {
            element: Element::Symbol { symbol: '*' },
            location: 3,
        }]
    }

    const LINE3: &str = "..35..633.";
    fn line3() -> Vec<LocatedElement> {
        vec![
            LocatedElement {
                element: Element::Number { number: 35 },
                location: 2,
            },
            LocatedElement {
                element: Element::Number { number: 633 },
                location: 6,
            },
        ]
    }

    const LINE4: &str = "......#...";
    fn line4() -> Vec<LocatedElement> {
        vec![LocatedElement {
            element: Element::Symbol { symbol: '#' },
            location: 6,
        }]
    }

    const LINE5: &str = "617*......";
    fn line5() -> Vec<LocatedElement> {
        vec![
            LocatedElement {
                element: Element::Number { number: 617 },
                location: 0,
            },
            LocatedElement {
                element: Element::Symbol { symbol: '*' },
                location: 3,
            },
        ]
    }

    const LINE6: &str = ".....+.58.";
    fn line6() -> Vec<LocatedElement> {
        vec![
            LocatedElement {
                element: Element::Symbol { symbol: '+' },
                location: 5,
            },
            LocatedElement {
                element: Element::Number { number: 58 },
                location: 7,
            },
        ]
    }

    const LINE7: &str = "..592.....";
    fn line7() -> Vec<LocatedElement> {
        vec![LocatedElement {
            element: Element::Number { number: 592 },
            location: 2,
        }]
    }

    const LINE8: &str = "......755.";
    fn line8() -> Vec<LocatedElement> {
        vec![LocatedElement {
            element: Element::Number { number: 755 },
            location: 6,
        }]
    }

    const LINE9: &str = "...$.*....";
    fn line9() -> Vec<LocatedElement> {
        vec![
            LocatedElement {
                element: Element::Symbol { symbol: '$' },
                location: 3,
            },
            LocatedElement {
                element: Element::Symbol { symbol: '*' },
                location: 5,
            },
        ]
    }

    const LINE10: &str = ".664.598..";
    fn line10() -> Vec<LocatedElement> {
        vec![
            LocatedElement {
                element: Element::Number { number: 664 },
                location: 1,
            },
            LocatedElement {
                element: Element::Number { number: 598 },
                location: 5,
            },
        ]
    }

    fn engine() -> String {
        format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
            LINE1, LINE2, LINE3, LINE4, LINE5, LINE6, LINE7, LINE8, LINE9, LINE10
        )
    }

    #[test]
    fn parse_line() {
        assert_eq!(
            char_to_located_element(LINE1.chars()).collect::<Vec<LocatedElement>>(),
            line1()
        );
        assert_eq!(
            char_to_located_element(LINE2.chars()).collect::<Vec<LocatedElement>>(),
            line2()
        );
        assert_eq!(
            char_to_located_element(LINE3.chars()).collect::<Vec<LocatedElement>>(),
            line3()
        );
        assert_eq!(
            char_to_located_element(LINE4.chars()).collect::<Vec<LocatedElement>>(),
            line4()
        );
        assert_eq!(
            char_to_located_element(LINE5.chars()).collect::<Vec<LocatedElement>>(),
            line5()
        );
        assert_eq!(
            char_to_located_element(LINE6.chars()).collect::<Vec<LocatedElement>>(),
            line6()
        );
        assert_eq!(
            char_to_located_element(LINE7.chars()).collect::<Vec<LocatedElement>>(),
            line7()
        );
        assert_eq!(
            char_to_located_element(LINE8.chars()).collect::<Vec<LocatedElement>>(),
            line8()
        );
        assert_eq!(
            char_to_located_element(LINE9.chars()).collect::<Vec<LocatedElement>>(),
            line9()
        );
        assert_eq!(
            char_to_located_element(LINE10.chars()).collect::<Vec<LocatedElement>>(),
            line10()
        );
    }

    #[test]
    fn example_solve1() {
        let result = solve1(
            engine()
                .lines()
                .map(|line| char_to_located_element(line.chars()).collect()),
        );

        assert_eq!(result, 4361);
    }

    #[test]
    fn example_solve2() {
        let result = solve2(
            engine()
                .lines()
                .map(|line| char_to_located_element(line.chars()).collect()),
        );

        assert_eq!(result, 467835);
    }

    #[test]
    fn input_solve1() {
        let file = File::open("input").unwrap();
        let reader = BufReader::new(file);
        let input = reader
            .lines()
            .map(|line| Ok::<_, Box<dyn Error>>(char_to_located_element(line?.chars()).collect()));
        let result = process_results(input, |itr| solve1(itr)).unwrap();

        assert_eq!(result, 533784);
    }

    #[test]
    fn input_solve2() {
        let file = File::open("input").unwrap();
        let reader = BufReader::new(file);
        let input = reader
            .lines()
            .map(|line| Ok::<_, Box<dyn Error>>(char_to_located_element(line?.chars()).collect()));
        let result = process_results(input, |itr| solve2(itr)).unwrap();

        assert_eq!(result, 78826761);
    }
}
