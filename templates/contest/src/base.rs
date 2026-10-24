// 以下の入出力・State・探索は、生成後のa.rs内で直接編集できる。
// build/runで生成元から上書きしない。
#[allow(dead_code)]
mod problem {
    include!("problem.rs");
}
#[allow(dead_code)]
mod framework {
    pub mod constructive {
        include!("framework/constructive.rs");
    }
}
mod constructive_state {
    include!("constructive_state.rs");
}

use constructive_state::State;
#[allow(unused_imports)] // ビーム等の完成例でも同じbaseを使う。
use framework::constructive::solve_greedy;
#[cfg(debug_assertions)]
use problem::validate_output;
use problem::{print_answer, read_input};
