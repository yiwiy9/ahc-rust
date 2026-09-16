// Pre-contest AI-generated template (published before the contest):
// https://github.com/yiwiy9/ahc-rust/tree/main/templates

const W: usize = 60;
const H: usize = 40;
const BEAM_WIDTH: usize = 1500;
const LEGAL_ACTION_WIDTH: usize = 3;
const INITIAL_SCORE: i64 = (W * H) as i64 * 5 + 1;

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
        let mut best_cost = i64::MAX;
        let mut best_action = None;

        for action in state.legal_actions(input) {
            let mut next_state = state.clone();
            next_state.advance(input, &action);

            if next_state.evaluated_cost < best_cost {
                best_cost = next_state.evaluated_cost;
                best_action = Some(action);
            }
        }

        let best_action = best_action.unwrap();
        state.advance(input, &best_action);
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
                next_state.advance(input, &action);
                next_beam.push(next_state);
            }
        }

        next_beam.sort_unstable_by(|left, right| left.evaluated_cost.cmp(&right.evaluated_cost));
        next_beam.truncate(beam_width);
        beam = next_beam;
    }

    beam.swap_remove(0)
}

fn main() {
    let input = read_input();
    // let state = solve_greedy(&input, State::new(&input));
    let state = solve_beam(&input, State::new(&input), BEAM_WIDTH);

    // デバッグビルドでは、State内の得点と完成解の全体再計算を照合する。
    // debug_assert_eq!はreleaseビルドでは実行されない。
    debug_assert_eq!(state.actual_score, calculate_score(&input, &state.output));
    // println!(
    //     "{} {}",
    //     state.actual_score,
    //     calculate_score(&input, &state.output)
    // );
    print_output(&state.output);
}

// ============================================================================
// 問題ごとに実装する部分
//
// 入出力と得点計算を用意し、部分解の持ち方、操作列挙、状態更新の順に実装する。
// ============================================================================

use proconio::input;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use superslice::*;

/// 問題文から読み取った、変更されない入力。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Input {
    costs: Vec<i64>,
    row_holes_list: Vec<Vec<usize>>,
    height: usize,
}

/// AtCoderへ出力する解そのもの。
type Output = Vec<(usize, usize, usize)>;

pub fn read_input() -> Input {
    input! {
        _w: usize,
        _h: usize,
        k: usize,
        costs: [i64; 5],
        holes: [(usize, usize); k],
    }

    let mut height = 0;
    let mut row_holes_list = vec![vec![]; 40];
    for &(i, j) in &holes {
        height = height.max(j);
        row_holes_list[j].push(i);
    }

    row_holes_list.iter_mut().for_each(|row| row.sort());

    Input {
        costs,
        row_holes_list,
        height,
    }
}

pub fn print_output(output: &Output) {
    println!("{}", output.len());
    for &(x, y, l) in output {
        println!("{} {} {}", x, y, l);
    }
}

fn l_to_cost_idx(l: usize) -> usize {
    (l - 1) / 2
}

fn cost_idx_to_l(cost_idx: usize) -> usize {
    cost_idx * 2 + 1
}

/// デバッグ時の検算用。公式得点と同じ定義を、まず素直に実装する。
fn calculate_score(input: &Input, output: &Output) -> i64 {
    let mut score = INITIAL_SCORE;

    for &(_, _, l) in output {
        let cost_idx = l_to_cost_idx(l);
        score -= input.costs[cost_idx];
    }

    score
}

/// 構築中の解に加え、次の1手の評価に必要な問題固有データを持つ。
#[derive(Debug, Clone)]
pub struct State {
    output: Output,
    evaluated_cost: i64,
    actual_score: i64,
    // 今何行目か
    row_idx: usize,
    // 新たに追加された穴
    new_row_holes: Vec<usize>,
}

impl State {
    fn new(input: &Input) -> Self {
        Self {
            output: Vec::new(),
            evaluated_cost: 0,
            actual_score: INITIAL_SCORE,
            row_idx: input.height + 1,
            new_row_holes: Vec::new(),
        }
    }

    // 最後まで操作を選び終えたか判定する。
    fn is_done(&self, _input: &Input) -> bool {
        self.row_idx == 0
    }

    // 指定した操作で状態を1つ進め、実際の得点と探索用の評価値を更新する。
    fn advance(&mut self, input: &Input, action: &(Vec<(usize, usize)>, i64)) {
        self.row_idx -= 1;

        self.new_row_holes = vec![];
        for &(x, l) in &action.0 {
            // actual_score
            let cost_idx = l_to_cost_idx(l);
            self.actual_score -= input.costs[cost_idx];

            // output
            self.output.push((x, self.row_idx, l));

            // new_row_holes
            self.new_row_holes.push(x + cost_idx);
        }
        // evaluated_score
        self.evaluated_cost += action.1;
    }

    // 現在の部分解から次に試す操作を列挙する。
    fn legal_actions(&self, input: &Input) -> Vec<(Vec<(usize, usize)>, i64)> {
        let mut row_holes = input.row_holes_list[self.row_idx - 1].clone();
        for &j in &self.new_row_holes {
            row_holes.push(j);
        }
        row_holes.sort_unstable();
        row_holes.dedup();

        let (min_actions, min_cost) = dijkstra(&row_holes, &input.costs);

        // 演習2: 探索対象にする操作を絞り込む場所。
        let mut res = vec![(min_actions.clone(), min_cost)];

        // 同じ状態なら同じ候補を再現しつつ、異なるビーム状態では
        // 別の乱数列を使う。毎回同じ定数seedから始めると、どの状態でも
        // 板の順番ごとに同じ伸ばし方になってしまう。
        let mut hasher = DefaultHasher::new();
        self.output.hash(&mut hasher);
        let mut rng = Pcg64Mcg::seed_from_u64(hasher.finish() ^ 181);
        for _ in 0..LEGAL_ACTION_WIDTH * min_actions.len() {
            let mut cur_actions = vec![];
            let mut cur_cost = 0;
            let mut i = 0;

            while i < min_actions.len() {
                let cur_cost_idx = l_to_cost_idx(min_actions[i].1);
                if cur_cost_idx < 4 && rng.random_bool(0.5) {
                    let next_cost_idx = rng.random_range(cur_cost_idx..5);
                    let next_cost_l = cost_idx_to_l(next_cost_idx);

                    if let Some(&(next_x, next_l)) = min_actions.get(i + 1) {
                        if min_actions[i].0 + next_cost_l > next_x {
                            // 右隣と重なったら、2枚を覆える最短の1枚に統合する。
                            let need_l = next_x + next_l - min_actions[i].0;
                            let merged_l = (1..=9).step_by(2).find(|&l| l >= need_l);
                            let next_next_x =
                                min_actions.get(i + 2).map(|action| action.0).unwrap_or(W);

                            if let Some(merged_l) = merged_l {
                                if min_actions[i].0 + merged_l <= next_next_x
                                    && min_actions[i].0 + merged_l <= W
                                {
                                    cur_actions.push((min_actions[i].0, merged_l));
                                    cur_cost += input.costs[l_to_cost_idx(merged_l)];
                                    i += 2;
                                    continue;
                                }
                            }
                        } else if min_actions[i].0 + next_cost_l <= W {
                            cur_actions.push((min_actions[i].0, next_cost_l));
                            cur_cost += input.costs[next_cost_idx];
                            i += 1;
                            continue;
                        }
                    } else if min_actions[i].0 + next_cost_l <= W {
                        cur_actions.push((min_actions[i].0, next_cost_l));
                        cur_cost += input.costs[next_cost_idx];
                        i += 1;
                        continue;
                    }
                }

                // 統合できない、または壁からはみ出す場合は元の板を残す。
                cur_actions.push(min_actions[i]);
                cur_cost += input.costs[cur_cost_idx];
                i += 1;
            }

            res.push((cur_actions, cur_cost));
        }

        res.sort_unstable();
        res.dedup();
        res
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct Node {
    vertex: usize,
    cost: i64,
}
impl Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.cost.cmp(&self.cost)
    }
}
impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
pub fn dijkstra(row_holes: &[usize], costs: &[i64]) -> (Vec<(usize, usize)>, i64) {
    let inf: i64 = 1 << 60;
    let n = row_holes.len();
    let mut dist = vec![inf; n + 1];
    let mut prev = vec![(0, 0); n + 1];
    let mut pq = std::collections::BinaryHeap::new();

    dist[0] = 0;
    pq.push(Node { vertex: 0, cost: 0 });
    while let Some(Node { vertex, cost }) = pq.pop() {
        if dist[vertex] < cost {
            continue;
        }
        for (cost_idx, &edge_cost) in costs.iter().enumerate() {
            let new_cost = cost + edge_cost;
            let l = cost_idx_to_l(cost_idx);
            let next_vertex = row_holes.upper_bound(&(row_holes[vertex] + l - 1));
            if new_cost < dist[next_vertex] {
                dist[next_vertex] = new_cost;
                prev[next_vertex] = (vertex, l);
                if next_vertex < n {
                    pq.push(Node {
                        vertex: next_vertex,
                        cost: new_cost,
                    });
                }
            }
        }
    }

    let mut actions = vec![];
    let mut v = n;
    while prev[v] != (0, 0) {
        let (prev_v, prev_l) = prev[v];
        actions.push((
            row_holes[prev_v] - (row_holes[prev_v] + prev_l).saturating_sub(W),
            prev_l,
        ));
        v = prev_v;
    }
    actions.reverse();
    (actions, dist[n])
}
