use std::io::{self, Read};

/// 問題文から読み取った、変更されない入力。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Input {
    pub raw: String,
}

/// AtCoderへ出力する解そのもの。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    pub lines: Vec<String>,
}

pub fn read_input() -> Input {
    let mut raw = String::new();
    io::stdin().read_to_string(&mut raw).unwrap();
    Input { raw }
}

pub fn print_answer(output: &Output) {
    for line in &output.lines {
        println!("{line}");
    }
}

/// 出力形式や制約違反を早めに検出する。問題ごとの条件を追加する。
pub fn validate_output(_input: &Input, _output: &Output) -> Result<(), String> {
    Ok(())
}

/// デバッグ時の検算用。公式得点と同じ定義を、まず素直に実装する。
pub fn calculate_score(_input: &Input, _output: &Output) -> i64 {
    0
}
