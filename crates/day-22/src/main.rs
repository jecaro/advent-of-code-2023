use itertools::Itertools;
use lib::get_args;
use std::{
    collections::{HashMap, HashSet},
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
            let bricks = stdin()
                .lock()
                .lines()
                .process_results(|lines| parse(lines))??;

            let fallen_bricks = fall(&bricks);
            let result = if arg == "-1" {
                solve1(&fallen_bricks)
            } else {
                solve2(&fallen_bricks)
            };

            println!("{}", result);
        }
        _ => usage(prog_name),
    }

    Ok(())
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct Brick {
    from: Coordinate,
    to: Coordinate,
}

impl FromStr for Brick {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (p1_str, p2_str) = s.split_once("~").ok_or("Invalid line")?;

        let p1 = p1_str.parse::<Coordinate>()?;
        let p2 = p2_str.parse::<Coordinate>()?;

        Ok(Brick { from: p1, to: p2 })
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct Coordinate {
    x: i32,
    y: i32,
    z: i32,
}

impl FromStr for Coordinate {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (x, y, z) = s
            .split_once(",")
            .and_then(|(x, yz)| yz.split_once(",").map(|(y, z)| (x, y, z)))
            .ok_or("Invalid coordinate")?;

        Ok(Coordinate {
            x: x.parse()?,
            y: y.parse()?,
            z: z.parse()?,
        })
    }
}

fn parse(itr: impl Iterator<Item = String>) -> Result<Vec<Brick>, Box<dyn Error>> {
    itr.map(|line| line.parse()).collect()
}

fn fall(bricks: &Vec<Brick>) -> Vec<Brick> {
    // sort the bricks by z ascending
    let mut sorted_bricks = bricks.clone();
    sorted_bricks.sort_by_key(|b| bottom(b));

    let result: Vec<Brick> =
        sorted_bricks
            .iter()
            .enumerate()
            .fold(vec![], |mut acc, (i, current_brick)| {
                // get the highest fallen bricks in (0..=i).rev()
                let highest_intersected_brick = acc
                    // in fallen bricks
                    .get(0..i)
                    .unwrap_or(&[])
                    .iter()
                    // that intersect with current brick
                    .filter(|other| intersect_xy(current_brick, *other))
                    // in reverse top z order
                    .sorted_by_key(|b| top(b))
                    .rev()
                    .next();

                let new_bottom = highest_intersected_brick.map_or(1, |b| top(b) + 1);
                let moved_brick = move_bottom_to(current_brick, new_bottom);

                acc.push(moved_brick);
                acc
            });

    result
}

fn solve1(bricks: &Vec<Brick>) -> i32 {
    let supporters = bricks.iter().fold(HashMap::new(), |mut acc, b| {
        let below_b = bricks
            .iter()
            .filter(|other| *other != b && top(other) + 1 == bottom(b) && intersect_xy(b, other))
            .collect::<Vec<_>>();

        acc.insert(b, below_b);
        acc
    });

    let unsafe_to_delete = bricks
        .iter()
        .filter_map(|b| {
            supporters
                .get(b)
                .and_then(|s| (s.len() == 1).then_some(s.get(0)).flatten())
        })
        .collect::<HashSet<_>>()
        .len();

    bricks.len() as i32 - unsafe_to_delete as i32
}

fn solve2(bricks: &Vec<Brick>) -> i32 {
    let (supporters, supporting) = bricks.iter().fold(
        (HashMap::new(), HashMap::new()),
        |(mut supporters, mut supporting), b| {
            let below_b = bricks
                .iter()
                .filter(|other| {
                    *other != b && top(other) + 1 == bottom(b) && intersect_xy(b, other)
                })
                .collect::<Vec<_>>();

            supporters.insert(b, below_b);

            let over_b = bricks
                .iter()
                .filter(|other| {
                    *other != b && top(b) + 1 == bottom(other) && intersect_xy(b, other)
                })
                .collect::<Vec<_>>();

            supporting.insert(b, over_b);

            (supporters, supporting)
        },
    );
    bricks
        .iter()
        .map(|b| {
            let mut falling: HashSet<&Brick> = HashSet::new();
            // that brick doesn't count in the final result, see -1 at the end of the scope
            falling.insert(b);

            // put in the stack all the bricks that will fall if b is desintegrated
            let mut stack = supporting
                .get(b)
                .unwrap_or(&vec![])
                .iter()
                .copied()
                .filter(|b| supporters.get(*b).unwrap_or(&vec![]).len() == 1)
                .unique()
                .collect::<Vec<_>>();

            while let Some(brick) = stack.pop() {
                // if all the supporters of the brick are falling, then the brick will fall too
                if supporters
                    .get(brick)
                    .unwrap_or(&vec![])
                    .iter()
                    .all(|b| falling.contains(b))
                {
                    falling.insert(brick);
                    stack.extend(
                        supporting
                            .get(brick)
                            .unwrap_or(&vec![])
                            .iter()
                            .filter(|b| !falling.contains(**b)),
                    );
                }
            }

            falling.len() as i32 - 1
        })
        .sum()
}

fn intersect_xy(brick1: &Brick, brick2: &Brick) -> bool {
    !disjoint_xy(brick1, brick2)
}

fn disjoint_xy(brick1: &Brick, brick2: &Brick) -> bool {
    left(brick1) > right(brick2)
        || left(brick2) > right(brick1)
        || back(brick1) > front(brick2)
        || back(brick2) > front(brick1)
}

fn top(brick: &Brick) -> i32 {
    brick.from.z.max(brick.to.z)
}

fn bottom(brick: &Brick) -> i32 {
    brick.from.z.min(brick.to.z)
}

fn left(brick: &Brick) -> i32 {
    brick.from.x.min(brick.to.x)
}

fn right(brick: &Brick) -> i32 {
    brick.from.x.max(brick.to.x)
}

fn front(brick: &Brick) -> i32 {
    brick.from.y.max(brick.to.y)
}

fn back(brick: &Brick) -> i32 {
    brick.from.y.min(brick.to.y)
}

fn move_bottom_to(brick: &Brick, z: i32) -> Brick {
    let offset = bottom(brick) - z;
    Brick {
        from: Coordinate {
            x: brick.from.x,
            y: brick.from.y,
            z: brick.from.z - offset,
        },
        to: Coordinate {
            x: brick.to.x,
            y: brick.to.y,
            z: brick.to.z - offset,
        },
    }
}

#[cfg(test)]
mod day22 {
    use std::{
        error::Error,
        fs::File,
        io::{BufRead, BufReader},
    };

    use itertools::Itertools;

    use crate::{fall, intersect_xy, parse, solve1, solve2};

    const EXAMPLE: &str = "\
        1,0,1~1,2,1\n\
        0,0,2~2,0,2\n\
        0,2,3~2,2,3\n\
        0,0,4~0,2,4\n\
        2,0,5~2,2,5\n\
        0,1,6~2,1,6\n\
        1,1,8~1,1,9";

    #[test]
    fn test_parse() -> Result<(), Box<dyn Error>> {
        let bricks = parse(EXAMPLE.lines().map(|s| s.to_string()))?;

        assert_eq!(bricks.len(), 7);

        Ok(())
    }

    #[test]
    fn test_fall() -> Result<(), Box<dyn Error>> {
        let bricks = parse(EXAMPLE.lines().map(|s| s.to_string()))?;
        let fallen_bricks = fall(&bricks);

        assert_eq!(fallen_bricks.len(), 7);

        Ok(())
    }

    #[test]
    fn test_intersect() -> Result<(), Box<dyn Error>> {
        let bricks = parse(EXAMPLE.lines().map(|s| s.to_string()))?;
        let brick_a = &bricks.get(0).ok_or("No brick")?;
        let brick_b = &bricks.get(1).ok_or("No brick")?;
        // let brick_c = &bricks.get(2).ok_or("No brick")?;
        let brick_d = &bricks.get(3).ok_or("No brick")?;
        let brick_e = &bricks.get(4).ok_or("No brick")?;
        let brick_f = &bricks.get(5).ok_or("No brick")?;
        // let brick_g = &bricks.get(6).ok_or("No brick")?;

        assert!(intersect_xy(brick_a, brick_b));
        assert!(intersect_xy(brick_b, brick_a));
        assert!(intersect_xy(brick_d, brick_f));
        assert!(intersect_xy(brick_f, brick_d));
        assert!(intersect_xy(brick_e, brick_f));
        assert!(intersect_xy(brick_f, brick_e));

        Ok(())
    }

    #[test]
    fn test_solve1_example() -> Result<(), Box<dyn Error>> {
        let bricks = parse(EXAMPLE.lines().map(|s| s.to_string()))?;
        let fallen_bricks = fall(&bricks);
        let result = solve1(&fallen_bricks);

        assert_eq!(result, 5);

        Ok(())
    }

    #[test]
    fn test_solve2_example() -> Result<(), Box<dyn Error>> {
        let bricks = parse(EXAMPLE.lines().map(|s| s.to_string()))?;
        let fallen_bricks = fall(&bricks);
        let result = solve2(&fallen_bricks);

        assert_eq!(result, 7);

        Ok(())
    }

    #[test]
    fn test_solve1_input() -> Result<(), Box<dyn Error>> {
        let file = File::open("input")?;
        let reader = BufReader::new(file);
        let bricks = reader.lines().process_results(|itr| parse(itr))??;
        let fallen_bricks = fall(&bricks);
        let result = solve1(&fallen_bricks);

        assert_eq!(result, 432);

        Ok(())
    }

    #[test]
    fn test_solve2_input() -> Result<(), Box<dyn Error>> {
        let file = File::open("input")?;
        let reader = BufReader::new(file);
        let bricks = reader.lines().process_results(|itr| parse(itr))??;
        let fallen_bricks = fall(&bricks);
        let result = solve2(&fallen_bricks);

        assert_eq!(result, 63166);

        Ok(())
    }
}
