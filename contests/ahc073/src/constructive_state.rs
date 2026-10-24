use crate::problem::{Input, Output};

/// 構築中の解と、手を進めるたびに変わるキャッシュを持つ。
/// 変わらない入力は各メソッドのinputから参照し、Stateへコピーしない。
#[derive(Debug, Clone)]
pub struct State {
    output: Output,
}

/// 構築中の解に加える1手。問題に合わせてフィールドを置き換える。
#[derive(Debug, Clone)]
pub struct Action;

impl State {
    pub fn new(_input: &Input) -> Self {
        Self {
            output: Output { lines: Vec::new() },
        }
    }

    pub fn into_output(self) -> Output {
        self.output
    }
}

impl crate::framework::constructive::ConstructiveState for State {
    type Input = Input;
    type Action = Action;

    fn is_complete(&self, _input: &Input) -> bool {
        true
    }

    fn legal_actions(&self, _input: &Input) -> Vec<Self::Action> {
        Vec::new()
    }

    fn advance(&mut self, _input: &Input, _action: &Self::Action) {
        // outputと評価用キャッシュを、必ず同じ1手で更新する。
    }

    fn evaluated_value(&self, _input: &Input) -> i64 {
        // 探索エンジンは「大きいほどよい」値として比較する。
        0
    }

    fn debug_validate(&self, _input: &Input) {
        // 小さい入力やdebug buildで、キャッシュを素直な再計算結果と照合する。
    }
}
