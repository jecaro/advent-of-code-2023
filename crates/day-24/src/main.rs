use itertools::process_results;
use itertools::Itertools;
use lib::get_args;
use nalgebra::Matrix6;
use nalgebra::RowVector6;
use nalgebra::Vector6;
use std::{
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
            let hailstones = process_results(stdin().lock().lines(), |lines| parse(lines))??;

            let result = if arg == "-1" {
                solve1(&hailstones)
            } else {
                solve2(&hailstones)?
            };

            println!("{}", result);
        }
        _ => usage(prog_name),
    }

    Ok(())
}

#[derive(Clone, Debug, PartialEq)]
struct Coordinates {
    x: f64,
    y: f64,
    z: f64,
}

impl FromStr for Coordinates {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut coords = s.split(',').map(|s| s.trim().parse());

        Ok(Self {
            x: coords.next().ok_or("missing x")??,
            y: coords.next().ok_or("missing y")??,
            z: coords.next().ok_or("missing z")??,
        })
    }
}

type Position = Coordinates;
type Velocity = Coordinates;

#[derive(Clone, Debug, PartialEq)]
struct Hailstone {
    position: Position,
    velocity: Velocity,
}

impl FromStr for Hailstone {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (position_str, velocity_str) = s.split_once('@').ok_or("missing @")?;

        let position = position_str.trim().parse::<Position>()?;
        let velocity = velocity_str.trim().parse::<Velocity>()?;

        Ok(Self { position, velocity })
    }
}

fn parse(itr: impl Iterator<Item = String>) -> Result<Vec<Hailstone>, Box<dyn Error>> {
    itr.map(|line| line.parse()).collect()
}

// https://stackoverflow.com/a/2932601/12819315
fn intersect_2d(h1: &Hailstone, h2: &Hailstone) -> Option<Position> {
    let dx = h2.position.x - h1.position.x;
    let dy = h2.position.y - h1.position.y;
    let det = h2.velocity.x * h1.velocity.y - h2.velocity.y * h1.velocity.x;

    (det != 0.)
        .then_some({
            let u = (dy * h2.velocity.x - dx * h2.velocity.y) / det;
            let v = (dy * h1.velocity.x - dx * h1.velocity.y) / det;

            (u >= 0. && v >= 0.).then_some(Position {
                x: h1.position.x + u * h1.velocity.x,
                y: h1.position.y + u * h1.velocity.y,
                z: 0.,
            })
        })
        .flatten()
}

fn in_2d_range(p: &Position, (x_min, y_min): (f64, f64), (x_max, y_max): (f64, f64)) -> bool {
    p.x >= x_min && p.x <= x_max && p.y >= y_min && p.y <= y_max
}

fn solve1_any_range(hailstones: &[Hailstone], p_min: (f64, f64), p_max: (f64, f64)) -> usize {
    hailstones
        .iter()
        .combinations(2)
        .filter_map(|two_hailstones| {
            two_hailstones
                .get(0)
                .zip(two_hailstones.get(1))
                .and_then(|(h1, h2)| intersect_2d(h1, h2))
        })
        .filter(|p| in_2d_range(p, p_min, p_max))
        .count()
}

fn solve1(hailstones: &[Hailstone]) -> usize {
    solve1_any_range(
        &hailstones,
        (200_000_000_000_000., 200_000_000_000_000.),
        (400_000_000_000_000., 400_000_000_000_000.),
    )
}

// considering the rock starting from p and moving with velocity v, it move with
// p' = p + v * t
// and a hailstone1 at p1 moving with velocity v1:
// p1' = p1 + v1 * t
// they intersect if:
// p' = p1'
// p + v * t = p1 + v1 * t
// t = (p1 - p) / (v - v1)
//
// this must be valid for x, y and z:
// t = (p1.x - p.x) / (v.x - v1.x) = (p1.y - p.y) / (v.y - v1.y) = (p1.z - p.z) / (v.z - v1.z)
//
// now let's write it as:
// 1)  (p1.x - p.x) / (v.x - v1.x) = (p1.y - p.y) / (v.y - v1.y)
// 2)  (p1.x - p.x) / (v.x - v1.x) = (p1.z - p.z) / (v.z - v1.z)
// 3)  (p1.y - p.y) / (v.y - v1.y) = (p1.z - p.z) / (v.z - v1.z)
//
// let's expand the first one 1):
// (p1.x - p.x) * (v.y - v1.y) = (p1.y - p.y) * (v.x - v1.x)
// (p1.x - p.x) * (v.y - v1.y) - (p1.y - p.y) * (v.x - v1.x) = 0
// p1.x * v.y - p1.x * v1.y - p.x * v.y + p.x * v1.y - p1.y * v.x + p1.y * v1.x + p.y * v.x - p.y * v1.x = 0
//        ---                 ---------   ---                 ---                 ---------   ---
// p.x * v.y and p.y * v.x are non-linear terms
//
// now let's consider another point p2 with a velocity v2:
// p2.x * v.y - p2.x * v2.y - p.x * v.y + p.x * v2.y - p2.y * v.x + p2.y * v2.x - p.y * v.x - p.y * v2.x = 0
//
// if we take the difference between the two equations we get:
//   p1.x * v.y - p1.x * v1.y + p.x * v1.y - p1.y * v.x + p1.y * v1.x - p.y * v1.x
// - p2.x * v.y + p2.x * v2.y - p.x * v2.y + p2.y * v.x - p2.y * v2.x + p.y * v2.x = 0
// p.x * (v1.y - v2.y) + p.y * (v2.x - v1.x) + v.x * (p2.y - p1.y) + v.y * (p1.x - p2.x) - p1.x * v1.y + p2.x * v2.y + p1.y * v1.x - p2.y * v2.x = 0
//
// Now the second one 2):
// p.x * (v1.z - v2.z) + p.z * (v2.x - v1.x) + v.x * (p2.z - p1.z) + v.z * (p1.x - p2.x) - p1.x * v1.z + p2.x * v2.z + p1.z * v1.x - p2.z * v2.x = 0
//
// And the third 3):
// p.y * (v1.z - v2.z) + p.z * (v2.y - v1.y) + v.y * (p2.z - p1.z) + v.z * (p1.y - p2.y) - p1.y * v1.z + p2.y * v2.z + p1.z * v1.y - p2.z * v2.y = 0
//
// Now we have three equations with 6 unknowns. We need another hailstone to get the final system
// p.x * (v1.y - v2.y) + p.y * (v2.x - v1.x)                       + v.x * (p2.y - p1.y) + v.y * (p1.x - p2.x)                       - p1.x * v1.y + p2.x * v2.y + p1.y * v1.x - p2.y * v2.x = 0
// p.x * (v1.z - v2.z)                       + p.z * (v2.x - v1.x) + v.x * (p2.z - p1.z)                       + v.z * (p1.x - p2.x) - p1.x * v1.z + p2.x * v2.z + p1.z * v1.x - p2.z * v2.x = 0
//                       p.y * (v1.z - v2.z) + p.z * (v2.y - v1.y)                       + v.y * (p2.z - p1.z) + v.z * (p1.y - p2.y) - p1.y * v1.z + p2.y * v2.z + p1.z * v1.y - p2.z * v2.y = 0
//
//  (dy'-dy) X + (dx-dx') Y              + (y-y') DX + (x'-x) DY             =  x' dy' - y' dx' - x dy + y dx
//  (dz'-dz) X              + (dx-dx') Z + (z-z') DX             + (x'-x) DZ =  x' dz' - z' dx' - x dz + z dx
//               (dz-dz') Y + (dy'-dy) Z             + (z'-z) DY + (y-y') DZ = -y' dz' + z' dy' + y dz - z dy
fn solve2(hailstones: &[Hailstone]) -> Result<usize, Box<dyn Error>> {
    let h1 = hailstones.get(0).ok_or("missing hailstone 1")?;
    let h2 = hailstones.get(1).ok_or("missing hailstone 2")?;
    let h3 = hailstones.get(2).ok_or("missing hailstone 3")?;

    let p1 = &h1.position;
    let p2 = &h2.position;
    let p3 = &h3.position;

    let v1 = &h1.velocity;
    let v2 = &h2.velocity;
    let v3 = &h3.velocity;

    let coefficients = Matrix6::from_rows(&[
        RowVector6::new(v1.y - v2.y, v2.x - v1.x, 0., p2.y - p1.y, p1.x - p2.x, 0.),
        RowVector6::new(v1.z - v2.z, 0., v2.x - v1.x, p2.z - p1.z, 0., p1.x - p2.x),
        RowVector6::new(0., v1.z - v2.z, v2.y - v1.y, 0., p2.z - p1.z, p1.y - p2.y),
        RowVector6::new(v1.y - v3.y, v3.x - v1.x, 0., p3.y - p1.y, p1.x - p3.x, 0.),
        RowVector6::new(v1.z - v3.z, 0., v3.x - v1.x, p3.z - p1.z, 0., p1.x - p3.x),
        RowVector6::new(0., v1.z - v3.z, v3.y - v1.y, 0., p3.z - p1.z, p1.y - p3.y),
    ]);
    let constant = -Vector6::new(
        -p1.x * v1.y + p2.x * v2.y + p1.y * v1.x - p2.y * v2.x,
        -p1.x * v1.z + p2.x * v2.z + p1.z * v1.x - p2.z * v2.x,
        -p1.y * v1.z + p2.y * v2.z + p1.z * v1.y - p2.z * v2.y,
        -p1.x * v1.y + p3.x * v3.y + p1.y * v1.x - p3.y * v3.x,
        -p1.x * v1.z + p3.x * v3.z + p1.z * v1.x - p3.z * v3.x,
        -p1.y * v1.z + p3.y * v3.z + p1.z * v1.y - p3.z * v3.y,
    );

    // In theory, we should check that there is a solution to the system and if not, take other
    // hailstones. As for this input, the first three hailstones yields the result.
    let result = coefficients.lu().solve(&constant).ok_or("no solution")?;

    let p = Position {
        x: result[0],
        y: result[1],
        z: result[2],
    };

    Ok(p.x.round() as usize + p.y.round() as usize + p.z.round() as usize)
}

#[cfg(test)]
mod day24 {
    use std::{
        fs::File,
        io::{BufRead, BufReader},
    };

    use itertools::process_results;

    use crate::{parse, solve1, solve1_any_range, solve2, Hailstone, Position, Velocity};

    const EXAMPLE: &str = "\
        19, 13, 30 @ -2,  1, -2\n\
        18, 19, 22 @ -1, -1, -2\n\
        20, 25, 34 @ -2, -2, -4\n\
        12, 31, 28 @ -1, -2, -1\n\
        20, 19, 15 @  1, -5, -3";

    fn example() -> Vec<Hailstone> {
        vec![
            Hailstone {
                position: Position {
                    x: 19.,
                    y: 13.,
                    z: 30.,
                },
                velocity: Velocity {
                    x: -2.,
                    y: 1.,
                    z: -2.,
                },
            },
            Hailstone {
                position: Position {
                    x: 18.,
                    y: 19.,
                    z: 22.,
                },
                velocity: Velocity {
                    x: -1.,
                    y: -1.,
                    z: -2.,
                },
            },
            Hailstone {
                position: Position {
                    x: 20.,
                    y: 25.,
                    z: 34.,
                },
                velocity: Velocity {
                    x: -2.,
                    y: -2.,
                    z: -4.,
                },
            },
            Hailstone {
                position: Position {
                    x: 12.,
                    y: 31.,
                    z: 28.,
                },
                velocity: Velocity {
                    x: -1.,
                    y: -2.,
                    z: -1.,
                },
            },
            Hailstone {
                position: Position {
                    x: 20.,
                    y: 19.,
                    z: 15.,
                },
                velocity: Velocity {
                    x: 1.,
                    y: -5.,
                    z: -3.,
                },
            },
        ]
    }

    #[test]
    fn test_parse() {
        let hailstones = parse(EXAMPLE.lines().map(|s| s.to_string())).unwrap();
        assert_eq!(hailstones, example());
    }

    #[test]
    fn test_solve1() {
        assert_eq!(solve1_any_range(&example(), (7., 7.), (27., 27.)), 2);
    }

    #[test]
    fn test_solve2() {
        assert_eq!(solve2(&example()).unwrap(), 47);
    }

    #[test]
    fn test_solve1_input() {
        let file = File::open("input").unwrap();
        let reader = BufReader::new(file);
        let hailstones = process_results(reader.lines(), |itr| parse(itr))
            .unwrap()
            .unwrap();
        let result = solve1(&hailstones);

        assert_eq!(result, 24627);
    }

    #[test]
    fn test_solve2_input() {
        let file = File::open("input").unwrap();
        let reader = BufReader::new(file);
        let hailstones = process_results(reader.lines(), |itr| parse(itr))
            .unwrap()
            .unwrap();
        let result = solve2(&hailstones).unwrap();

        assert_eq!(result, 527310134398221);
    }
}
