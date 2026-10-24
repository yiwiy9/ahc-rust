// このファイル全体を、複製したa.rsの末尾へ貼る。
// mainの探索1行を次へ変更する:
// let state = beam::solve_beam(&input, State::new(&input), 100);
// 同じConstructiveStateを使うので、入出力やStateをコピーし直す必要はない。
mod beam {
    include!("../framework/beam.rs");
}
