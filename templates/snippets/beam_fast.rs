// このファイル全体を複製先の末尾へ貼る。
// main: let state = beam_fast::solve_beam_fast(&input, State::new(&input), 100);
// 採用した手を履歴で持ち、最後に初期状態から再生する例。
// State自体に長い履歴があれば、採用状態のcloneは残る。
mod beam_fast {
    include!("../framework/beam_fast.rs");
}
