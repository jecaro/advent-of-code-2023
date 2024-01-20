use itertools::{Itertools, Position};
use lib::get_args;
use std::{
    collections::HashMap,
    error::Error,
    io::{stdin, BufRead},
    ops::{Index, IndexMut},
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
            let (workflows, parts) = stdin()
                .lock()
                .lines()
                .process_results(|lines| parse(lines))??;
            let result = if arg == "-1" {
                solve1(&workflows, &parts)?
            } else {
                solve2(&workflows)?
            };

            println!("{}", result);
        }
        _ => usage(prog_name),
    }

    Ok(())
}

const MIN_RANGE: i64 = 1;
const MAX_RANGE: i64 = 4000;

#[derive(Debug, PartialEq, Eq)]
struct Part {
    x: i64,
    m: i64,
    a: i64,
    s: i64,
}

// min and max are included in the range
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Range {
    min: i64,
    max: i64,
}

impl Default for Range {
    fn default() -> Self {
        Range {
            min: MIN_RANGE,
            max: MAX_RANGE,
        }
    }
}

fn possibilities(range: &Range) -> i64 {
    range.max - range.min + 1
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PartRanges {
    x: Vec<Range>,
    m: Vec<Range>,
    a: Vec<Range>,
    s: Vec<Range>,
}

impl Default for PartRanges {
    fn default() -> Self {
        PartRanges {
            x: vec![Default::default()],
            m: vec![Default::default()],
            a: vec![Default::default()],
            s: vec![Default::default()],
        }
    }
}

fn possibilities_ranges(ranges: &PartRanges) -> i64 {
    ranges
        .x
        .iter()
        .chain(ranges.m.iter())
        .chain(ranges.a.iter())
        .chain(ranges.s.iter())
        .map(possibilities)
        .product::<i64>()
}

impl Index<Category> for PartRanges {
    type Output = Vec<Range>;

    fn index(&self, category: Category) -> &Self::Output {
        match category {
            Category::X => &self.x,
            Category::M => &self.m,
            Category::A => &self.a,
            Category::S => &self.s,
        }
    }
}

impl IndexMut<Category> for PartRanges {
    fn index_mut(&mut self, category: Category) -> &mut Self::Output {
        match category {
            Category::X => &mut self.x,
            Category::M => &mut self.m,
            Category::A => &mut self.a,
            Category::S => &mut self.s,
        }
    }
}

impl Default for Part {
    fn default() -> Self {
        Part {
            x: 0,
            m: 0,
            a: 0,
            s: 0,
        }
    }
}

impl FromStr for Part {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s
            .strip_prefix('{')
            .ok_or("missing '{'")?
            .strip_suffix('}')
            .ok_or("missing '}'")?;

        s.split(',').try_fold(
            Default::default(),
            |part: Part, kv| -> Result<_, Box<dyn Error>> {
                let (k, v) = kv.split_once('=').ok_or("missing '='")?;
                let category = Category::try_from(k.chars().next().ok_or("missing category")?)?;
                let value = v.parse::<i64>()?;
                match category {
                    Category::X => Ok(Part { x: value, ..part }),
                    Category::M => Ok(Part { m: value, ..part }),
                    Category::A => Ok(Part { a: value, ..part }),
                    Category::S => Ok(Part { s: value, ..part }),
                }
            },
        )
    }
}

impl Index<Category> for Part {
    type Output = i64;

    fn index(&self, category: Category) -> &Self::Output {
        match category {
            Category::X => &self.x,
            Category::M => &self.m,
            Category::A => &self.a,
            Category::S => &self.s,
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
enum Category {
    X,
    M,
    A,
    S,
}

impl TryFrom<char> for Category {
    type Error = Box<dyn Error>;

    fn try_from(value: char) -> Result<Self, Box<dyn Error>> {
        match value {
            'x' => Ok(Category::X),
            'm' => Ok(Category::M),
            'a' => Ok(Category::A),
            's' => Ok(Category::S),
            _ => Err("invalid category".into()),
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
enum Comparison {
    LessThan,
    GreaterThan,
}

impl TryFrom<char> for Comparison {
    type Error = Box<dyn Error>;

    fn try_from(value: char) -> Result<Self, Box<dyn Error>> {
        match value {
            '<' => Ok(Comparison::LessThan),
            '>' => Ok(Comparison::GreaterThan),
            _ => Err("invalid comparison".into()),
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
struct Condition {
    category: Category,
    comparison: Comparison,
    value: i64,
}

fn to_range(condition: &Condition) -> Range {
    match condition.comparison {
        Comparison::LessThan => Range {
            min: MIN_RANGE,
            max: (condition.value - 1).max(MIN_RANGE),
        },
        Comparison::GreaterThan => Range {
            min: (condition.value + 1).min(MAX_RANGE),
            max: MAX_RANGE,
        },
    }
}

impl FromStr for Condition {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();

        let category = Category::try_from(chars.next().ok_or("missing category")?)?;
        let comparison = Comparison::try_from(chars.next().ok_or("missing comparison")?)?;
        let value = chars.collect::<String>().parse::<i64>()?;

        Ok(Condition {
            category,
            comparison,
            value,
        })
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
struct Workflow {
    name: String,
    conditions: Vec<(Condition, String)>,
    fallback: String,
}

impl FromStr for Workflow {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let brackets = s.find('{').ok_or("missing '{'")?;

        let (name, rest) = s.split_at(brackets);

        let conditions_str = rest
            .strip_prefix('{')
            .ok_or("missing '{'")?
            .strip_suffix('}')
            .ok_or("missing '}'")?;

        let mut conditions_iter = conditions_str.split(',').with_position();
        let conditions = conditions_iter
            .take_while_ref(|(position, _)| {
                *position != Position::Last && *position != Position::Only
            })
            .map(|(_, condition_str)| {
                let condition_and_name = condition_str.split(':').collect::<Vec<&str>>();

                let condition = condition_and_name.get(0).ok_or("missing condition")?;
                let name = condition_and_name.get(1).ok_or("missing name")?;

                Ok((condition.parse::<Condition>()?, name.to_string()))
            })
            .collect::<Result<Vec<(Condition, String)>, Box<dyn Error>>>()?;

        let fallback = conditions_iter
            .next()
            .ok_or("missing fallback")?
            .1
            .to_string();

        Ok(Workflow {
            name: name.to_string(),
            conditions,
            fallback,
        })
    }
}

fn parse(itr: impl Iterator<Item = String>) -> Result<(Vec<Workflow>, Vec<Part>), Box<dyn Error>> {
    let mut itr = itr;
    let workflows = itr
        .by_ref()
        .take_while(|s| !s.is_empty())
        .map(|s| s.parse::<Workflow>())
        .collect::<Result<Vec<_>, _>>()?;
    let parts = itr
        .map(|s| s.parse::<Part>())
        .collect::<Result<Vec<_>, _>>()?;
    Ok((workflows, parts))
}

fn apply_a_workflow1(part: &Part, workflow: &Workflow) -> String {
    workflow
        .conditions
        .iter()
        .find(|(condition, _)| match condition.comparison {
            Comparison::LessThan => part[condition.category] < condition.value,
            Comparison::GreaterThan => part[condition.category] > condition.value,
        })
        .map(|(_, name)| name)
        .unwrap_or(&workflow.fallback)
        .clone()
}

fn apply_workflows(part: &Part, workflows: &Vec<Workflow>) -> Result<bool, Box<dyn Error>> {
    let mut stack: Vec<String> = Vec::new();
    stack.push("in".to_string());

    let name_to_workflow = workflow_get_map(workflows);

    while let Some(name) = stack.pop() {
        match name.as_str() {
            "R" => return Ok(false),
            "A" => return Ok(true),
            _ => {
                let workflow = name_to_workflow.get(&name).ok_or("missing workflow")?;
                let next_workflow = apply_a_workflow1(&part, workflow);
                stack.push(next_workflow);
            }
        }
    }

    Err("no workflow found".into())
}

fn apply_a_workflow2(workflow: &Workflow) -> Vec<(String, PartRanges)> {
    // while we walk through the conditions, this variable stores the ranges that correspond to
    // the negated conditions
    let mut invalid_ranges: PartRanges = Default::default();

    let mut results = workflow
        .conditions
        .iter()
        .map(|(condition, next_workflow)| {
            let range = to_range(&condition);

            let ranges = intersect_ranges_range(&invalid_ranges[condition.category], &range);

            let part_ranges = match condition.category {
                Category::X => PartRanges {
                    x: ranges,
                    ..invalid_ranges.clone()
                },
                Category::M => PartRanges {
                    m: ranges,
                    ..invalid_ranges.clone()
                },
                Category::A => PartRanges {
                    a: ranges,
                    ..invalid_ranges.clone()
                },
                Category::S => PartRanges {
                    s: ranges,
                    ..invalid_ranges.clone()
                },
            };

            invalid_ranges[condition.category] =
                intersect_ranges_ranges(&invalid_ranges[condition.category], &opposite(&range));

            (next_workflow.clone(), part_ranges)
        })
        .collect::<Vec<(String, PartRanges)>>();

    results.push((workflow.fallback.clone(), invalid_ranges));

    results
}

fn range_valid(range: &Range) -> bool {
    range.min <= range.max
}

fn opposite(range: &Range) -> Vec<Range> {
    vec![
        Range {
            min: MIN_RANGE,
            max: (range.min - 1).max(MIN_RANGE),
        },
        Range {
            min: (range.max + 1).min(MAX_RANGE),
            max: MAX_RANGE,
        },
    ]
    .into_iter()
    .filter(range_valid)
    .collect()
}

fn intersect_part_ranges(ranges1: &PartRanges, ranges2: &PartRanges) -> PartRanges {
    PartRanges {
        x: intersect_ranges_ranges(&ranges1.x, &ranges2.x),
        m: intersect_ranges_ranges(&ranges1.m, &ranges2.m),
        a: intersect_ranges_ranges(&ranges1.a, &ranges2.a),
        s: intersect_ranges_ranges(&ranges1.s, &ranges2.s),
    }
}

fn intersect_ranges_ranges(ranges1: &Vec<Range>, ranges2: &Vec<Range>) -> Vec<Range> {
    ranges1
        .iter()
        .flat_map(|range1| intersect_ranges_range(ranges2, range1))
        .collect()
}

fn intersect_ranges_range(ranges: &Vec<Range>, range: &Range) -> Vec<Range> {
    ranges
        .iter()
        .filter_map(|range_| intersect_range_range(range_, range))
        .collect()
}

fn intersect_range_range(range1: &Range, range2: &Range) -> Option<Range> {
    let range = Range {
        min: range1.min.max(range2.min),
        max: range1.max.min(range2.max),
    };
    range_valid(&range).then_some(range)
}

fn workflow_get_map(workflows: &Vec<Workflow>) -> HashMap<String, &Workflow> {
    HashMap::from_iter(
        workflows
            .iter()
            .map(|workflow| (workflow.name.clone(), workflow)),
    )
}

fn solve1(workflows: &Vec<Workflow>, parts: &Vec<Part>) -> Result<i64, Box<dyn Error>> {
    parts
        .iter()
        .filter_map(|part| {
            let accepted = apply_workflows(part, workflows);
            match accepted {
                Err(e) => Some(Err(e)),
                Ok(false) => None,
                Ok(true) => Some(Ok(part.x + part.m + part.a + part.s)),
            }
        })
        .sum()
}

fn solve2(workflows: &Vec<Workflow>) -> Result<i64, Box<dyn Error>> {
    let mut stack: Vec<(String, PartRanges)> = Vec::new();
    stack.push(("in".to_string(), Default::default()));

    let name_to_workflow = workflow_get_map(workflows);

    let mut result = 0;

    while let Some((name, ranges)) = stack.pop() {
        match name.as_str() {
            "R" => continue,
            "A" => {
                result += possibilities_ranges(&ranges);
            }
            _ => {
                let workflow = name_to_workflow.get(&name).ok_or("missing workflow")?;
                let workflows_and_ranges = apply_a_workflow2(workflow);

                workflows_and_ranges
                    .iter()
                    .for_each(|(next_workflow, next_ranges)| {
                        let new_ranges = intersect_part_ranges(&ranges, &next_ranges);

                        if possibilities_ranges(&new_ranges) != 0 {
                            stack.push((next_workflow.to_string(), new_ranges));
                        }
                    });
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod day19 {
    use std::{
        error::Error,
        fs::File,
        io::{BufRead, BufReader},
    };

    use itertools::Itertools;

    use crate::{parse, solve1, solve2, Category, Comparison, Condition, Part, Workflow};

    const WORKFLOW: &str = "\
        px{a<2006:qkq,m>2090:A,rfg}\n\
        pv{a>1716:R,A}\n\
        lnx{m>1548:A,A}\n\
        rfg{s<537:gd,x>2440:R,A}\n\
        qs{s>3448:A,lnx}\n\
        qkq{x<1416:A,crn}\n\
        crn{x>2662:A,R}\n\
        in{s<1351:px,qqz}\n\
        qqz{s>2770:qs,m<1801:hdj,R}\n\
        gd{a>3333:R,R}\n\
        hdj{m>838:A,pv}";

    fn workflows() -> Vec<Workflow> {
        vec![
            Workflow {
                name: "px".to_string(),
                conditions: vec![
                    (
                        Condition {
                            category: Category::A,
                            comparison: Comparison::LessThan,
                            value: 2006,
                        },
                        "qkq".to_string(),
                    ),
                    (
                        Condition {
                            category: Category::M,
                            comparison: Comparison::GreaterThan,
                            value: 2090,
                        },
                        "A".to_string(),
                    ),
                ],
                fallback: "rfg".to_string(),
            },
            Workflow {
                name: "pv".to_string(),
                conditions: vec![(
                    Condition {
                        category: Category::A,
                        comparison: Comparison::GreaterThan,
                        value: 1716,
                    },
                    "R".to_string(),
                )],
                fallback: "A".to_string(),
            },
            Workflow {
                name: "lnx".to_string(),
                conditions: vec![(
                    Condition {
                        category: Category::M,
                        comparison: Comparison::GreaterThan,
                        value: 1548,
                    },
                    "A".to_string(),
                )],
                fallback: "A".to_string(),
            },
            Workflow {
                name: "rfg".to_string(),
                conditions: vec![
                    (
                        Condition {
                            category: Category::S,
                            comparison: Comparison::LessThan,
                            value: 537,
                        },
                        "gd".to_string(),
                    ),
                    (
                        Condition {
                            category: Category::X,
                            comparison: Comparison::GreaterThan,
                            value: 2440,
                        },
                        "R".to_string(),
                    ),
                ],
                fallback: "A".to_string(),
            },
            Workflow {
                name: "qs".to_string(),
                conditions: vec![(
                    Condition {
                        category: Category::S,
                        comparison: Comparison::GreaterThan,
                        value: 3448,
                    },
                    "A".to_string(),
                )],
                fallback: "lnx".to_string(),
            },
            Workflow {
                name: "qkq".to_string(),
                conditions: vec![(
                    Condition {
                        category: Category::X,
                        comparison: Comparison::LessThan,
                        value: 1416,
                    },
                    "A".to_string(),
                )],
                fallback: "crn".to_string(),
            },
            Workflow {
                name: "crn".to_string(),
                conditions: vec![(
                    Condition {
                        category: Category::X,
                        comparison: Comparison::GreaterThan,
                        value: 2662,
                    },
                    "A".to_string(),
                )],
                fallback: "R".to_string(),
            },
            Workflow {
                name: "in".to_string(),
                conditions: vec![(
                    Condition {
                        category: Category::S,
                        comparison: Comparison::LessThan,
                        value: 1351,
                    },
                    "px".to_string(),
                )],
                fallback: "qqz".to_string(),
            },
            Workflow {
                name: "qqz".to_string(),
                conditions: vec![
                    (
                        Condition {
                            category: Category::S,
                            comparison: Comparison::GreaterThan,
                            value: 2770,
                        },
                        "qs".to_string(),
                    ),
                    (
                        Condition {
                            category: Category::M,
                            comparison: Comparison::LessThan,
                            value: 1801,
                        },
                        "hdj".to_string(),
                    ),
                ],
                fallback: "R".to_string(),
            },
            Workflow {
                name: "gd".to_string(),
                conditions: vec![(
                    Condition {
                        category: Category::A,
                        comparison: Comparison::GreaterThan,
                        value: 3333,
                    },
                    "R".to_string(),
                )],
                fallback: "R".to_string(),
            },
            Workflow {
                name: "hdj".to_string(),
                conditions: vec![(
                    Condition {
                        category: Category::M,
                        comparison: Comparison::GreaterThan,
                        value: 838,
                    },
                    "A".to_string(),
                )],
                fallback: "pv".to_string(),
            },
        ]
    }

    const PARTS: &str = "\
        {x=787,m=2655,a=1222,s=2876}\n\
        {x=1679,m=44,a=2067,s=496}\n\
        {x=2036,m=264,a=79,s=2244}\n\
        {x=2461,m=1339,a=466,s=291}\n\
        {x=2127,m=1623,a=2188,s=1013}";

    fn parts() -> Vec<Part> {
        vec![
            Part {
                x: 787,
                m: 2655,
                a: 1222,
                s: 2876,
            },
            Part {
                x: 1679,
                m: 44,
                a: 2067,
                s: 496,
            },
            Part {
                x: 2036,
                m: 264,
                a: 79,
                s: 2244,
            },
            Part {
                x: 2461,
                m: 1339,
                a: 466,
                s: 291,
            },
            Part {
                x: 2127,
                m: 1623,
                a: 2188,
                s: 1013,
            },
        ]
    }

    #[test]
    fn test_parse_workflows() -> Result<(), Box<dyn Error>> {
        let workflows_ = WORKFLOW
            .lines()
            .map(|s| s.parse::<Workflow>())
            .collect::<Result<Vec<_>, _>>()?;
        assert_eq!(workflows_, workflows());
        Ok(())
    }

    #[test]
    fn test_parse_parts() -> Result<(), Box<dyn Error>> {
        let parts_ = PARTS
            .lines()
            .map(|s| s.parse::<Part>())
            .collect::<Result<Vec<_>, _>>()?;
        assert_eq!(parts_, parts());
        Ok(())
    }

    #[test]
    fn test_parse() -> Result<(), Box<dyn Error>> {
        let input = format!("{}\n\n{}", WORKFLOW, PARTS);
        let (workflows_, parts_) = parse(input.lines().map(|s| s.to_string()))?;
        assert_eq!(workflows_, workflows());
        assert_eq!(parts_, parts());
        Ok(())
    }

    #[test]
    fn test_solve1_example() -> Result<(), Box<dyn Error>> {
        let result = solve1(&workflows(), &parts())?;
        assert_eq!(result, 19114);
        Ok(())
    }

    #[test]
    fn test_solve2_example() -> Result<(), Box<dyn Error>> {
        let result = solve2(&workflows())?;
        assert_eq!(result, 167409079868000);
        Ok(())
    }

    #[test]
    fn test_solve1_input() -> Result<(), Box<dyn Error>> {
        let file = File::open("input")?;
        let reader = BufReader::new(file);
        let (workflows, parts) = reader.lines().process_results(|itr| parse(itr))??;
        let result = solve1(&workflows, &parts)?;

        assert_eq!(result, 432434);
        Ok(())
    }

    #[test]
    fn test_solve2_input() -> Result<(), Box<dyn Error>> {
        let file = File::open("input")?;
        let reader = BufReader::new(file);
        let (workflows, _) = reader.lines().process_results(|itr| parse(itr))??;
        let result = solve2(&workflows)?;

        assert_eq!(result, 132557544578569);
        Ok(())
    }
}
