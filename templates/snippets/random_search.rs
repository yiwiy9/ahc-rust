// このファイル全体を複製先の末尾へ貼る。
// mainの探索1行を次へ変更する（20回、各手で上位3候補から乱択）:
// let state = random_search::solve_seeded(&input, &State::new(&input), 20, 3, 42);
// 乱数seedを固定すれば同じ入力・反復数で再現できる。
#[allow(dead_code)]
mod random_search {
    include!("../framework/random_search.rs");
    pub fn solve_seeded<S: crate::framework::constructive::ConstructiveState>(
        input: &S::Input,
        initial: &S,
        trials: usize,
        top_k: usize,
        seed: u64,
    ) -> S {
        use rand::SeedableRng;
        let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(seed);
        solve_randomized_greedy(input, initial, trials, top_k, &mut rng)
    }
}
