use std::time::{Duration, Instant};

mod problem {
    include!("../problem.rs");
}
mod framework {
    pub mod constructive {
        include!("../framework/constructive.rs");
    }
    pub mod local_search {
        include!("../framework/local_search.rs");
    }
}
mod constructive_state {
    include!("../constructive_state.rs");
}
mod local_search_state {
    include!("../local_search_state_rebuild.rs");
}

use constructive_state::State as ConstructiveState;
use framework::constructive::solve_greedy;
use framework::local_search::{optimize, Acceptance, SearchBudget};
use local_search_state::State;
use problem::{print_answer, read_input, validate_output};
use rand::SeedableRng;
use rand_pcg::Pcg64Mcg;

const TIME_LIMIT: Duration = Duration::from_millis(1_900);

fn main() {
    let started_at = Instant::now();
    let input = read_input();
    let initial_output = solve_greedy(ConstructiveState::new(&input)).into_output();
    let initial = State::new(&input, initial_output);
    let budget = SearchBudget::from_env(started_at, TIME_LIMIT);
    let mut rng = Pcg64Mcg::seed_from_u64(42);
    let acceptance = Acceptance::SimulatedAnnealing {
        start_temperature: 1_000.0,
        end_temperature: 10.0,
    };
    let state = optimize(initial, acceptance, budget, &mut rng);
    let output = state.into_output();

    #[cfg(debug_assertions)]
    validate_output(&input, &output).expect("invalid output");
    print_answer(&output);
}
