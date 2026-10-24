// AI-assisted template source:
// https://github.com/yiwiy9/ahc-rust/tree/main/templates
// intro-heuristicsで動く完成例。自分の解を育てる場合はsamples/local_search_rebuild.rsの追加部分だけを使う。
include!("../contest/src/base.rs");
include!("../snippets/local_search_rebuild.rs");

fn main() {
    let started_at = std::time::Instant::now();
    let input = read_input();
    let state = solve_greedy(&input, State::new(&input));
    let output = state.into_output();
    let output = local_search_rebuild::improve(&input, output, started_at);
    #[cfg(debug_assertions)]
    validate_output(&input, &output).expect("invalid output");
    print_answer(&output);
}
