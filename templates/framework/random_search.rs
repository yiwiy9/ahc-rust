use rand::Rng;

use super::constructive::ConstructiveState;

/// 合法手をランダムに選び、制限回数だけ初期解を作って最良解を返す。
pub fn solve_random_search<S, R>(initial: &S, trials: usize, rng: &mut R) -> S
where
    S: ConstructiveState,
    R: Rng + ?Sized,
{
    assert!(trials > 0);
    let mut best = None;

    for _ in 0..trials {
        let mut state = initial.clone();
        while !state.is_complete() {
            let actions = state.legal_actions();
            assert!(
                !actions.is_empty(),
                "a non-complete state must have a legal action"
            );
            let action = &actions[rng.random_range(0..actions.len())];
            state.advance(action);
        }
        if best
            .as_ref()
            .is_none_or(|best: &S| state.evaluated_value() > best.evaluated_value())
        {
            best = Some(state);
        }
    }

    best.unwrap()
}
