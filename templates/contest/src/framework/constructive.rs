/// 貪欲・乱択・ビームサーチが共通で使う、構築型問題の最小インターフェース。
pub trait ConstructiveState: Clone {
    type Action: Clone;

    fn is_complete(&self) -> bool;
    fn legal_actions(&self) -> Vec<Self::Action>;
    fn advance(&mut self, action: &Self::Action);
    fn evaluated_value(&self) -> i64;
    fn debug_validate(&self) {}
}

/// 各手で、進めた後の評価値が最大になる合法手を選ぶ。
/// 問題が変わってもこの制御は変えず、State側の4メソッドを実装する。
pub fn solve_greedy<S: ConstructiveState>(mut state: S) -> S {
    while !state.is_complete() {
        let mut best = None;
        for action in state.legal_actions() {
            let mut next = state.clone();
            next.advance(&action);
            let value = next.evaluated_value();
            if best
                .as_ref()
                .is_none_or(|(best_value, _)| value > *best_value)
            {
                best = Some((value, action));
            }
        }
        let (_, action) = best.expect("a non-complete state must have a legal action");
        state.advance(&action);
        #[cfg(debug_assertions)]
        state.debug_validate();
    }
    state
}
