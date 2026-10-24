// このファイル全体を複製先の末尾へ貼る。state内のMoveが問題固有の変更箇所。
// mainの冒頭に let started_at = std::time::Instant::now(); を追加。
// 貪欲で作ったoutputの後、print_answerの前に次を追加:
// let output = local_search_rebuild::improve(&input, output, started_at);
// この例はintro用の近傍。別の問題ではstateの表現・近傍・評価を変更する。
#[allow(dead_code)]
mod local_search_rebuild {
    pub mod engine {
        include!("../framework/local_search.rs");
    }
    mod state {
        include!("../states/local_search_rebuild.rs");
    }

    pub fn improve(
        input: &crate::problem::Input,
        initial_output: crate::problem::Output,
        started_at: std::time::Instant,
    ) -> crate::problem::Output {
        use rand::SeedableRng;
        let initial = state::State::new(input, initial_output);
        // 【時間】入力・初期解生成も含む。実行環境に合わせて余裕を持たせる。
        let budget =
            engine::SearchBudget::from_env(started_at, std::time::Duration::from_millis(1_800));
        let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(42);
        // 【採否】山登りは engine::Acceptance::HillClimbing に変更する。
        // 温度は1近傍の改悪幅に合わせる。下記はintro用の例。
        let acceptance = engine::Acceptance::SimulatedAnnealing {
            start_temperature: 2_000.0,
            end_temperature: 10.0,
        };
        engine::optimize(input, initial, acceptance, budget, &mut rng).into_output()
    }
}
