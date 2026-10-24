//! Shared input must not require Clone; all engines use the same input reference.
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;

mod framework {
    pub mod constructive {
        include!("framework/constructive.rs");
    }
    pub mod beam {
        include!("framework/beam.rs");
    }
    pub mod beam_fast {
        include!("framework/beam_fast.rs");
    }
    pub mod random_search {
        include!("framework/random_search.rs");
    }
    pub mod local_search {
        include!("framework/local_search.rs");
        pub fn fixed_budget() -> SearchBudget {
            SearchBudget {
                started_at: std::time::Instant::now(),
                time_limit: std::time::Duration::from_secs(1),
                iteration_limit: Some(100),
            }
        }
    }
}

// Deliberately no Clone, Debug, or PartialEq: Input is not part of State.
struct Input {
    rewards: Vec<i64>,
    target: i64,
}

#[derive(Clone, Debug, PartialEq)]
struct Construction {
    actions: Vec<usize>,
    score: i64,
}

impl framework::constructive::ConstructiveState for Construction {
    type Input = Input;
    type Action = usize;
    fn is_complete(&self, input: &Input) -> bool {
        self.actions.len() == input.rewards.len()
    }
    fn legal_actions(&self, _: &Input) -> Vec<usize> {
        vec![0, 1]
    }
    fn advance(&mut self, input: &Input, action: &usize) {
        self.score += input.rewards[self.actions.len()] * *action as i64;
        self.actions.push(*action);
    }
    fn evaluated_value(&self, _: &Input) -> i64 {
        self.score
    }
    fn debug_validate(&self, input: &Input) {
        assert_eq!(
            self.score,
            self.actions
                .iter()
                .zip(&input.rewards)
                .map(|(&a, &b)| a as i64 * b)
                .sum()
        );
    }
}

#[test]
fn construction_engines_share_non_clone_input() {
    let input = Input {
        rewards: vec![3, -4, 5],
        target: 3,
    };
    let initial = Construction {
        actions: vec![],
        score: 0,
    };
    let greedy = framework::constructive::solve_greedy(&input, initial.clone());
    assert_eq!(greedy.actions, vec![1, 0, 1]);
    assert_eq!(greedy.score, 8);
    assert_eq!(
        framework::beam::solve_beam(&input, initial.clone(), 4),
        greedy
    );
    assert_eq!(
        framework::beam_fast::solve_beam_fast(&input, initial.clone(), 4),
        greedy
    );
    let mut rng = Pcg64Mcg::seed_from_u64(42);
    assert_eq!(
        framework::random_search::solve_random_search(&input, &initial, 100, &mut rng),
        greedy
    );
}

struct PreviewInput {
    clones: std::cell::Cell<usize>,
    depth: usize,
    branching: usize,
}

struct PreviewState<'a> {
    input: &'a PreviewInput,
    path: Vec<usize>,
    score: i64,
    incorrect_preview: bool,
}

impl Clone for PreviewState<'_> {
    fn clone(&self) -> Self {
        self.input.clones.set(self.input.clones.get() + 1);
        Self {
            input: self.input,
            path: self.path.clone(),
            score: self.score,
            incorrect_preview: self.incorrect_preview,
        }
    }
}

impl framework::constructive::ConstructiveState for PreviewState<'_> {
    type Input = PreviewInput;
    type Action = usize;
    fn is_complete(&self, input: &PreviewInput) -> bool {
        self.path.len() == input.depth
    }
    fn legal_actions(&self, input: &PreviewInput) -> Vec<usize> {
        (0..input.branching).collect()
    }
    fn advance(&mut self, _: &PreviewInput, action: &usize) {
        self.score += *action as i64;
        self.path.push(*action);
    }
    fn evaluated_value(&self, _: &PreviewInput) -> i64 {
        self.score
    }
    fn evaluated_value_after(&self, _: &PreviewInput, action: &usize) -> i64 {
        // 不一致テストでは棄却される低評価の候補だけを意図的に壊す。
        self.score + *action as i64
            - if self.incorrect_preview && *action == 0 {
                1000
            } else {
                0
            }
    }
}

fn preview_state(input: &PreviewInput, incorrect_preview: bool) -> PreviewState<'_> {
    PreviewState {
        input,
        path: Vec::new(),
        score: 0,
        incorrect_preview,
    }
}

#[test]
fn lightweight_evaluation_preserves_the_best_complete_path() {
    let input = PreviewInput {
        clones: std::cell::Cell::new(0),
        depth: 4,
        branching: 50,
    };
    let greedy = framework::constructive::solve_greedy(&input, preview_state(&input, false));
    let beam = framework::beam::solve_beam(&input, preview_state(&input, false), 3);
    let fast = framework::beam_fast::solve_beam_fast(&input, preview_state(&input, false), 3);
    for state in [greedy, beam, fast] {
        assert_eq!(state.path, vec![49; 4]);
        assert_eq!(state.score, 196);
    }
}

#[test]
#[cfg(not(debug_assertions))]
fn release_copies_only_retained_states_with_lightweight_evaluation() {
    let input = PreviewInput {
        clones: std::cell::Cell::new(0),
        depth: 4,
        branching: 50,
    };
    framework::constructive::solve_greedy(&input, preview_state(&input, false));
    assert_eq!(input.clones.get(), 0);
    framework::beam::solve_beam(&input, preview_state(&input, false), 3);
    assert_eq!(input.clones.replace(0), 4 * 3);
    framework::beam_fast::solve_beam_fast(&input, preview_state(&input, false), 3);
    // 最初の状態の保持に1回、その後は各層の採用3状態だけ。
    assert_eq!(input.clones.get(), 1 + 4 * 3);
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "evaluated_value_after must match")]
fn debug_checks_lightweight_scores_even_for_discarded_actions() {
    let input = PreviewInput {
        clones: std::cell::Cell::new(0),
        depth: 1,
        branching: 2,
    };
    framework::beam::solve_beam(&input, preview_state(&input, true), 1);
}

#[test]
fn beam_ties_keep_the_same_generation_order_in_both_engines() {
    let input = Input {
        rewards: vec![0, 0, 0],
        target: 0,
    };
    let initial = Construction {
        actions: vec![],
        score: 0,
    };
    let beam = framework::beam::solve_beam(&input, initial.clone(), 3);
    let fast = framework::beam_fast::solve_beam_fast(&input, initial, 3);
    assert_eq!(beam.actions, vec![0, 0, 0]);
    assert_eq!(beam, fast);
}

#[derive(Debug, Clone, PartialEq)]
struct Local {
    parameter: i64,
    output: i64,
    score: i64,
}

#[derive(Debug, Clone, PartialEq)]
struct StaleOutput(Local);

impl framework::local_search::LocalSearchState for StaleOutput {
    type Input = Input;
    type Move = i64;
    fn evaluated_value(&self, _: &Input) -> i64 {
        self.0.score
    }
    fn propose_move<R: Rng + ?Sized>(&self, _: &Input, _: &mut R) -> Option<i64> {
        Some(1)
    }
    fn apply_move(&mut self, input: &Input, movement: &i64) {
        self.0.parameter += movement;
        // 意図的なバグ: スコアだけ更新し、解の再構築を忘れる。
        self.0.score = -(self.0.parameter * 2 - input.target * 2).abs();
    }
    fn undo_move(&mut self, input: &Input, movement: &i64) {
        self.0.parameter -= movement;
        self.0.rebuild(input);
    }
    fn debug_validate(&self, input: &Input) {
        self.0.debug_validate(input);
    }
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "assertion")]
fn debug_search_detects_score_updated_but_output_stale() {
    use framework::local_search::{fixed_budget, optimize, Acceptance};
    let input = Input {
        rewards: vec![],
        target: 3,
    };
    let state = StaleOutput(Local {
        parameter: 0,
        output: 0,
        score: -6,
    });
    let mut rng = Pcg64Mcg::seed_from_u64(42);
    optimize(
        &input,
        state,
        Acceptance::HillClimbing,
        fixed_budget(),
        &mut rng,
    );
}
impl Local {
    fn rebuild(&mut self, input: &Input) {
        self.output = self.parameter * 2;
        self.score = -(self.output - input.target * 2).abs();
    }
}
impl framework::local_search::LocalSearchState for Local {
    type Input = Input;
    type Move = i64;
    fn evaluated_value(&self, _: &Input) -> i64 {
        self.score
    }
    fn propose_move<R: Rng + ?Sized>(&self, _: &Input, rng: &mut R) -> Option<i64> {
        Some(if rng.random::<bool>() { 1 } else { -1 })
    }
    fn apply_move(&mut self, input: &Input, movement: &i64) {
        self.parameter += movement;
        self.rebuild(input);
    }
    fn undo_move(&mut self, input: &Input, movement: &i64) {
        self.parameter -= movement;
        self.rebuild(input);
    }
    fn debug_validate(&self, input: &Input) {
        let mut expected = self.clone();
        expected.rebuild(input);
        assert_eq!(*self, expected);
    }
}

#[test]
fn hill_and_annealing_rebuild_and_restore_output() {
    use framework::local_search::{fixed_budget, optimize, Acceptance, LocalSearchState};
    let input = Input {
        rewards: vec![],
        target: 3,
    };
    let mut initial = Local {
        parameter: 0,
        output: 0,
        score: 0,
    };
    initial.rebuild(&input);
    let before = initial.clone();
    initial.apply_move(&input, &2);
    assert_eq!(initial.output, 4);
    initial.undo_move(&input, &2);
    assert_eq!(initial, before);
    for acceptance in [
        Acceptance::HillClimbing,
        Acceptance::SimulatedAnnealing {
            start_temperature: 2.0,
            end_temperature: 0.01,
        },
    ] {
        let mut rng = Pcg64Mcg::seed_from_u64(42);
        let best = optimize(
            &input,
            initial.clone(),
            acceptance,
            fixed_budget(),
            &mut rng,
        );
        best.debug_validate(&input);
        assert_eq!(best.score, 0);
        assert_eq!(best.output, 6);
    }
}
