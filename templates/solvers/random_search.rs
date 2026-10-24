// AI-assisted template source:
// https://github.com/yiwiy9/ahc-rust/tree/main/templates
// intro-heuristicsで動く完成例。自分の解を育てる場合はsamples/random_search.rsの追加部分だけを使う。
include!("../contest/src/base.rs");
include!("../snippets/random_search.rs");

fn main() {
    let input = read_input();
    let state = random_search::solve_seeded(&input, &State::new(&input), 20, 3, 42);
    let output = state.into_output();
    #[cfg(debug_assertions)]
    validate_output(&input, &output).expect("invalid output");
    print_answer(&output);
}
