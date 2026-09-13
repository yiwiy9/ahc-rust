// 参考資料（Introduction to Heuristics Contest 公式解説）:
// https://img.atcoder.jp/intro-heuristics/editorial.pdf
// 参考資料（山登り法・焼きなまし法）:
// https://img.atcoder.jp/ahf1/ahf001-2.pdf

use std::time::{Duration, Instant};

use itertools::Itertools;
use proconio::input;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;

const SCORE_DIFF_SAMPLE_LIMIT: usize = 10_000;

// ============================================================================
// 局所探索の共通部分
//
// 問題固有の処理はStateに任せ、この範囲では近傍の採用・巻き戻しと
// 制限時間の管理だけを行う。新しい問題ではmainより下を実装する。
// ============================================================================

// 山登り法から始める場合でも、後から焼きなまし法へ切り替えられるよう両方定義する。
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
enum SearchMethod {
    HillClimbing,
    SimulatedAnnealing {
        start_temperature: f64,
        end_temperature: f64,
    },
}

#[derive(Debug, Clone, Copy)]
struct SearchConfig {
    search_method: SearchMethod,
    time_limit_seconds: f64,
    random_seed: u64,
}

// 提案された改悪Moveの大きさを集計し、焼きなまし法の温度候補を出す。
// eprintln!は標準エラーへ出力するため、提出する解には混ざらない。
#[derive(Debug)]
struct ScoreDiffStats {
    worsening_diffs: Vec<i64>,
}

impl ScoreDiffStats {
    fn new() -> Self {
        Self {
            worsening_diffs: Vec::with_capacity(SCORE_DIFF_SAMPLE_LIMIT),
        }
    }

    fn record(&mut self, score_diff: i64) {
        if score_diff < 0 && self.worsening_diffs.len() < SCORE_DIFF_SAMPLE_LIMIT {
            self.worsening_diffs.push(score_diff.saturating_neg());
        }
    }

    fn print_summary(&mut self) {
        if self.worsening_diffs.is_empty() {
            eprintln!("worsening_diff: samples=0");
            return;
        }

        self.worsening_diffs.sort_unstable();

        let p50 = self.worsening_diffs[self.worsening_diffs.len() / 2];
        let p90 = self.worsening_diffs[(self.worsening_diffs.len() - 1) * 9 / 10];

        eprintln!(
            "worsening_diff: samples={} p50={} p90={}",
            self.worsening_diffs.len(),
            p50,
            p90
        );
        eprintln!(
            "temperature_hint: start={:.1} end={:.1}",
            p50 as f64 * 1.5,
            p50 as f64 * 0.1
        );
    }
}

#[derive(Debug)]
struct TimeKeeper {
    started_at: Instant,
    time_limit: Duration,
}

struct LocalSearch {
    current_state: State,
    best_state: State,
    search_method: SearchMethod,
    rng: Pcg64Mcg,
    time_keeper: TimeKeeper,
    iteration_count: usize,
    accepted_count: usize,
}

impl TimeKeeper {
    fn new(time_limit_seconds: f64) -> Self {
        Self {
            started_at: Instant::now(),
            time_limit: Duration::from_secs_f64(time_limit_seconds),
        }
    }

    fn is_time_over(&self) -> bool {
        self.started_at.elapsed() >= self.time_limit
    }

    // 焼きなまし法の温度更新に使う。開始時は0、終了時は1になる。
    fn progress(&self) -> f64 {
        (self.started_at.elapsed().as_secs_f64() / self.time_limit.as_secs_f64()).min(1.0)
    }
}

impl LocalSearch {
    fn new(initial_state: State, config: SearchConfig) -> Self {
        Self {
            best_state: initial_state.clone(),
            current_state: initial_state,
            search_method: config.search_method,
            rng: Pcg64Mcg::seed_from_u64(config.random_seed),
            time_keeper: TimeKeeper::new(config.time_limit_seconds),
            iteration_count: 0,
            accepted_count: 0,
        }
    }

    fn solve(mut self, input: &Input) -> State {
        let mut score_diff_stats = ScoreDiffStats::new();

        while !self.time_keeper.is_time_over() {
            let selected_move = self.current_state.generate_move(input, &mut self.rng);
            let old_score = self.current_state.score;

            self.current_state.apply_move(input, selected_move);
            let score_diff = self.current_state.score - old_score;
            score_diff_stats.record(score_diff);

            if self.should_accept(score_diff) {
                self.accepted_count += 1;

                if self.current_state.score > self.best_state.score {
                    self.best_state = self.current_state.clone();
                }
            } else {
                self.current_state.undo_move(selected_move, old_score);
            }

            self.iteration_count += 1;
        }

        eprintln!(
            "iterations={} accepted={} best_score={}",
            self.iteration_count, self.accepted_count, self.best_state.score
        );
        score_diff_stats.print_summary();

        // 焼きなまし法では現在の解が悪化することがあるため、最良解を別に保存して返す。
        self.best_state
    }

    // 山登り法は改善のみ、焼きなまし法は温度に応じて悪化も受け入れる。
    fn should_accept(&mut self, score_diff: i64) -> bool {
        match self.search_method {
            SearchMethod::HillClimbing => score_diff >= 0,
            SearchMethod::SimulatedAnnealing {
                start_temperature,
                end_temperature,
            } => {
                let progress = self.time_keeper.progress();
                let temperature =
                    start_temperature.powf(1.0 - progress) * end_temperature.powf(progress);
                score_diff >= 0
                    || self.rng.random::<f64>() < (score_diff as f64 / temperature).exp()
            }
        }
    }
}

fn main() {
    let input = read_input();
    let output = solve_greedy(&input);
    let initial_state = State::new(&input, output);

    let config = SearchConfig {
        // search_method: SearchMethod::HillClimbing,
        search_method: SearchMethod::SimulatedAnnealing {
            start_temperature: 2_000.0,
            end_temperature: 600.0,
        },
        time_limit_seconds: 1.8,
        random_seed: 890482,
    };

    let state = LocalSearch::new(initial_state, config).solve(&input);

    // デバッグビルドでは、Stateの得点と全体再計算が一致することを確認する。
    // debug_assert_eq!はreleaseビルドでは実行されない。
    debug_assert_eq!(state.score, calculate_score(&input, &state.output));
    print_output(&state.output);
}

// ============================================================================
// 問題ごとに実装する部分
//
// 入出力と得点計算を用意し、初期解、状態、近傍の順に実装する。
// ============================================================================

const CONTEST_TYPES: usize = 26;
const INITIAL_SCORE: i64 = 1_000_000;
const TWO_POINT_SWAP_PROBABILITY: f64 = 0.5;
const TWO_POINT_SWAP_MAX_DISTANCE: usize = 16;

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

// 局所探索を開始するための完成解を作る。
fn solve_greedy(input: &Input) -> Output {
    let mut output = Vec::with_capacity(input.days);
    let mut last_days = [-1_i64; CONTEST_TYPES];

    for day in 0..input.days {
        // 選ばない種類の不満足度はどの候補でも同じなので、
        // 当日の満足度と、選ぶことで防げる不満足度だけを比較する。
        let action = (0..CONTEST_TYPES)
            .max_by_key(|&contest| {
                input.satisfaction[day][contest]
                    + input.decay[contest] * (day as i64 - last_days[contest])
            })
            .unwrap();

        output.push(action);
        last_days[action] = day as i64;
    }

    output
}

// 局所探索中の完成解と得点を保持する。
// 差分計算に補助情報が必要な問題では、この構造体へフィールドを追加する。
#[derive(Debug, Clone)]
struct State {
    // 最後に提出形式で出力する解。
    output: Output,
    // 現在の解の得点。近傍の採用判定と最良解の比較に使う。
    score: i64,
}

// 現在の解を少しだけ変化させる操作。
#[derive(Debug, Clone, Copy)]
enum Move {
    Change {
        day: usize,
        old_action: usize,
        new_action: usize,
    },
    Swap {
        day1: usize,
        day2: usize,
    },
}

impl State {
    fn new(input: &Input, output: Output) -> Self {
        let score = calculate_score(input, &output);
        Self { output, score }
    }

    // 近傍の種類と、操作を選ぶ確率を問題に合わせて決める。
    fn generate_move<R: Rng>(&self, input: &Input, rng: &mut R) -> Move {
        if input.days >= 2 && rng.random_bool(TWO_POINT_SWAP_PROBABILITY) {
            let day1 = rng.random_range(0..input.days - 1);
            let last_day2 = (day1 + TWO_POINT_SWAP_MAX_DISTANCE).min(input.days - 1);
            let day2 = rng.random_range(day1 + 1..=last_day2);
            Move::Swap { day1, day2 }
        } else {
            let day = rng.random_range(0..input.days);
            let old_action = self.output[day];
            let new_action = rng.random_range(0..CONTEST_TYPES);
            Move::Change {
                day,
                old_action,
                new_action,
            }
        }
    }

    // 最初は変更後の得点を全体から再計算する。
    // 反復回数が必要になったら、インターフェースを変えずに内部を差分更新へ変更する。
    fn apply_move(&mut self, input: &Input, selected_move: Move) {
        match selected_move {
            Move::Change {
                day,
                old_action,
                new_action,
            } => {
                debug_assert_eq!(self.output[day], old_action);
                self.output[day] = new_action;
            }
            Move::Swap { day1, day2 } => {
                self.output.swap(day1, day2);
            }
        }

        self.score = calculate_score(input, &self.output);
    }

    // 不採用になった操作を元に戻す。
    // 得点は変更前の値が分かっているため、再計算せずold_scoreを復元する。
    fn undo_move(&mut self, selected_move: Move, old_score: i64) {
        match selected_move {
            Move::Change {
                day,
                old_action,
                new_action,
            } => {
                debug_assert_eq!(self.output[day], new_action);
                self.output[day] = old_action;
            }
            Move::Swap { day1, day2 } => {
                self.output.swap(day1, day2);
            }
        }

        self.score = old_score;
    }
}
