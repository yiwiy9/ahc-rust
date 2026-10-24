use super::engine::LocalSearchState;
// 【変更: 型と評価関数】自作のInput・Output・Action等に合わせ、不要なimportは削除する。
use crate::constructive_state::{Action, State as Construction};
use crate::framework::constructive::ConstructiveState;
use crate::problem::{calculate_value, validate_output, Input, Output, TYPES};
use rand::Rng;

// 【再構築型の変更箇所】各日に「評価上位何番目を選ぶか」をパラメータにする。
// 前の日を変えると後日の順位も変わるので、変更後は構築処理をやり直す。
// 新しい問題では、Parametersとconstruct_outputを自分の表現へ置き換える。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameters {
    ranks: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    parameters: Parameters,
    output: Output,
    value: i64,
}

#[derive(Debug, Clone)]
pub struct Move {
    day: usize,
    before: usize,
    after: usize,
}

fn ranked_actions(input: &Input, state: &Construction) -> Vec<Action> {
    // 【変更: 候補の順位付け】intro固有のAction.contestを使う。自作Actionへ変更する。
    // 別のconstruct_outputへ置き換えてこの関数が不要になったら、関数ごと削除する。
    let mut actions = state.legal_actions(input);
    actions.sort_by_cached_key(|action| {
        (
            std::cmp::Reverse(state.evaluated_value_after(input, action)),
            action.contest,
        )
    });
    actions
}

fn construct_output(input: &Input, parameters: &Parameters) -> Output {
    // 【変更: 再構築】Parametersから合法な完成解を作る、自分の構築処理へ置き換える。
    let mut state = Construction::new(input);
    for &rank in &parameters.ranks {
        let actions = ranked_actions(input, &state);
        state.advance(input, &actions[rank]);
    }
    state.into_output()
}

impl State {
    pub fn new(input: &Input, initial: Output) -> Self {
        // 初期解を同じ構築手順で再現する順位列へ変換する。
        let mut construction = Construction::new(input);
        let mut ranks = Vec::with_capacity(input.days);
        for &contest in &initial.schedule {
            let actions = ranked_actions(input, &construction);
            let rank = actions
                .iter()
                .position(|action| action.contest == contest)
                .unwrap();
            ranks.push(rank);
            construction.advance(input, &actions[rank]);
        }
        let mut state = Self {
            parameters: Parameters { ranks },
            output: initial,
            value: 0,
        };
        state.rebuild(input);
        state
    }

    // Outputと評価は必ず同時に作り直す。派生データだけの更新を避ける。
    fn rebuild(&mut self, input: &Input) {
        self.output = construct_output(input, &self.parameters);
        self.value = calculate_value(input, &self.output);
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
        // 【変更: 近傍生成】自分のParametersに対する変更前後の値を作る。
        if self.parameters.ranks.is_empty() {
            return None;
        }
        let day = rng.random_range(0..self.parameters.ranks.len());
        let before = self.parameters.ranks[day];
        let after = (before + rng.random_range(1..TYPES)) % TYPES;
        Some(Move { day, before, after })
    }
    fn apply_move(&mut self, input: &Input, movement: &Move) {
        // 【変更: 適用】Parametersを変更したら、必ず出力と評価も作り直す。
        self.parameters.ranks[movement.day] = movement.after;
        self.rebuild(input);
    }
    fn undo_move(&mut self, input: &Input, movement: &Move) {
        // 【変更: 取り消し】Parametersを戻してから、同じ経路で再構築する。
        self.parameters.ranks[movement.day] = movement.before;
        self.rebuild(input);
    }
    fn debug_validate(&self, input: &Input) {
        validate_output(input, &self.output).expect("invalid rebuilt output");
        let mut rebuilt = self.clone();
        rebuilt.rebuild(input);
        debug_assert_eq!(
            *self, rebuilt,
            "parameters, output and evaluation must agree"
        );
    }
}
