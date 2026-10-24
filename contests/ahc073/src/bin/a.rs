// Pre-contest AI-generated template (published before the contest):
// https://github.com/yiwiy9/ahc-rust/tree/main/templates

use itertools::Itertools;
use proconio::input;

const N: usize = 128;
const M: usize = 150;
const K: usize = 3;
const BEAM_WIDTH: usize = 100;

const DX: [i32; 4] = [1, 0, -1, 0];
const DY: [i32; 4] = [0, 1, 0, -1];

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
        let mut best_score = i64::MIN;
        let mut best_action = None;

        for action in state.legal_actions(input) {
            let mut next_state = state.clone();
            next_state.advance(input, (&action.0, action.1, action.2));

            if next_state.evaluated_score > best_score {
                best_score = next_state.evaluated_score;
                best_action = Some(action);
            }
        }

        let best_action = best_action.unwrap();
        state.advance(input, (&best_action.0, best_action.1, best_action.2));
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
                next_state.advance(input, (&action.0, action.1, action.2));
                next_beam.push(next_state);
            }
        }

        next_beam.sort_unstable_by(|left, right| right.evaluated_score.cmp(&left.evaluated_score));
        next_beam.truncate(beam_width);
        beam = next_beam;
    }

    beam.swap_remove(0)
}

fn main() {
    let initial_circles = [(32, 32), (64, 64), (96, 96)];
    let input = read_input();
    // let state = solve_greedy(&input, State::new(&input, &initial_circles));
    let state = solve_beam(&input, State::new(&input, &initial_circles), BEAM_WIDTH);

    let output = Output {
        initial_circles,
        operations: get_operations_from_moving(input.s, &state.moving),
    };

    // デバッグビルドでは、State内の得点と完成解の全体再計算を照合する。
    // debug_assert_eq!はreleaseビルドでは実行されない。
    debug_assert_eq!(state.actual_score, calculate_score(&input, &output));
    print_output(&output);
}

// ============================================================================
// 問題ごとに実装する部分
//
// 入出力と得点計算を用意し、部分解の持ち方、操作列挙、状態更新の順に実装する。
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
struct Input {
    t: usize,
    s: (usize, usize),
    g: Vec<(usize, usize)>,
}

fn read_input() -> Input {
    input! {
        _n: usize,
        _m: usize,
        _k: usize,
        t: usize,
        s: (usize,usize),
        g: [(usize,usize); M],
    }

    Input { t, s, g }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Operation {
    Walk(char),
    Transfer((char, usize, usize)),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Output {
    initial_circles: [(usize, usize); K],
    operations: Vec<Operation>,
}
fn print_output(output: &Output) {
    println!(
        "{}",
        output
            .initial_circles
            .iter()
            .map(|(a, b)| format!("{a} {b}"))
            .join("\n")
    );
    println!(
        "{}",
        output
            .operations
            .iter()
            .map(|operation| {
                match operation {
                    Operation::Walk(c) => format!("{c}"),
                    Operation::Transfer((c, a, b)) => format!("{c} {a} {b}"),
                }
            })
            .join("\n")
    );
}

fn calculate_score(input: &Input, output: &Output) -> i64 {
    (M + input.t + 1) as i64 - output.operations.len() as i64
}

fn get_operations_from_moving(s: (usize, usize), moving: &Vec<MovingOp>) -> Vec<Operation> {
    let mut operations = vec![];
    let mut cur = s;
    for &moving_op in moving {
        match moving_op {
            MovingOp::Walk(next) => {
                let diff_i = next.0 as i64 - cur.0 as i64;
                let diff_j = next.1 as i64 - cur.1 as i64;
                for _ in 0..diff_i.abs() {
                    operations.push(Operation::Walk(if diff_i > 0 { 'D' } else { 'U' }));
                }
                for _ in 0..diff_j.abs() {
                    operations.push(Operation::Walk(if diff_j > 0 { 'R' } else { 'L' }));
                }
                cur = next;
            }
            MovingOp::Transfer(jump) => {
                operations.push(Operation::Transfer(jump));
                cur = (2 * jump.1 - cur.0, 2 * jump.2 - cur.1);
            }
        }
    }
    operations
}

fn calc_manhattan_dist(cur: (usize, usize), next: (usize, usize)) -> usize {
    let diff_i = next.0 as i64 - cur.0 as i64;
    let diff_j = next.1 as i64 - cur.1 as i64;
    (diff_i.abs() + diff_j.abs()) as usize
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum MovingOp {
    Walk((usize, usize)),
    Transfer((char, usize, usize)),
}

#[derive(Debug, Clone)]
struct State {
    // これまでに選んだ操作列。探索終了時にそのまま提出形式で出力する。
    moving: Vec<MovingOp>,
    // 候補を残す順番を決めるための評価値。
    evaluated_score: i64,

    // ここから下は、State::advanceで状態を1手進めるために使う。
    actual_score: i64,
    goal_idx: usize,
    current_circles: [(usize, usize); K],
    current_pos: (usize, usize),
}

impl State {
    fn new(input: &Input, initial_circles: &[(usize, usize); K]) -> Self {
        Self {
            moving: Vec::new(),
            evaluated_score: (M + input.t + 1) as i64,
            actual_score: (M + input.t + 1) as i64,
            goal_idx: 0,
            current_circles: initial_circles.clone(),
            current_pos: input.s,
        }
    }

    // 最後まで操作を選び終えたか判定する。
    fn is_done(&self, _input: &Input) -> bool {
        self.goal_idx == M
    }

    // 指定した操作で状態を1日進め、実際の得点と探索用の評価値を更新する。
    fn advance(&mut self, input: &Input, action: (&Vec<MovingOp>, usize, [(usize, usize); K])) {
        let (moving_ops, times, circles) = action;

        self.actual_score -= times as i64;
        self.current_circles = circles;

        // 演習2: 探索途中の評価方法を工夫する場所。
        self.evaluated_score = self.actual_score;

        let dist_from_center = self
            .current_circles
            .iter()
            .map(|&circle| calc_manhattan_dist(circle, (N / 2, N / 2)))
            .sum::<usize>();
        self.evaluated_score -= dist_from_center as i64;

        self.moving.extend_from_slice(moving_ops);
        self.current_pos = input.g[self.goal_idx];
        self.goal_idx += 1;
    }

    // 現在の部分解から次に試す操作を列挙する。
    fn legal_actions(&self, input: &Input) -> Vec<(Vec<MovingOp>, usize, [(usize, usize); K])> {
        let mut actions = vec![];

        actions.push((
            vec![MovingOp::Walk(input.g[self.goal_idx])],
            calc_manhattan_dist(self.current_pos, input.g[self.goal_idx]),
            self.current_circles,
        ));

        let mut cur = self.current_pos;
        while cur != input.g[self.goal_idx] {
            for (k, &(a, b)) in self.current_circles.iter().enumerate() {
                let next_i = 2 * a as i64 - cur.0 as i64;
                let next_j = 2 * b as i64 - cur.1 as i64;
                if next_i >= 0 && next_i < N as i64 && next_j >= 0 && next_j < N as i64 {
                    let mut next_circles = self.current_circles.clone();
                    next_circles[k] = cur;

                    let mut moving_ops = vec![];
                    let mut times = 0;
                    if cur != self.current_pos {
                        moving_ops.push(MovingOp::Walk(cur));
                        times += calc_manhattan_dist(self.current_pos, cur);
                    }

                    moving_ops.push(MovingOp::Transfer(('J', a, b)));
                    times += 1;

                    let next = (next_i as usize, next_j as usize);
                    moving_ops.push(MovingOp::Walk(input.g[self.goal_idx]));
                    times += calc_manhattan_dist(next, input.g[self.goal_idx]);

                    actions.push((moving_ops, times, next_circles));
                }
            }

            let from_cur = calc_manhattan_dist(cur, input.g[self.goal_idx]);

            let mut candidates = vec![];
            for dir in 0..4 {
                let nx = cur.0 as i32 + DX[dir];
                let ny = cur.1 as i32 + DY[dir];
                if nx < 0 || N as i32 <= nx || ny < 0 || N as i32 <= ny {
                    continue;
                }
                let next = (nx as usize, ny as usize);
                let from_next = calc_manhattan_dist(next, input.g[self.goal_idx]);

                if from_next < from_cur {
                    candidates.push(next);
                }
            }

            candidates.sort_by(|a, b| {
                calc_manhattan_dist(*a, (N / 2, N / 2))
                    .cmp(&calc_manhattan_dist(*b, (N / 2, N / 2)))
            });
            cur = candidates[0];
        }

        actions
    }
}
