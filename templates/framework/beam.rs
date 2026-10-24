use crate::framework::constructive::{
    evaluate_action, keep_best_actions, ConstructiveState, ScoredAction,
};

/// 親・手・評価値で上位を選び、採用候補だけStateをcloneするビームサーチ。
/// 軽量なevaluated_value_afterを実装すると、評価時の履歴コピーも省ける。
pub fn solve_beam<S: ConstructiveState>(input: &S::Input, initial: S, beam_width: usize) -> S {
    assert!(beam_width > 0);
    let mut beam = vec![initial];

    while !beam[0].is_complete(input) {
        let mut candidates = Vec::with_capacity(beam.len() * 8);
        for (parent, state) in beam.iter().enumerate() {
            for action in state.legal_actions(input) {
                let value = evaluate_action(input, state, &action);
                candidates.push(ScoredAction {
                    parent,
                    action,
                    value,
                    order: candidates.len(),
                });
            }
        }
        assert!(!candidates.is_empty(), "beam search has no candidate");
        keep_best_actions(&mut candidates, beam_width);
        let mut next_beam = Vec::with_capacity(candidates.len());
        for candidate in candidates {
            let mut next = beam[candidate.parent].clone();
            next.advance(input, &candidate.action);
            #[cfg(debug_assertions)]
            next.debug_validate(input);
            next_beam.push(next);
        }
        beam = next_beam;
    }

    beam.into_iter().next().unwrap()
}
