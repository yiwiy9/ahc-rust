use super::constructive::ConstructiveState;

/// 各深さで評価値上位の状態だけを残す、読みやすさ優先のビームサーチ。
/// State全体をcloneするため、まずはこちらで方針を確認し、重ければbeam_fastへ進む。
pub fn solve_beam<S: ConstructiveState>(input: &S::Input, initial: S, beam_width: usize) -> S {
    assert!(beam_width > 0);
    let mut beam = vec![initial];

    while !beam[0].is_complete(input) {
        let mut candidates = Vec::with_capacity(beam.len() * 8);
        for state in &beam {
            for action in state.legal_actions(input) {
                let mut next = state.clone();
                next.advance(input, &action);
                #[cfg(debug_assertions)]
                next.debug_validate(input);
                candidates.push(next);
            }
        }
        assert!(!candidates.is_empty(), "beam search has no candidate");
        candidates.sort_unstable_by_key(|state| std::cmp::Reverse(state.evaluated_value(input)));
        candidates.truncate(beam_width);
        beam = candidates;
    }

    beam.into_iter()
        .max_by_key(|state| state.evaluated_value(input))
        .unwrap()
}
