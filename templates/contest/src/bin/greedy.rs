// Pre-contest AI-generated template (published before the contest):
// https://github.com/yiwiy9/ahc-rust/tree/main/templates
mod problem {
    include!("../problem.rs");
}
mod framework {
    pub mod constructive {
        include!("../framework/constructive.rs");
    }
}
mod constructive_state {
    include!("../constructive_state.rs");
}

use constructive_state::State;
use framework::constructive::solve_greedy;
use problem::{print_answer, read_input, validate_output};

fn main() {
    let input = read_input();
    let state = solve_greedy(State::new(&input));
    let output = state.into_output();

    #[cfg(debug_assertions)]
    validate_output(&input, &output).expect("invalid output");
    print_answer(&output);
}
