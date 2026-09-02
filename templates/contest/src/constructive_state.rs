use crate::problem::{Input, Output};

/// 構築中の解に加え、次の1手の評価に必要な問題固有データを持つ。
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
    type Action = Action;

    fn is_complete(&self) -> bool {
        true
    }

    fn legal_actions(&self) -> Vec<Self::Action> {
        Vec::new()
    }

    fn advance(&mut self, _action: &Self::Action) {
        // outputと評価用キャッシュを、必ず同じ1手で更新する。
    }

    fn evaluated_value(&self) -> i64 {
        // 探索エンジンは「大きいほどよい」値として比較する。
        0
    }

    fn debug_validate(&self) {
        // 小さい入力やdebug buildで、キャッシュを素直な再計算結果と照合する。
    }
}
