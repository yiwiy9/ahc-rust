//! Shared input must not require Clone; all engines use the same input reference.
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;

mod framework {
    pub mod constructive {
        include!("../src/framework/constructive.rs");
    }
    pub mod beam {
        include!("../src/framework/beam.rs");
    }
    pub mod beam_fast {
        include!("../src/framework/beam_fast.rs");
    }
    pub mod random_search {
        include!("../src/framework/random_search.rs");
    }
    pub mod local_search {
        include!("../src/framework/local_search.rs");
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
    fn evaluated_value(&self, _: &Input) -> i64 { self.0.score }
    fn propose_move<R: Rng + ?Sized>(&self, _: &Input, _: &mut R) -> Option<i64> { Some(1) }
    fn apply_move(&mut self, input: &Input, movement: &i64) {
        self.0.parameter += movement;
        // 意図的なバグ: スコアだけ更新し、解の再構築を忘れる。
        self.0.score = -(self.0.parameter * 2 - input.target * 2).abs();
    }
    fn undo_move(&mut self, input: &Input, movement: &i64) {
        self.0.parameter -= movement;
        self.0.rebuild(input);
    }
    fn debug_validate(&self, input: &Input) { self.0.debug_validate(input); }
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "assertion")]
fn debug_search_detects_score_updated_but_output_stale() {
    use framework::local_search::{optimize, Acceptance, fixed_budget};
    let input = Input { rewards: vec![], target: 3 };
    let state = StaleOutput(Local { parameter: 0, output: 0, score: -6 });
    let mut rng = Pcg64Mcg::seed_from_u64(42);
    optimize(&input, state, Acceptance::HillClimbing, fixed_budget(), &mut rng);
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
