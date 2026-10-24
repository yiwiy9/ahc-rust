#![allow(non_snake_case)]
use proconio::input;
use rand::prelude::*;
use std::ops::RangeBounds;

pub type Point = (i32, i32);
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Input {
    pub N: i32,
    pub M: usize,
    pub K: usize,
    pub T: usize,
    pub s: Point,
    pub goals: Vec<Point>,
}
impl std::fmt::Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} {} {} {}", self.N, self.M, self.K, self.T)?;
        writeln!(f, "{} {}", self.s.0, self.s.1)?;
        for &(i, j) in &self.goals {
            writeln!(f, "{} {}", i, j)?;
        }
        Ok(())
    }
}
pub fn parse_input(text: &str) -> Input {
    let source = proconio::source::once::OnceSource::from(text);
    input! { from source, N: i32, M: usize, K: usize, T: usize,
    s: (i32,i32), goals: [(i32,i32); M] }
    Input {
        N,
        M,
        K,
        T,
        s,
        goals,
    }
}
fn abbreviate(s: &str) -> String {
    const LIMIT: usize = 40;
    if s.chars().count() <= LIMIT {
        s.to_owned()
    } else {
        format!("{}...", s.chars().take(LIMIT).collect::<String>())
    }
}

#[allow(dead_code)]
fn read<T: Copy + PartialOrd + std::fmt::Display + std::str::FromStr, R: RangeBounds<T>>(
    token: Option<&str>,
    range: R,
) -> Result<T, String> {
    let Some(token) = token else {
        return Err("Unexpected EOF".to_owned());
    };
    let Ok(value) = token.parse::<T>() else {
        return Err(format!("Parse error: {}", abbreviate(token)));
    };
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
    let mut cells: Vec<i32> = (0..128 * 128).collect();
    for a in 0..151 {
        let b = rng.random_range(a as u32..cells.len() as u32) as usize;
        cells.swap(a, b);
    }
    let points: Vec<Point> = cells[..151].iter().map(|&x| (x / 128, x % 128)).collect();
    Input {
        N: 128,
        M: 150,
        K: 3,
        T: points
            .windows(2)
            .map(|p| ((p[1].0 - p[0].0).abs() + (p[1].1 - p[0].1).abs()) as usize)
            .sum(),
        s: points[0],
        goals: points[1..].to_vec(),
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    U,
    D,
    L,
    R,
    Teleport(Point),
}
#[derive(Clone, Debug)]
pub struct Output {
    pub circles: Vec<Point>,
    pub actions: Vec<Action>,
}
pub fn parse_output(input: &Input, text: &str) -> Result<Output, String> {
    let mut tokens = text.split_whitespace();
    let mut circles = Vec::new();
    for _ in 0..input.K {
        circles.push((
            read(tokens.next(), 0..input.N)?,
            read(tokens.next(), 0..input.N)?,
        ));
    }
    let mut actions = Vec::new();
    while let Some(token) = tokens.next() {
        if actions.len() == input.T {
            return Err(format!("Too many operations (limit {})", input.T));
        }
        actions.push(match token {
            "U" => Action::U,
            "D" => Action::D,
            "L" => Action::L,
            "R" => Action::R,
            "J" => Action::Teleport((
                read(tokens.next(), 0..input.N)?,
                read(tokens.next(), 0..input.N)?,
            )),
            _ => return Err(format!("Unknown operation: {}", abbreviate(token))),
        });
    }
    Ok(Output { circles, actions })
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    pub position: Point,
    pub circles: Vec<Point>,
    pub visited: usize,
    pub steps: usize,
}
impl State {
    pub fn new(input: &Input, out: &Output) -> Self {
        Self {
            position: input.s,
            circles: out.circles.clone(),
            visited: 0,
            steps: 0,
        }
    }
    pub fn apply(&mut self, input: &Input, action: &Action) -> Result<(), String> {
        let (i, j) = self.position;
        let circle_index = if let Action::Teleport(p) = *action {
            Some(self.circles.iter().position(|&q| q == p).ok_or_else(|| {
                format!(
                    "Operation {}: no magic circle at ({}, {})",
                    self.steps + 1,
                    p.0,
                    p.1
                )
            })?)
        } else {
            None
        };
        let next = match *action {
            Action::U => (i - 1, j),
            Action::D => (i + 1, j),
            Action::L => (i, j - 1),
            Action::R => (i, j + 1),
            Action::Teleport((a, b)) => (2 * a - i, 2 * b - j),
        };
        if next.0 < 0 || next.0 >= input.N || next.1 < 0 || next.1 >= input.N {
            return Err(format!(
                "Operation {}: landing outside board ({}, {})",
                self.steps + 1,
                next.0,
                next.1
            ));
        }
        if let Some(k) = circle_index {
            self.circles[k] = self.position;
        }
        self.position = next;
        self.steps += 1;
        if self.visited < input.M && self.position == input.goals[self.visited] {
            self.visited += 1;
        }
        Ok(())
    }
    pub fn score(&self, input: &Input) -> i64 {
        if self.visited == input.M {
            input.M as i64 + input.T as i64 - self.steps as i64 + 1
        } else {
            self.visited as i64 + 1
        }
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
    let mut state = State::new(input, out);
    for action in &out.actions {
        if let Err(error) = state.apply(input, action) {
            return Evaluation {
                verdict: Err(error),
            };
        }
    }
    Evaluation {
        verdict: Ok(state.score(input)),
    }
}
pub mod vis;
