/// 貪欲・乱択・ビームサーチが共通で使う、構築型問題の最小インターフェース。
pub trait ConstructiveState: Clone {
    /// 入力は探索全体で共有し、Stateのcloneでは複製しない。
    type Input;
    type Action: Clone;

    fn is_complete(&self, input: &Self::Input) -> bool;
    fn legal_actions(&self, input: &Self::Input) -> Vec<Self::Action>;
    fn advance(&mut self, input: &Self::Input, action: &Self::Action);
    fn evaluated_value(&self, input: &Self::Input) -> i64;
    fn debug_validate(&self, _input: &Self::Input) {}
}

/// 各手で、進めた後の評価値が最大になる合法手を選ぶ。
/// 問題が変わってもこの制御は変えず、State側の4メソッドを実装する。
pub fn solve_greedy<S: ConstructiveState>(input: &S::Input, mut state: S) -> S {
    while !state.is_complete(input) {
        let mut best = None;
        for action in state.legal_actions(input) {
            let mut next = state.clone();
            next.advance(input, &action);
            #[cfg(debug_assertions)]
            next.debug_validate(input);
            let value = next.evaluated_value(input);
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
