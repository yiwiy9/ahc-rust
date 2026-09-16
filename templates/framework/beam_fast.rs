use super::constructive::ConstructiveState;

#[derive(Debug)]
struct History<A> {
    parent: Option<usize>,
    action: Option<A>,
}

#[derive(Debug)]
struct Candidate<S> {
    state: S,
    history_id: usize,
}

/// 親ポインタで手順を記録し、最良手順を初期状態から再生する版。
/// Stateのcloneは残る。長い出力をStateに保持したままでは複製コストは減らない。
/// 通常版より速いとは限らないので、Stateを軽量化した上で実測して選ぶ。
pub fn solve_beam_fast<S: ConstructiveState>(input: &S::Input, initial: S, beam_width: usize) -> S {
    assert!(beam_width > 0);
    let mut history = vec![History {
        parent: None,
        action: None,
    }];
    let mut beam = vec![Candidate {
        state: initial.clone(),
        history_id: 0,
    }];

    while !beam[0].state.is_complete(input) {
        let mut candidates = Vec::with_capacity(beam.len() * 8);
        for candidate in &beam {
            for action in candidate.state.legal_actions(input) {
                let mut next = candidate.state.clone();
                next.advance(input, &action);
                #[cfg(debug_assertions)]
                next.debug_validate(input);
                let history_id = history.len();
                history.push(History {
                    parent: Some(candidate.history_id),
                    action: Some(action),
                });
                candidates.push(Candidate {
                    state: next,
                    history_id,
                });
            }
        }
        assert!(!candidates.is_empty(), "beam search has no candidate");
        candidates.sort_unstable_by_key(|candidate| {
            std::cmp::Reverse(candidate.state.evaluated_value(input))
        });
        candidates.truncate(beam_width);
        beam = candidates;
    }

    let best = beam
        .into_iter()
        .max_by_key(|candidate| candidate.state.evaluated_value(input))
        .unwrap();
    let mut reversed = Vec::new();
    let mut cursor = best.history_id;
    while let Some(parent) = history[cursor].parent {
        reversed.push(history[cursor].action.as_ref().unwrap().clone());
        cursor = parent;
    }

    let mut result = initial;
    for action in reversed.into_iter().rev() {
        result.advance(input, &action);
    }
    result
}
