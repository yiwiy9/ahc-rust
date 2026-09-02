// AHC-SOLUTION: ahc070 beam constructive
// Pre-contest AI-generated template (published before the contest):
// https://gist.github.com/yiwiy9/7e47236abfa9b464015b5308c1cb8bc8/6c536ada69b81a9815193f4a04604621975646da

use itertools::Itertools;
use proconio::input;
use std::collections::HashSet;

// ============================================================================
// 構築型探索の共通部分
//
// 問題固有の処理はStateに任せ、この範囲では候補を1つ残す貪欲法と、
// 評価の高い候補を複数残すビームサーチだけを記述する。
// ============================================================================

// Stateが提供する操作だけを使い、各段階で最良の1候補を選ぶ。
// Stateのインターフェースが同じなら、この関数は変更せず流用できる。
#[allow(dead_code)]
fn solve_greedy(input: &Input, mut state: State) -> State {
    while !state.is_done(input) {
        let mut best_risk = i64::MAX;
        let mut best_action = None;

        for action in state.legal_actions(input) {
            let mut next_state = state.clone();
            next_state.advance(input, action);

            if next_state.evaluated_risk < best_risk {
                best_risk = next_state.evaluated_risk;
                best_action = Some(action);
            }
        }

        let best_action = best_action.unwrap();
        state.advance(input, best_action);
    }

    state
}

// Stateが提供する操作だけを使い、各段階で評価の高い候補をbeam_width個残す。
// Stateのインターフェースが同じなら、この関数は変更せず流用できる。
#[allow(dead_code)]
fn solve_beam(input: &Input, initial_state: State, beam_width: usize) -> State {
    assert!(beam_width > 0);

    let mut beam = vec![initial_state];

    while !beam[0].is_done(input) {
        let mut next_beam = Vec::new();

        for state in &beam {
            for action in state.legal_actions(input) {
                let mut next_state = state.clone();
                next_state.advance(input, action);
                next_beam.push(next_state);
            }
        }

        next_beam.sort_unstable_by(|left, right| left.evaluated_risk.cmp(&right.evaluated_risk));
        next_beam.truncate(beam_width);
        beam = next_beam;
    }

    beam.swap_remove(0)
}

fn main() {
    let input = read_input();
    let patterns = vec![(0, 1), (1, 0), (50, 50)];

    // let state = solve_greedy(&input, State::new(patterns));
    let state = solve_beam(&input, State::new(patterns), BEAM_WIDTH);

    eprintln!(
        "{} {}",
        state.actual_risk,
        calculate_score(&input, &state.output)
    );

    print_output(&state.output);
}

const GRID_LEN: i64 = 100;
const TURN_COUNT: usize = 10000;
const PATTERN_COUNT: usize = 3;

const BEAM_WIDTH: usize = 3;

#[derive(Debug)]
struct Input {
    strange_positions: Vec<(i64, i64)>,
}

fn read_input() -> Input {
    input! {
        _n: usize,
        _m: usize,
        strange_positions: [(i64, i64); TURN_COUNT],
    }

    Input { strange_positions }
}

#[derive(Debug, Clone)]
struct Output {
    patterns: Vec<(i64, i64)>,
    actions: Vec<usize>,
}

fn print_output(output: &Output) {
    println!(
        "{}",
        output
            .patterns
            .iter()
            .map(|&(i, j)| format!("{i} {j}", i = i, j = j))
            .join("\n")
    );
    println!("{}", output.actions.iter().join("\n"));
}

fn calculate_score(input: &Input, output: &Output) -> i64 {
    let mut risk = 0;

    let mut visited_set: HashSet<(i64, i64)> = HashSet::new();
    let mut cur = (0, 0);
    for (turn, &action) in output.actions.iter().enumerate() {
        cur = get_next_pos(cur, output.patterns[action]);
        visited_set.insert(cur);

        let min_dist = visited_set
            .iter()
            .map(|&visited| calc_dist(input.strange_positions[turn], visited))
            .min()
            .unwrap();

        let risk_sq = (min_dist * min_dist * (turn as i64 + 1)) as f64;
        risk += risk_sq.sqrt().floor() as i64;
    }

    risk
    // let score = (1_000_000 * GRID_LEN.pow(3)) as f64 / (risk + 1) as f64;
    // score.round() as i64
}

fn calc_dist(pos_1: (i64, i64), pos_2: (i64, i64)) -> i64 {
    (pos_1.0 - pos_2.0).abs() + (pos_1.1 - pos_2.1).abs()
}

fn get_next_pos(cur: (i64, i64), diff: (i64, i64)) -> (i64, i64) {
    ((cur.0 + diff.0) % GRID_LEN, (cur.1 + diff.1) % GRID_LEN)
}

#[derive(Debug, Clone)]
struct State {
    // これまでに選んだ操作列。探索終了時にそのまま提出形式で出力する。
    output: Output,
    // 候補を残す順番を決めるための評価値。
    evaluated_risk: i64,

    // ここから下は、State::advanceで状態を1日進めるために使う。
    actual_risk: i64,
    turn: usize,
    cur: (i64, i64),
    visited_set: HashSet<(i64, i64)>,
}

impl State {
    fn new(pattens: Vec<(i64, i64)>) -> Self {
        Self {
            output: Output {
                patterns: pattens,
                actions: Vec::new(),
            },
            evaluated_risk: 0,
            actual_risk: 0,
            turn: 0,
            cur: (0, 0),
            visited_set: HashSet::new(),
        }
    }

    // 最後まで操作を選び終えたか判定する。
    fn is_done(&self, _input: &Input) -> bool {
        self.turn == TURN_COUNT
    }

    // 指定した操作で状態を1日進め、実際の得点と探索用の評価値を更新する。
    fn advance(&mut self, input: &Input, action: usize) {
        self.cur = get_next_pos(self.cur, self.output.patterns[action]);
        self.visited_set.insert(self.cur);

        let min_dist = self
            .visited_set
            .iter()
            .map(|&visited| calc_dist(input.strange_positions[self.turn], visited))
            .min()
            .unwrap();

        let risk_sq = (min_dist * min_dist * (self.turn as i64 + 1)) as f64;
        self.actual_risk += risk_sq.sqrt().floor() as i64;

        // 演習2: 探索途中の評価方法を工夫する場所。
        self.evaluated_risk = self.actual_risk;

        self.output.actions.push(action);
        self.turn += 1;
    }

    // 現在の部分解から次に試す操作を列挙する。
    fn legal_actions(&self, _input: &Input) -> Vec<usize> {
        // 演習2: 探索対象にする操作を絞り込む場所。
        (0..PATTERN_COUNT).collect()
    }
}
