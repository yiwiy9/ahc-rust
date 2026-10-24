use super::engine::LocalSearchState;
use crate::problem::{calculate_value, validate_output, Input, Output, TYPES};
use rand::Rng;

// 【局所探索の変更箇所】完成したscheduleを直接変更する例。
// 最初は全体再計算で正しさを確認し、必要になってから差分評価へ置き換える。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    output: Output,
    value: i64,
}

// undoに必要な変更前の値も保持する。
#[derive(Debug, Clone)]
pub struct Move {
    day: usize,
    before: usize,
    after: usize,
}

impl State {
    pub fn new(input: &Input, output: Output) -> Self {
        let value = calculate_value(input, &output);
        Self { output, value }
    }
    pub fn into_output(self) -> Output {
        self.output
    }
}

impl LocalSearchState for State {
    type Input = Input;
    type Move = Move;
    fn evaluated_value(&self, _input: &Input) -> i64 {
        self.value
    }

    fn propose_move<R: Rng + ?Sized>(&self, _input: &Input, rng: &mut R) -> Option<Move> {
        // 【変更: 近傍生成】scheduleの1日変更を、自分の解に対する合法な変更へ置き換える。
        if self.output.schedule.is_empty() {
            return None;
        }
        let day = rng.random_range(0..self.output.schedule.len());
        let before = self.output.schedule[day];
        let after = (before + rng.random_range(1..TYPES)) % TYPES;
        Some(Move { day, before, after })
    }

    fn apply_move(&mut self, input: &Input, movement: &Move) {
        // 【変更: 適用と評価】出力と評価を同時に更新する。最初は全体再計算でよい。
        debug_assert_eq!(self.output.schedule[movement.day], movement.before);
        self.output.schedule[movement.day] = movement.after;
        self.value = calculate_value(input, &self.output);
    }

    fn undo_move(&mut self, input: &Input, movement: &Move) {
        // 【変更: 取り消し】Moveに保持した変更前の値で、出力も評価も元に戻す。
        self.output.schedule[movement.day] = movement.before;
        self.value = calculate_value(input, &self.output);
    }

    fn debug_validate(&self, input: &Input) {
        // 【変更: 検算】自分の出力制約と、キャッシュの全体再計算を確認する。
        validate_output(input, &self.output).expect("invalid local-search output");
        debug_assert_eq!(self.value, calculate_value(input, &self.output));
    }
}
