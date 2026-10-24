// AI-assisted template source:
// https://github.com/yiwiy9/ahc-rust/tree/main/templates
// intro-heuristicsで動く完成例。自分の解を育てる場合はsamples/beam_fast.rsの追加部分だけを使う。
include!("../contest/src/base.rs");
include!("../snippets/beam_fast.rs");

fn main() {
    let input = read_input();
    let state = beam_fast::solve_beam_fast(&input, State::new(&input), 100);
    let output = state.into_output();
    #[cfg(debug_assertions)]
    validate_output(&input, &output).expect("invalid output");
    print_answer(&output);
}
