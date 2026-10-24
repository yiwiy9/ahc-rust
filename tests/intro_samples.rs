//! Check real sample states, not only the generic engine contracts.
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;

mod problem {
    include!("fragments/problem.rs");
}
mod framework {
    pub mod constructive {
        include!("framework/constructive.rs");
    }
    pub mod random_search {
        include!("framework/random_search.rs");
    }
}
mod constructive_state {
    include!("fragments/constructive_state.rs");
}
mod direct {
    pub mod engine {
        include!("framework/local_search.rs");
    }
    pub mod state {
        include!("fragments/local_search_direct.rs");
    }
}
mod rebuild {
    pub mod engine {
        include!("framework/local_search.rs");
    }
    pub mod state {
        include!("fragments/local_search_rebuild.rs");
    }
}
use framework::constructive::{solve_greedy, ConstructiveState};

fn input() -> problem::Input {
    let mut satisfaction = vec![vec![0; 26]; 3];
    for day in 0..3 {
        satisfaction[day][day] = 100;
    }
    problem::Input {
        days: 3,
        decay: vec![1; 26],
        satisfaction,
    }
}

#[test]
fn greedy_score_matches_hand_calculation_and_all_previews() {
    let input = input();
    let mut state = constructive_state::State::new(&input);
    while !state.is_complete(&input) {
        for action in state.legal_actions(&input) {
            let mut next = state.clone();
            next.advance(&input, &action);
            assert_eq!(
                state.evaluated_value_after(&input, &action),
                next.evaluated_value(&input)
            );
            next.debug_validate(&input);
        }
        let day = state.clone().into_output().schedule.len();
        state.advance(&input, &constructive_state::Action { contest: day });
    }
    let greedy = solve_greedy(&input, constructive_state::State::new(&input)).into_output();
    assert_eq!(greedy.schedule, vec![0, 1, 2]);
    assert_eq!(problem::calculate_value(&input, &greedy), 154);
    assert_eq!(problem::calculate_score(&input, &greedy), 1_000_154);
    assert!(problem::validate_output(&input, &greedy).is_ok());
    assert!(problem::validate_output(&input, &problem::Output { schedule: vec![0] }).is_err());
    assert!(problem::validate_output(
        &input,
        &problem::Output {
            schedule: vec![26; 3]
        }
    )
    .is_err());
}

#[test]
fn randomized_greedy_is_reproducible_and_keeps_the_greedy_baseline() {
    let input = input();
    let initial = constructive_state::State::new(&input);
    let greedy = solve_greedy(&input, initial.clone()).into_output();
    let run = || {
        framework::random_search::solve_randomized_greedy(
            &input,
            &initial,
            20,
            3,
            &mut Pcg64Mcg::seed_from_u64(42),
        )
        .into_output()
    };
    let output = run();
    assert_eq!(run(), output);
    assert!(problem::calculate_value(&input, &output) >= problem::calculate_value(&input, &greedy));
}

#[test]
fn direct_changes_are_legal_and_undo_restores_the_entire_state() {
    use direct::engine::LocalSearchState;
    let input = input();
    let output = solve_greedy(&input, constructive_state::State::new(&input)).into_output();
    let mut state = direct::state::State::new(&input, output);
    let mut rng = Pcg64Mcg::seed_from_u64(42);
    for _ in 0..50 {
        let before = state.clone();
        let movement = state.propose_move(&input, &mut rng).unwrap();
        state.apply_move(&input, &movement);
        assert_ne!(state, before);
        state.debug_validate(&input);
        state.undo_move(&input, &movement);
        assert_eq!(state, before);
    }
}

#[test]
fn rebuild_reproduces_initial_solutions_and_undo_restores_parameters_and_output() {
    use rebuild::engine::LocalSearchState;
    let input = input();
    let mut rng = Pcg64Mcg::seed_from_u64(13);
    for _ in 0..10 {
        let output = problem::Output {
            schedule: (0..input.days).map(|_| rng.random_range(0..26)).collect(),
        };
        let mut state = rebuild::state::State::new(&input, output.clone());
        assert_eq!(state.clone().into_output(), output);
        for _ in 0..10 {
            let before = state.clone();
            let movement = state.propose_move(&input, &mut rng).unwrap();
            state.apply_move(&input, &movement);
            assert_ne!(state, before);
            state.debug_validate(&input);
            state.undo_move(&input, &movement);
            assert_eq!(state, before);
        }
    }
}
