use rand::Rng;

use crate::framework::constructive::{
    evaluate_action, keep_best_actions, ConstructiveState, ScoredAction,
};

/// 各手で評価上位top_k個から乱択する構築をtrials回行う。
/// 初回は貪欲(top_k=1)にし、乱択が外れてもその解を残す。
pub fn solve_randomized_greedy<S, R>(
    input: &S::Input,
    initial: &S,
    trials: usize,
    top_k: usize,
    rng: &mut R,
) -> S
where
    S: ConstructiveState,
    R: Rng + ?Sized,
{
    assert!(trials > 0 && top_k > 0);
    let mut best: Option<S> = None;
    for trial in 0..trials {
        let mut state = initial.clone();
        while !state.is_complete(input) {
            let mut candidates = Vec::new();
            for action in state.legal_actions(input) {
                candidates.push(ScoredAction {
                    parent: 0,
                    value: evaluate_action(input, &state, &action),
                    action,
                    order: candidates.len(),
                });
            }
            assert!(
                !candidates.is_empty(),
                "a non-complete state must have a legal action"
            );
            keep_best_actions(&mut candidates, if trial == 0 { 1 } else { top_k });
            let index = rng.random_range(0..candidates.len());
            state.advance(input, &candidates[index].action);
            #[cfg(debug_assertions)]
            state.debug_validate(input);
        }
        if best
            .as_ref()
            .is_none_or(|best| state.evaluated_value(input) > best.evaluated_value(input))
        {
            best = Some(state);
        }
    }
    best.unwrap()
}

/// 合法手をランダムに選び、制限回数だけ初期解を作って最良解を返す。
pub fn solve_random_search<S, R>(input: &S::Input, initial: &S, trials: usize, rng: &mut R) -> S
where
    S: ConstructiveState,
    R: Rng + ?Sized,
{
    assert!(trials > 0);
    let mut best = None;

    for _ in 0..trials {
        let mut state = initial.clone();
        while !state.is_complete(input) {
            let actions = state.legal_actions(input);
            assert!(
                !actions.is_empty(),
                "a non-complete state must have a legal action"
            );
            let action = &actions[rng.random_range(0..actions.len())];
            state.advance(input, action);
            #[cfg(debug_assertions)]
            state.debug_validate(input);
        }
        if best
            .as_ref()
            .is_none_or(|best: &S| state.evaluated_value(input) > best.evaluated_value(input))
        {
            best = Some(state);
        }
    }

    best.unwrap()
}
