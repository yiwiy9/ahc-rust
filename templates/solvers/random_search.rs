// Pre-contest AI-generated template (published before the contest):
// https://github.com/yiwiy9/ahc-rust/tree/main/templates
mod problem {
    include!("../problem.rs");
}
mod framework {
    pub mod constructive {
        include!("../framework/constructive.rs");
    }
    pub mod random_search {
        include!("../framework/random_search.rs");
    }
}
mod constructive_state {
    include!("../constructive_state.rs");
}

use constructive_state::State;
use framework::random_search::solve_random_search;
use problem::{print_answer, read_input, validate_output};
use rand::SeedableRng;
use rand_pcg::Pcg64Mcg;

const TRIALS: usize = 100;

fn main() {
    let input = read_input();
    let initial = State::new(&input);
    let mut rng = Pcg64Mcg::seed_from_u64(42);
    let state = solve_random_search(&initial, TRIALS, &mut rng);
    let output = state.into_output();

    #[cfg(debug_assertions)]
    validate_output(&input, &output).expect("invalid output");
    print_answer(&output);
}
