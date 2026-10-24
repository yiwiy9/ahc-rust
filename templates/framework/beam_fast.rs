use crate::framework::constructive::{
    evaluate_action, keep_best_actions, ConstructiveState, ScoredAction,
};

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
/// 採用候補だけStateと履歴を作り、棄却した候補の履歴は保存しない。
/// evaluated_value_afterを実装しなければ、評価用のStateのcloneは残る。
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
        for (parent, candidate) in beam.iter().enumerate() {
            for action in candidate.state.legal_actions(input) {
                let value = evaluate_action(input, &candidate.state, &action);
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
            let parent = &beam[candidate.parent];
            let mut next = parent.state.clone();
            next.advance(input, &candidate.action);
            #[cfg(debug_assertions)]
            next.debug_validate(input);
            let history_id = history.len();
            history.push(History {
                parent: Some(parent.history_id),
                action: Some(candidate.action),
            });
            next_beam.push(Candidate {
                state: next,
                history_id,
            });
        }
        beam = next_beam;
    }

    let best = beam.into_iter().next().unwrap();
    let mut reversed = Vec::new();
    let mut cursor = best.history_id;
    while let Some(parent) = history[cursor].parent {
        reversed.push(history[cursor].action.as_ref().unwrap().clone());
        cursor = parent;
    }

    let mut result = initial;
    for action in reversed.into_iter().rev() {
        result.advance(input, &action);
        #[cfg(debug_assertions)]
        result.debug_validate(input);
    }
    result
}
