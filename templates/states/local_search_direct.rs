use rand::Rng;

use crate::framework::local_search::LocalSearchState;
use crate::problem::{calculate_score, Input, Output};

/// Output自体が探索状態で、Moveの適用と取り消しを直接行う。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    input: Input,
    output: Output,
    evaluated_value: i64,
}

/// 取り消しに必要な変更前の値もMoveへ持たせる。
#[derive(Debug, Clone)]
pub enum Move {
    Noop,
}

impl State {
    pub fn new(input: &Input, output: Output) -> Self {
        let evaluated_value = calculate_score(input, &output);
        Self {
            input: input.clone(),
            output,
            evaluated_value,
        }
    }

    pub fn into_output(self) -> Output {
        self.output
    }
}

impl LocalSearchState for State {
    type Move = Move;

    fn evaluated_value(&self) -> i64 {
        self.evaluated_value
    }

    fn propose_move<R: Rng + ?Sized>(&self, _rng: &mut R) -> Option<Self::Move> {
        // 例: 1か所変更、2か所交換、区間反転。まず1種類だけ実装する。
        None
    }

    fn apply_move(&mut self, movement: &Self::Move) {
        match movement {
            Move::Noop => {}
        }
        self.evaluated_value = calculate_score(&self.input, &self.output);
    }

    fn undo_move(&mut self, movement: &Self::Move) {
        match movement {
            Move::Noop => {}
        }
        self.evaluated_value = calculate_score(&self.input, &self.output);
    }

    fn debug_validate(&self) {
        debug_assert_eq!(
            self.evaluated_value,
            calculate_score(&self.input, &self.output),
            "cached evaluation diverged from the output"
        );
    }
}
