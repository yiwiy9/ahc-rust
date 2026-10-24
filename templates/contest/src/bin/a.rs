// AI-assisted template source:
// https://github.com/yiwiy9/ahc-rust/tree/main/templates
// AHC_SINGLE_FILE_TEMPLATE_V1
// 動く例: intro-heuristics A。新しい問題では「変更1〜6」を置き換える。
// 一つの解法はこの一枚で完結する。探索関数も自由に編集してよい。
include!("../base.rs");

fn main() {
    let input = read_input();
    // 【変更6: 探索】まず貪欲。複製後にbeam等のサンプルを追加し、ここを変更する。
    let state = solve_greedy(&input, State::new(&input));
    let output = state.into_output();
    #[cfg(debug_assertions)]
    validate_output(&input, &output).expect("invalid output");
    print_answer(&output);
}
