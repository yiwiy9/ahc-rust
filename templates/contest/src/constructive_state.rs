use crate::framework::constructive::ConstructiveState;
use crate::problem::{calculate_value, Input, Output, TYPES};

// 【変更2: 構築状態】途中の解と、次の1手に必要なキャッシュだけを持つ。
#[derive(Debug, Clone)]
pub struct State {
    output: Output,
    last_held: [usize; TYPES],
    value: i64,
}

// この例の1手は「次の日に開催する種類」。経路なら移動先などへ置き換える。
#[derive(Debug, Clone)]
pub struct Action {
    pub contest: usize,
}

impl State {
    pub fn new(_input: &Input) -> Self {
        Self {
            output: Output {
                schedule: Vec::new(),
            },
            last_held: [0; TYPES],
            value: 0,
        }
    }

    pub fn into_output(self) -> Output {
        self.output
    }

    // 【変更5: 評価】次の評価値だけを計算し、長い出力履歴をコピーしない。
    // advanceと軽量評価で共用する。最小化なら-costなど、大きいほどよい値にする。
    // traitのevaluated_value_afterを省略しても、状態更新はこの関数で独立して動く。
    fn value_after(&self, input: &Input, action: &Action) -> i64 {
        let index = self.output.schedule.len();
        let day = index + 1;
        let mut value = self.value + input.satisfaction[index][action.contest];
        for kind in 0..TYPES {
            let last = if kind == action.contest {
                day
            } else {
                self.last_held[kind]
            };
            value -= input.decay[kind] * (day - last) as i64;
        }
        value
    }
}

impl ConstructiveState for State {
    type Input = Input;
    type Action = Action;

    fn is_complete(&self, input: &Input) -> bool {
        self.output.schedule.len() == input.days
    }

    // 【変更3: 候補】ここでは26種類すべて。重い問題では候補数も設計する。
    fn legal_actions(&self, _input: &Input) -> Vec<Action> {
        (0..TYPES).map(|contest| Action { contest }).collect()
    }

    // 【変更4: 更新】評価・出力・キャッシュを同じ1手で更新する。
    fn advance(&mut self, input: &Input, action: &Action) {
        self.value = self.value_after(input, action);
        self.output.schedule.push(action.contest);
        self.last_held[action.contest] = self.output.schedule.len();
    }

    fn evaluated_value(&self, _input: &Input) -> i64 {
        self.value
    }

    // 軽量評価を使う場合の入口。評価式は上のvalue_afterにまとめる。
    // 省略時のtrait既定実装はclone→advance→evaluated_valueで検算できる。
    fn evaluated_value_after(&self, input: &Input, action: &Action) -> i64 {
        self.value_after(input, action)
    }

    // debug実行では、差分評価と素直な全体再計算の一致を確認する。
    fn debug_validate(&self, input: &Input) {
        debug_assert_eq!(self.value, calculate_value(input, &self.output));
        let mut last_held = [0; TYPES];
        for (index, &contest) in self.output.schedule.iter().enumerate() {
            last_held[contest] = index + 1;
        }
        debug_assert_eq!(self.last_held, last_held);
    }
}
