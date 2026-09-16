use std::time::{Duration, Instant};

use rand::Rng;

/// 問題固有の解・スコア・近傍操作を、探索ループから分離する。
pub trait LocalSearchState: Clone + std::fmt::Debug + PartialEq {
    type Input;
    type Move: Clone;

    /// 探索内部の値。貪欲やビームと同様に「大きいほどよい」にそろえる。
    fn evaluated_value(&self, input: &Self::Input) -> i64;
    fn propose_move<R: Rng + ?Sized>(&self, input: &Self::Input, rng: &mut R)
        -> Option<Self::Move>;
    fn apply_move(&mut self, input: &Self::Input, movement: &Self::Move);
    fn undo_move(&mut self, input: &Self::Input, movement: &Self::Move);
    fn debug_validate(&self, _input: &Self::Input) {}
}

#[derive(Debug, Clone, Copy)]
pub enum Acceptance {
    HillClimbing,
    SimulatedAnnealing {
        start_temperature: f64,
        end_temperature: f64,
    },
}

/// 本番は時間、再現テストはAHC_ITERATIONS環境変数による固定反復を使える。
#[derive(Debug, Clone)]
pub struct SearchBudget {
    started_at: Instant,
    time_limit: Duration,
    iteration_limit: Option<u64>,
}

impl SearchBudget {
    /// main冒頭で作ると、入力・初期解生成を含む全体時間から探索残量を判断できる。
    pub fn from_env(started_at: Instant, time_limit: Duration) -> Self {
        let iteration_limit = std::env::var("AHC_ITERATIONS")
            .ok()
            .and_then(|value| value.parse().ok());
        Self {
            started_at,
            time_limit,
            iteration_limit,
        }
    }

    fn progress(&self, iteration: u64) -> f64 {
        if let Some(limit) = self.iteration_limit {
            return (iteration as f64 / limit.max(1) as f64).clamp(0.0, 1.0);
        }
        (self.started_at.elapsed().as_secs_f64() / self.time_limit.as_secs_f64()).clamp(0.0, 1.0)
    }

    fn has_next(&self, iteration: u64) -> bool {
        if let Some(limit) = self.iteration_limit {
            iteration < limit
        } else {
            self.started_at.elapsed() < self.time_limit
        }
    }
}

/// 山登りと焼きなましの共通ループ。問題ごとの差はLocalSearchStateへ閉じ込める。
pub fn optimize<S, R>(
    input: &S::Input,
    mut current: S,
    acceptance: Acceptance,
    budget: SearchBudget,
    rng: &mut R,
) -> S
where
    S: LocalSearchState,
    R: Rng + ?Sized,
{
    let mut best = current.clone();
    let mut current_value = current.evaluated_value(input);
    let mut best_value = current_value;
    let mut iteration = 0;
    let collect_deltas = std::env::var_os("AHC_SAMPLE_DELTAS").is_some();
    let mut bad_deltas = Vec::with_capacity(if collect_deltas { 10_000 } else { 0 });

    if let Acceptance::SimulatedAnnealing {
        start_temperature,
        end_temperature,
    } = acceptance
    {
        assert!(start_temperature > 0.0 && end_temperature > 0.0);
    }

    while budget.has_next(iteration) {
        let Some(movement) = current.propose_move(input, rng) else {
            break;
        };
        #[cfg(debug_assertions)]
        let before_move = current.clone();
        current.apply_move(input, &movement);
        // 棄却する候補も検算する。debugでは速さより更新漏れの発見を優先する。
        #[cfg(debug_assertions)]
        current.debug_validate(input);
        let next_value = current.evaluated_value(input);
        let delta = next_value - current_value;
        if collect_deltas {
            if delta < 0 && bad_deltas.len() < 10_000 {
                bad_deltas.push(-delta);
            }
            current.undo_move(input, &movement);
            #[cfg(debug_assertions)]
            debug_assert_eq!(current, before_move, "apply_move + undo_move changed State");
            iteration += 1;
            continue;
        }
        let progress = budget.progress(iteration);
        let accepted = match acceptance {
            Acceptance::HillClimbing => delta >= 0,
            Acceptance::SimulatedAnnealing {
                start_temperature,
                end_temperature,
            } => {
                let temperature =
                    start_temperature.powf(1.0 - progress) * end_temperature.powf(progress);
                delta >= 0 || rng.random::<f64>() < (delta as f64 / temperature).exp()
            }
        };

        if accepted {
            current_value = next_value;
            if current_value > best_value {
                best = current.clone();
                best_value = current_value;
            }
        } else {
            current.undo_move(input, &movement);
            #[cfg(debug_assertions)]
            debug_assert_eq!(current, before_move, "apply_move + undo_move changed State");
        }

        #[cfg(debug_assertions)]
        if iteration % 100 == 0 {
            current.debug_validate(input);
        }
        iteration += 1;
    }

    #[cfg(debug_assertions)]
    best.debug_validate(input);
    if collect_deltas {
        print_delta_stats(&mut bad_deltas);
    }
    best
}

fn print_delta_stats(values: &mut [i64]) {
    if values.is_empty() {
        eprintln!("AHC_DELTA_SAMPLES count=0");
        return;
    }
    values.sort_unstable();
    let median = values[values.len() / 2];
    let p90 = values[values.len() * 9 / 10];
    eprintln!(
        "AHC_DELTA_SAMPLES count={} median={} p90={} start_candidates=[{:.1},{:.1},{:.1}] end_candidates=[{:.2},{:.2},{:.2}]",
        values.len(),
        median,
        p90,
        median as f64,
        median as f64 * 2.0,
        median as f64 * 5.0,
        median as f64 * 0.005,
        median as f64 * 0.02,
        median as f64 * 0.1,
    );
}
