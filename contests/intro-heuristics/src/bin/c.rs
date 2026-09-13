use proconio::{input, marker::Usize1};
use std::collections::BTreeSet;

const CONTEST_TYPES: usize = 26;

fn calculate_score(c: &[i64], s: &[Vec<i64>], t: &[usize]) -> i64 {
    let mut score = 0_i64;
    let mut last_held = [0_i64; CONTEST_TYPES];

    for (day_index, &contest) in t.iter().enumerate() {
        let day = day_index as i64 + 1;
        score += s[day_index][contest];
        last_held[contest] = day;

        for contest in 0..CONTEST_TYPES {
            score -= c[contest] * (day - last_held[contest]);
        }
    }

    score
}

fn dissatisfaction_reduction(c: i64, previous: usize, day: usize, next: usize) -> i64 {
    c * (day - previous) as i64 * (next - day) as i64
}

fn main() {
    input! {
        d: usize,
        c: [i64; CONTEST_TYPES],
        s: [[i64; CONTEST_TYPES]; d],
        mut t: [Usize1; d],
        m: usize,
        queries: [(Usize1, Usize1); m],
    }

    let mut held_days = vec![BTreeSet::new(); CONTEST_TYPES];
    for days in &mut held_days {
        days.insert(0);
        days.insert(d + 1);
    }
    for (day_index, &contest) in t.iter().enumerate() {
        held_days[contest].insert(day_index + 1);
    }

    let mut score = calculate_score(&c, &s, &t);

    for (day_index, new_contest) in queries {
        let day = day_index + 1;
        let old_contest = t[day_index];

        let previous = *held_days[old_contest].range(..day).next_back().unwrap();
        let next = *held_days[old_contest].range(day + 1..).next().unwrap();

        score -= s[day_index][old_contest];
        score -= dissatisfaction_reduction(c[old_contest], previous, day, next);
        held_days[old_contest].remove(&day);

        let previous = *held_days[new_contest].range(..day).next_back().unwrap();
        let next = *held_days[new_contest].range(day..).next().unwrap();

        score += s[day_index][new_contest];
        score += dissatisfaction_reduction(c[new_contest], previous, day, next);
        held_days[new_contest].insert(day);
        t[day_index] = new_contest;

        debug_assert_eq!(score, calculate_score(&c, &s, &t));
        println!("{score}");
    }
}
