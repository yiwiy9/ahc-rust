use proconio::{input, marker::Usize1};

fn main() {
    input! {
        d: usize,
        c: [i64; 26],
        s: [[i64; 26]; d],
        t: [Usize1; d]
    }

    let mut last_held = vec![0; 26];
    let mut ans: i64 = 0;
    for (j, &t_i) in t.iter().enumerate() {
        ans += s[j][t_i];
        let day = j as i64 + 1;
        last_held[t_i] = day;

        for i in 0..26 {
            ans -= c[i] * (day - last_held[i]);
        }
        println!("{}", ans);
    }
}
