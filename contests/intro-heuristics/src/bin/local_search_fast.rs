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

    // デバッグビルドでは、差分更新した得点と全体再計算が一致することを確認する。
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

fn interval_cost(prev: Option<usize>, next: Option<usize>, days: usize) -> i64 {
    // contest_days はすべて 0-indexed。
    // 先頭側の番兵を -1、末尾側の番兵を days とみなして、
    // その区間で発生する不満足度の係数を計算する。
    let left = prev.map_or(-1_i64, |day| day as i64);
    let right = next.map_or(days as i64, |day| day as i64);
    let distance = right - left;
    distance * (distance - 1) / 2
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

    // 各コンテストを開催する0-indexedの日付を、得点の差分計算用に昇順で保持する。
    // 番兵は保存せず、区間の端では-1とinput.daysを使う。
    contest_days: Vec<Vec<usize>>,
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
        let mut contest_days = vec![vec![]; CONTEST_TYPES];
        for (day, &action) in output.iter().enumerate() {
            contest_days[action].push(day);
        }
        Self {
            output,
            score,
            contest_days,
        }
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

    // 選んだ近傍を適用し、得点を差分更新する。
    // 得点計算を高速化しても、このメソッドのインターフェースは変えない。
    fn apply_move(&mut self, input: &Input, selected_move: Move) {
        match selected_move {
            Move::Change {
                day,
                old_action,
                new_action,
            } => {
                debug_assert_eq!(self.output[day], old_action);
                self.change(input, day, old_action, new_action);
            }
            Move::Swap { day1, day2 } => {
                let day1_action = self.output[day1];
                let day2_action = self.output[day2];
                self.change(input, day1, day1_action, day2_action);
                self.change(input, day2, day2_action, day1_action);
            }
        }

        // 重い全体再計算との照合はデバッグビルドだけで行う。
        debug_assert_eq!(self.score, calculate_score(input, &self.output));
    }

    // 不採用になった操作を元に戻す。
    // 得点は変更前の値が分かっているため再計算せず、解と補助情報だけを復元する。
    fn undo_move(&mut self, selected_move: Move, old_score: i64) {
        match selected_move {
            Move::Change {
                day,
                old_action,
                new_action,
            } => {
                debug_assert_eq!(self.output[day], new_action);
                self.replace_action(day, new_action, old_action);
            }
            Move::Swap { day1, day2 } => {
                let day1_action = self.output[day1];
                let day2_action = self.output[day2];

                // apply_move と逆順で戻す。
                self.replace_action(day2, day2_action, day1_action);
                self.replace_action(day1, day1_action, day2_action);
            }
        }

        self.score = old_score;
    }

    // 1日分の開催内容を変更したときの得点差を計算する。
    fn change(&mut self, input: &Input, day: usize, old_action: usize, new_action: usize) {
        if old_action == new_action {
            return;
        }

        debug_assert_eq!(self.output[day], old_action);

        // contest_days も output と同じ 0-indexed で保持する。
        let old_pos = self.contest_days[old_action]
            .binary_search(&day)
            .expect("day must exist in contest_days[old_action]");

        let old_prev = old_pos
            .checked_sub(1)
            .map(|pos| self.contest_days[old_action][pos]);
        let old_next = self.contest_days[old_action].get(old_pos + 1).copied();

        // 開催日を削除すると、2区間が1区間に結合される。
        self.score += (interval_cost(old_prev, Some(day), input.days)
            + interval_cost(Some(day), old_next, input.days)
            - interval_cost(old_prev, old_next, input.days))
            * input.decay[old_action];

        // new_action の開催日列も昇順の 0-indexed。
        let new_pos = self.contest_days[new_action].partition_point(|&d| d < day);
        let new_prev = new_pos
            .checked_sub(1)
            .map(|pos| self.contest_days[new_action][pos]);
        let new_next = self.contest_days[new_action].get(new_pos).copied();

        // 開催日を追加すると、1区間が2区間に分割される。
        self.score -= (interval_cost(new_prev, Some(day), input.days)
            + interval_cost(Some(day), new_next, input.days)
            - interval_cost(new_prev, new_next, input.days))
            * input.decay[new_action];

        self.score += input.satisfaction[day][new_action] - input.satisfaction[day][old_action];
        self.replace_action(day, old_action, new_action);
    }

    // 得点を変更せず、解と差分計算用の開催日だけを同時に更新する。
    // apply_moveとundo_moveの両方がこのメソッドを使い、2つの表現を常に一致させる。
    fn replace_action(&mut self, day: usize, old_action: usize, new_action: usize) {
        if old_action == new_action {
            return;
        }

        debug_assert_eq!(self.output[day], old_action);

        let old_pos = self.contest_days[old_action]
            .binary_search(&day)
            .expect("day must exist in contest_days[old_action]");
        self.contest_days[old_action].remove(old_pos);

        let new_pos = self.contest_days[new_action].partition_point(|&d| d < day);
        self.contest_days[new_action].insert(new_pos, day);
        self.output[day] = new_action;

        self.debug_assert_contest_days_consistent();
    }

    // outputとcontest_daysが同じ0-indexedの開催予定を表していることを検査する。
    // cfg(debug_assertions)内はreleaseビルドから除かれる。
    fn debug_assert_contest_days_consistent(&self) {
        #[cfg(debug_assertions)]
        {
            let mut expected = vec![vec![]; CONTEST_TYPES];
            for (day, &action) in self.output.iter().enumerate() {
                expected[action].push(day);
            }
            debug_assert_eq!(self.contest_days, expected);
        }
    }
}

// 山登り: 2,333,714
// 山登り swapあり: 2,595,870
// 焼きなまし swapあり: 2,618,311
// 焼きなまし swapあり 高速化: 2,710,719
