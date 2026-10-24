use proconio::input;

// 【変更1: 入出力】動く例としてintro-heuristics Aの「毎日のコンテスト選択」を使う。
// 新しい問題では、この定数・Input・Output・読み書き・得点計算を置き換える。
pub const TYPES: usize = 26;

// 入力はmainで一つだけ持ち、探索へ&Inputで渡す。Stateにコピーしない。
#[derive(Debug)]
pub struct Input {
    pub days: usize,
    pub decay: Vec<i64>,
    pub satisfaction: Vec<Vec<i64>>,
}

// 文字列化はprint_answerだけで行う。内部では問題に合う型で解を持つ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    pub schedule: Vec<usize>, // 各日に開催する種類。内部では0-indexed。
}

pub fn read_input() -> Input {
    input! {
        days: usize,
        decay: [i64; TYPES],
        satisfaction: [[i64; TYPES]; days],
    }
    Input {
        days,
        decay,
        satisfaction,
    }
}

pub fn print_answer(output: &Output) {
    for &contest in &output.schedule {
        println!("{}", contest + 1); // 提出形式は1-indexed。
    }
}

pub fn validate_output(input: &Input, output: &Output) -> Result<(), String> {
    if output.schedule.len() != input.days {
        return Err(format!(
            "expected {} days, got {}",
            input.days,
            output.schedule.len()
        ));
    }
    if output.schedule.iter().any(|&contest| contest >= TYPES) {
        return Err("contest type is out of range".to_string());
    }
    Ok(())
}

// 探索用の生の満足度。途中までのscheduleも計算できる。
// 公式得点は0で丸められるので、探索では丸める前の値を比較する。
pub fn calculate_value(input: &Input, output: &Output) -> i64 {
    let mut last_held = [0; TYPES];
    let mut value = 0;
    for (index, &contest) in output.schedule.iter().enumerate() {
        let day = index + 1;
        value += input.satisfaction[index][contest];
        last_held[contest] = day;
        for (kind, &last) in last_held.iter().enumerate() {
            value -= input.decay[kind] * (day - last) as i64;
        }
    }
    value
}

// 公式toolsと同じ得点。提出用の完成解を検算するときに使う。
pub fn calculate_score(input: &Input, output: &Output) -> i64 {
    (1_000_000 + calculate_value(input, output)).max(0)
}
