#![allow(non_snake_case)]

use proconio::input;
use rand::prelude::*;
use std::ops::RangeBounds;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Input {
    pub W: usize,
    pub H: usize,
    pub costs: [i64; 5],
    pub holes: Vec<(usize, usize)>,
}

impl std::fmt::Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} {} {}", self.W, self.H, self.holes.len())?;
        writeln!(
            f,
            "{} {} {} {} {}",
            self.costs[0], self.costs[1], self.costs[2], self.costs[3], self.costs[4]
        )?;
        for &(x, y) in &self.holes {
            writeln!(f, "{} {}", x, y)?;
        }
        Ok(())
    }
}

pub fn parse_input(text: &str) -> Input {
    let source = proconio::source::once::OnceSource::from(text);
    input! { from source, W: usize, H: usize, K: usize, costs: [i64; 5], holes: [(usize, usize); K] }
    Input {
        W,
        H,
        costs: costs.try_into().unwrap(),
        holes,
    }
}

fn abbreviate(s: &str) -> String {
    if s.chars().count() <= 40 {
        s.to_owned()
    } else {
        format!("{}...", s.chars().take(40).collect::<String>())
    }
}

fn read<T: Copy + PartialOrd + std::fmt::Display + std::str::FromStr, R: RangeBounds<T>>(
    token: Option<&str>,
    range: R,
) -> Result<T, String> {
    let token = token.ok_or("Unexpected EOF")?;
    let value = token
        .parse::<T>()
        .map_err(|_| format!("Parse error: {}", abbreviate(token)))?;
    if range.contains(&value) {
        Ok(value)
    } else {
        Err(format!("Out of range: {}", value))
    }
}

#[derive(Clone, Debug, Default)]
pub struct GenOption {}

impl GenOption {
    pub fn set_i32(&mut self, name: &str, _value: i32) -> Result<(), String> {
        Err(format!("Unknown generator option: {}", name))
    }
}

pub fn generate(seed: u64, _option: &GenOption) -> Input {
    let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(seed);
    let costs = [
        5,
        rng.random_range(5..=10),
        rng.random_range(9..=14),
        rng.random_range(13..=18),
        rng.random_range(17..=22),
    ];
    let mut holes = Vec::new();
    let mut used = vec![false; 60 * 40];
    while holes.len() < 120 {
        let x = rng.random_range(0..60u32) as usize;
        let y = rng.random_range(0..40u32) as usize;
        if !used[y * 60 + x] {
            used[y * 60 + x] = true;
            holes.push((x, y));
        }
    }
    Input {
        W: 60,
        H: 40,
        costs,
        holes,
    }
}

#[derive(Clone, Debug)]
pub struct Brick {
    pub x: usize,
    pub y: usize,
    pub width: usize,
}

#[derive(Clone, Debug)]
pub struct Output {
    pub solutions: Vec<Vec<Brick>>,
}

pub fn parse_output(input: &Input, text: &str) -> Result<Output, String> {
    let mut tokens = text.split_whitespace();
    let mut solutions = Vec::new();
    while let Some(token) = tokens.next() {
        let m: usize = read(Some(token), 0..=input.W * input.H)?;
        let mut bricks = Vec::with_capacity(m);
        for _ in 0..m {
            let x = read(tokens.next(), 0..input.W)?;
            let y = read(tokens.next(), 0..input.H)?;
            let width = read(tokens.next(), 1..=9)?;
            if width % 2 == 0 {
                return Err(format!("Invalid brick width: {}", width));
            }
            bricks.push(Brick { x, y, width });
        }
        solutions.push(bricks);
    }
    if solutions.is_empty() {
        return Err("No solution was output".to_owned());
    }
    Ok(Output { solutions })
}

pub struct Placement {
    pub cells: Vec<Option<usize>>,
    pub cost: i64,
    pub covered: usize,
    pub verdict: Result<i64, String>,
}

pub fn evaluate_solution(input: &Input, bricks: &[Brick]) -> Placement {
    let mut cells = vec![None; input.W * input.H];
    let mut error = None;
    let mut cost = 0;
    for (i, brick) in bricks.iter().enumerate() {
        cost += input.costs[(brick.width - 1) / 2];
        if brick.x + brick.width > input.W || brick.y >= input.H {
            error.get_or_insert_with(|| format!("Brick {} is outside the wall", i));
        }
        if brick.y < input.H {
            for x in brick.x..(brick.x + brick.width).min(input.W) {
                let cell = &mut cells[brick.y * input.W + x];
                if cell.is_some() {
                    error.get_or_insert_with(|| format!("Bricks overlap at ({}, {})", x, brick.y));
                }
                *cell = Some(i);
            }
        }
    }
    for (i, brick) in bricks.iter().enumerate() {
        let center = brick.x + brick.width / 2;
        if brick.y > 0
            && brick.y < input.H
            && center < input.W
            && cells[(brick.y - 1) * input.W + center].is_none()
        {
            error.get_or_insert_with(|| format!("Brick {} has no central support", i));
        }
    }
    let mut covered = 0;
    for &(x, y) in &input.holes {
        if cells[y * input.W + x].is_some() {
            covered += 1;
        } else {
            error.get_or_insert_with(|| format!("Hole ({}, {}) is not covered", x, y));
        }
    }
    let score = (input.W as i64 * input.H as i64 * input.costs[0] - cost + 1).max(0);
    Placement {
        cells,
        cost,
        covered,
        verdict: error.map_or(Ok(score), Err),
    }
}

pub struct Evaluation {
    verdict: Result<i64, String>,
}

impl Evaluation {
    pub fn verdict(&self) -> &Result<i64, String> {
        &self.verdict
    }
}

pub fn evaluate(input: &Input, out: &Output) -> Evaluation {
    let verdict = out.solutions.last().map_or_else(
        || Err("No solution was output".to_owned()),
        |bricks| evaluate_solution(input, bricks).verdict,
    );
    Evaluation { verdict }
}

pub mod vis;
