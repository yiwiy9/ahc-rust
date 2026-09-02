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

/// 各候補が手順Vecを持たず、共有された親ポインタで最良手順だけを復元する。
/// State自体のcloneは残るため、Stateには入力や長い出力を持たせず軽量に保つ。
pub fn solve_beam_fast<S: ConstructiveState>(initial: S, beam_width: usize) -> S {
    assert!(beam_width > 0);
    let mut history = vec![History {
        parent: None,
        action: None,
    }];
    let mut beam = vec![Candidate {
        state: initial.clone(),
        history_id: 0,
    }];

    while !beam[0].state.is_complete() {
        let mut candidates = Vec::with_capacity(beam.len() * 8);
        for candidate in &beam {
            for action in candidate.state.legal_actions() {
                let mut next = candidate.state.clone();
                next.advance(&action);
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
        candidates
            .sort_unstable_by_key(|candidate| std::cmp::Reverse(candidate.state.evaluated_value()));
        candidates.truncate(beam_width);
        beam = candidates;
    }

    let best = beam
        .into_iter()
        .max_by_key(|candidate| candidate.state.evaluated_value())
        .unwrap();
    let mut reversed = Vec::new();
    let mut cursor = best.history_id;
    while let Some(parent) = history[cursor].parent {
        reversed.push(history[cursor].action.as_ref().unwrap().clone());
        cursor = parent;
    }

    let mut result = initial;
    for action in reversed.into_iter().rev() {
        result.advance(&action);
    }
    result
}
