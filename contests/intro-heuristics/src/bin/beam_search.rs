// 参考資料（ビームサーチ演習）:
// https://img.atcoder.jp/ahf2-a7k3m9q2/ahf002-1answer.pdf

use itertools::Itertools;
use proconio::input;

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
            next_state.advance(input, action);

            if next_state.evaluated_score > best_score {
                best_score = next_state.evaluated_score;
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

        next_beam.sort_unstable_by(|left, right| right.evaluated_score.cmp(&left.evaluated_score));
        next_beam.truncate(beam_width);
        beam = next_beam;
    }

    beam.swap_remove(0)
}

fn main() {
    let input = read_input();
    // let state = solve_greedy(&input, State::new());
    let state = solve_beam(&input, State::new(), BEAM_WIDTH);

    // デバッグビルドでは、State内の得点と完成解の全体再計算を照合する。
    // debug_assert_eq!はreleaseビルドでは実行されない。
    debug_assert_eq!(state.actual_score, calculate_score(&input, &state.output));
    print_output(&state.output);
}

// ============================================================================
// 問題ごとに実装する部分
//
// 入出力と得点計算を用意し、部分解の持ち方、操作列挙、状態更新の順に実装する。
// ============================================================================

const CONTEST_TYPES: usize = 26;
const BEAM_WIDTH: usize = 500;
const INITIAL_SCORE: i64 = 1_000_000;

#[derive(Debug)]
struct Input {
    days: usize,
    decay: Vec<i64>,
    satisfaction: Vec<Vec<i64>>,
}

fn read_input() -> Input {
    input! {
        days: usize,
        decay: [i64; CONTEST_TYPES],
        satisfaction: [[i64; CONTEST_TYPES]; days],
    }

    Input {
        days,
        decay,
        satisfaction,
    }
}

type Output = Vec<usize>;
fn print_output(output: &Output) {
    println!("{}", output.iter().map(|contest| contest + 1).join("\n"));
}

fn calculate_score(input: &Input, output: &Output) -> i64 {
    let mut score = INITIAL_SCORE;
    let mut last_days = [-1_i64; CONTEST_TYPES];

    for (day, &action) in output.iter().enumerate() {
        score += input.satisfaction[day][action];
        last_days[action] = day as i64;

        for (contest, &last_day) in last_days.iter().enumerate() {
            score -= input.decay[contest] * (day as i64 - last_day);
        }
    }

    score
}

// 先頭から1日ずつ操作を追加していく部分解。
// solve_greedyとsolve_beamはevaluated_scoreで候補を比較し、mainはoutputを出力する。
// actual_score、day、last_daysはState自身が状態を進めるために管理する。
#[derive(Debug, Clone)]
struct State {
    // これまでに選んだ操作列。探索終了時にそのまま提出形式で出力する。
    output: Output,
    // 候補を残す順番を決めるための評価値。
    evaluated_score: i64,

    // ここから下は、State::advanceで状態を1日進めるために使う。
    actual_score: i64,
    day: usize,
    last_days: [i64; CONTEST_TYPES],
}

impl State {
    fn new() -> Self {
        Self {
            output: Vec::new(),
            evaluated_score: INITIAL_SCORE,
            actual_score: INITIAL_SCORE,
            day: 0,
            last_days: [-1; CONTEST_TYPES],
        }
    }

    // 最後まで操作を選び終えたか判定する。
    fn is_done(&self, input: &Input) -> bool {
        self.day == input.days
    }

    // 指定した操作で状態を1日進め、実際の得点と探索用の評価値を更新する。
    fn advance(&mut self, input: &Input, action: usize) {
        self.actual_score += input.satisfaction[self.day][action];
        self.last_days[action] = self.day as i64;

        for (contest, &last_day) in self.last_days.iter().enumerate() {
            self.actual_score -= input.decay[contest] * (self.day as i64 - last_day);
        }

        // 演習2: 探索途中の評価方法を工夫する場所。
        self.evaluated_score = self.actual_score;
        for d in self.day + 1..(self.day + 1 + 5).min(input.days) {
            for (contest, &last_day) in self.last_days.iter().enumerate() {
                self.evaluated_score -= input.decay[contest] * (d as i64 - last_day);
            }
        }

        self.output.push(action);
        self.day += 1;
    }

    // 現在の部分解から次に試す操作を列挙する。
    fn legal_actions(&self, _input: &Input) -> Vec<usize> {
        // 演習2: 探索対象にする操作を絞り込む場所。
        (0..CONTEST_TYPES).collect()
    }
}

// 工夫前,600: 2,545,729
// minus,600: 2,556,438
// plus,600: 2,591,613
// 365,600: 2,591,613
// future,500: 2,640,570
