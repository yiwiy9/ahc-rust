/// 貪欲・乱択・ビームサーチが共通で使う、構築型問題の最小インターフェース。
pub trait ConstructiveState: Clone {
    /// 入力は探索全体で共有し、Stateのcloneでは複製しない。
    type Input;
    type Action: Clone;

    fn is_complete(&self, input: &Self::Input) -> bool;
    fn legal_actions(&self, input: &Self::Input) -> Vec<Self::Action>;
    fn advance(&mut self, input: &Self::Input, action: &Self::Action);
    fn evaluated_value(&self, input: &Self::Input) -> i64;

    /// advance後の評価値。履歴をコピーせず計算できる場合は上書きする。
    /// 未実装なら、従来どおりcloneして評価するので既存のStateも使える。
    /// 既定実装を使う場合、advanceからこのメソッドを呼ばないこと（再帰になる）。
    fn evaluated_value_after(&self, input: &Self::Input, action: &Self::Action) -> i64 {
        let mut next = self.clone();
        next.advance(input, action);
        next.evaluated_value(input)
    }

    fn debug_validate(&self, _input: &Self::Input) {}
}

/// 軽量評価の更新漏れも、採用・棄却にかかわらずdebugで検出する。
pub(crate) fn evaluate_action<S: ConstructiveState>(
    input: &S::Input,
    state: &S,
    action: &S::Action,
) -> i64 {
    let value = state.evaluated_value_after(input, action);
    #[cfg(debug_assertions)]
    {
        let mut next = state.clone();
        next.advance(input, action);
        next.debug_validate(input);
        assert_eq!(
            value,
            next.evaluated_value(input),
            "evaluated_value_after must match advance + evaluated_value"
        );
    }
    value
}

/// Stateは保持せず、親・手・評価値だけで候補を比較する。
pub(crate) struct ScoredAction<A> {
    pub parent: usize,
    pub action: A,
    pub value: i64,
    pub order: usize,
}

pub(crate) fn keep_best_actions<A>(candidates: &mut Vec<ScoredAction<A>>, width: usize) {
    let compare = |left: &ScoredAction<A>, right: &ScoredAction<A>| {
        right
            .value
            .cmp(&left.value)
            .then(left.order.cmp(&right.order))
    };
    if candidates.len() > width {
        candidates.select_nth_unstable_by(width, compare);
        candidates.truncate(width);
    }
    // 同点では生成順を使い、候補の構造体サイズに依存する並びの変化を避ける。
    candidates.sort_unstable_by(compare);
}

/// 各手で、進めた後の評価値が最大になる合法手を選ぶ。
/// evaluated_value_afterを実装すれば、候補の評価でStateをコピーしない。
pub fn solve_greedy<S: ConstructiveState>(input: &S::Input, mut state: S) -> S {
    while !state.is_complete(input) {
        let mut best = None;
        for action in state.legal_actions(input) {
            let value = evaluate_action(input, &state, &action);
            if best
                .as_ref()
                .is_none_or(|(best_value, _)| value > *best_value)
            {
                best = Some((value, action));
            }
        }
        let (_, action) = best.expect("a non-complete state must have a legal action");
        state.advance(input, &action);
        #[cfg(debug_assertions)]
        state.debug_validate(input);
    }
    state
}
