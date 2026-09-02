use rand::Rng;

use crate::framework::local_search::LocalSearchState;
use crate::problem::{calculate_score, Input, Output};

/// 探索が直接変更する、少量の意思決定だけを持つ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameters {
    // 仮の保持方法。問題を読んだら、近傍が直接変更する少量の値へ置き換える。
    draft: Output,
}

/// ParametersからOutputと評価値を毎回再構築し、派生データの更新漏れを防ぐ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    input: Input,
    parameters: Parameters,
    output: Output,
    evaluated_value: i64,
}

/// 変更前後のParametersを復元できる情報を持たせる。
#[derive(Debug, Clone)]
pub enum Move {
    Noop,
}

impl State {
    pub fn new(input: &Input, initial_output: Output) -> Self {
        let mut state = Self {
            input: input.clone(),
            parameters: Parameters {
                draft: initial_output,
            },
            output: Output { lines: Vec::new() },
            evaluated_value: 0,
        };
        state.rebuild();
        state
    }

    /// Parametersから、出力全体とスコアを必ず同時に作り直す唯一の経路。
    fn rebuild(&mut self) {
        self.output = self.parameters.draft.clone();
        self.evaluated_value = calculate_score(&self.input, &self.output);
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
        None
    }

    fn apply_move(&mut self, movement: &Self::Move) {
        match movement {
            Move::Noop => {}
        }
        self.rebuild();
    }

    fn undo_move(&mut self, movement: &Self::Move) {
        match movement {
            Move::Noop => {}
        }
        self.rebuild();
    }

    fn debug_validate(&self) {
        let mut rebuilt = self.clone();
        rebuilt.rebuild();
        debug_assert_eq!(self.output, rebuilt.output, "output is stale");
        debug_assert_eq!(
            self.evaluated_value, rebuilt.evaluated_value,
            "evaluation is stale"
        );
    }
}
