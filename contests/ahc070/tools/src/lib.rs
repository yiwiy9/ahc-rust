#![allow(non_snake_case, unused_macros)]

use itertools::*;
use proconio::input;
use rand::prelude::*;
use std::ops::RangeBounds;
use svg::node::element::{
    Circle, Definitions, Group, Line, Marker, Path, Rectangle, Style, Text,
};

pub trait SetMinMax {
    fn setmin(&mut self, v: Self) -> bool;
    fn setmax(&mut self, v: Self) -> bool;
}
impl<T> SetMinMax for T
where
    T: PartialOrd,
{
    fn setmin(&mut self, v: T) -> bool {
        *self > v && {
            *self = v;
            true
        }
    }
    fn setmax(&mut self, v: T) -> bool {
        *self < v && {
            *self = v;
            true
        }
    }
}

#[macro_export]
macro_rules! mat {
    ($($e:expr),*) => { Vec::from(vec![$($e),*]) };
    ($($e:expr,)*) => { Vec::from(vec![$($e),*]) };
    ($e:expr; $d:expr) => { Vec::from(vec![$e; $d]) };
    ($e:expr; $d:expr $(; $ds:expr)+) => { Vec::from(vec![mat![$e $(; $ds)*]; $d]) };
}

#[derive(Clone, Debug)]
pub struct Input {
    pub N: usize,                     // 盤面サイズ（100）
    pub M: usize,                     // 定義できる移動の数
    pub targets: Vec<(usize, usize)>, // 各ターンのターゲット座標 (a_t, b_t)
}

impl std::fmt::Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} {}", self.N, self.M)?;
        for &(x, y) in &self.targets {
            writeln!(f, "{x} {y}")?;
        }
        Ok(())
    }
}

pub fn parse_input(f: &str) -> Input {
    let f = proconio::source::once::OnceSource::from(f);
    input! {
        from f,
        N: usize,
        M: usize,
        targets: [(usize, usize); N*N], // N*N行分の (a_t, b_t) を取得
    }

    Input { N, M, targets }
}

pub fn read<
    T: Copy + PartialOrd + std::fmt::Display + std::str::FromStr,
    R: RangeBounds<T>,
>(
    token: Option<&str>,
    range: R,
) -> Result<T, String> {
    if let Some(v) = token {
        if let Ok(v) = v.parse::<T>() {
            if !range.contains(&v) {
                Err(format!("Out of range: {}", v))
            } else {
                Ok(v)
            }
        } else {
            Err(format!("Parse error: {}", v))
        }
    } else {
        Err("Unexpected EOF".to_owned())
    }
}

#[derive(Clone, Debug)]
pub struct Output {
    pub m: Vec<(i32, i32)>, // 3つの移動ベクトル [(x0, y0), (x1, y1), (x2, y2)]
    pub actions: Vec<usize>, // 各ターンの選択 (0, 1, 2) の長さ T の配列
}

pub fn parse_output(input: &Input, f: &str) -> Result<Output, String> {
    let mut tokens = f.split_whitespace();

    // 1. 3つのベクトル（計6個の整数）をパース
    let mut m = vec![];
    for _ in 0..3 {
        let x = read(tokens.next(), 0..input.N as i32)?;
        let y = read(tokens.next(), 0..input.N as i32)?;
        m.push((x, y));
    }

    // 2. 残りのトークンから各ターンの移動指示（'0', '1', '2'）をパース
    let mut actions = vec![];
    for _ in 0..input.N * input.N {
        let c = read(tokens.next(), 0..input.M)?;
        actions.push(c);
    }

    // 3. ターン数のバリデーション（多すぎても少なすぎてもダメ）
    if let Some(v) = tokens.next() {
        return Err(format!("Unexpected token: {}", v));
    }

    Ok(Output { m, actions })
}

#[derive(Clone, Debug)]
pub struct Sim {
    pub prev_pos: (usize, usize),
    pub cur_pos: (usize, usize),
    pub cur_target: (usize, usize),
    pub visited_at: Vec<Vec<Option<usize>>>,
    pub unique_count: usize,
    pub stepwise_costs: Vec<i64>,
}

impl Sim {
    fn new(input: &Input) -> Self {
        Sim {
            prev_pos: (0, 0),
            cur_pos: (0, 0),
            cur_target: (0, 0),
            visited_at: vec![vec![None; input.N]; input.N],
            unique_count: 0,
            stepwise_costs: vec![],
        }
    }

    fn total_cost(self: &Self, t: usize) -> i64 {
        self.stepwise_costs[..t].iter().sum::<i64>()
    }
}

pub fn compute_score(input: &Input, out: &Output) -> (i64, String) {
    let (_, mut score, err, _) =
        compute_score_details(input, out, input.N * input.N);
    if !err.is_empty() {
        score = 0;
    }
    (score, err)
}

fn compute_score_details(
    input: &Input,
    out: &Output,
    t: usize,
) -> (i64, i64, String, Sim) {
    // 1. 出力（actions）の長さのバリデーション
    if out.actions.len() != input.N * input.N {
        return (
            0,
            0,
            format!(
                "Invalid action length: {} (expected {})",
                out.actions.len(),
                input.N * input.N
            ),
            Sim::new(input),
        );
    }

    // 愚直な距離計算を高速に行うため、訪問済みの座標リスト（黒マスリスト）を保持
    let mut black_cells = vec![];
    let mut sim = Sim::new(input);
    let mut sim_at_t = None;

    // t = 0（初期状態）のスナップショットを保存
    if t == 0 {
        sim_at_t = Some(sim.clone());
    }

    // 全 T ターンをシミュレートして「最終スコア」を計算する
    for step in 0..input.N * input.N {
        let action = out.actions[step];
        if action >= out.m.len() {
            return (
                0,
                0,
                format!("Invalid action index {} at turn {}", action, step),
                Sim::new(input),
            );
        }

        // 移動ベクトルの適用
        let (dx, dy) = out.m[action];

        let (x, y) = sim.cur_pos;
        let x1 = (x as i32 + dx).rem_euclid(input.N as i32) as usize;
        let y1 = (y as i32 + dy).rem_euclid(input.N as i32) as usize;

        // 初めて踏むマスならユニークカウント用にリストへ追加
        if sim.visited_at[x1][y1].is_none() {
            sim.visited_at[x1][y1] = Some(step);
            black_cells.push((x1, y1));
            sim.unique_count += 1;
        }

        sim.prev_pos = sim.cur_pos;
        sim.cur_pos = (x1, y1);

        // 距離ペナルティの計算（その時点の全黒マスとの愚直ループ：最大10000個）
        let min_dist = {
            let (a, b) = input.targets[step];
            black_cells
                .iter()
                .map(|(x, y)| x.abs_diff(a) + y.abs_diff(b))
                .min()
                .unwrap()
        };

        let t_weight = ((step + 1) as f64).sqrt();
        let step_cost = (min_dist as f64 * t_weight) as i64; // 正の値なのでキャストで切り捨て
        sim.stepwise_costs.push(step_cost);

        // 指定された「t個の操作を実行した直後」のスナップショットを保持
        if step + 1 == t {
            sim_at_t = Some(sim.clone());
            if let Some(sim) = sim_at_t.as_mut() {
                sim.cur_target = input.targets[step];
            }
        }
    }

    let cost = sim.total_cost(input.N * input.N);
    // let x = 100.0 * (input.N as f64).powf(3.0) / (1.0 + cost as f64);
    // let score = (1e6 * f64::log2(x)).round();
    let score =
        (1e6 * (input.N as f64).powf(3.0) / (1.0 + cost as f64)).round();

    (cost, score as i64, String::new(), sim_at_t.unwrap())
}

pub fn gen(seed: u64) -> Input {
    let N = 100;
    let M = 3;
    let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(seed);
    let mut targets = iproduct!(0..N, 0..N).collect_vec();
    targets.shuffle(&mut rng);
    Input { N, M, targets }
}

/// 0 <= val <= 1
fn color(mut val: f64) -> String {
    val.setmin(1.0);
    val.setmax(0.0);
    let (r, g, b) = if val < 0.5 {
        let x = val * 2.0;
        (
            30. * (1.0 - x) + 144. * x,
            144. * (1.0 - x) + 255. * x,
            255. * (1.0 - x) + 30. * x,
        )
    } else {
        let x = val * 2.0 - 1.0;
        (
            144. * (1.0 - x) + 255. * x,
            255. * (1.0 - x) + 30. * x,
            30. * (1.0 - x) + 70. * x,
        )
    };
    format!(
        "#{:02x}{:02x}{:02x}",
        r.round() as i32,
        g.round() as i32,
        b.round() as i32
    )
}

fn rect(x: f64, y: f64, w: f64, h: f64, fill: &str) -> Rectangle {
    Rectangle::new()
        .set("x", x)
        .set("y", y)
        .set("width", w)
        .set("height", h)
        .set("fill", fill)
}

pub fn vis_default(input: &Input, out: &Output) -> (i64, String, String) {
    let (mut score, err, svg) = vis(input, out, input.N * input.N, true, true);
    if !err.is_empty() {
        score = 0;
    }
    (score, err, svg)
}

fn vis_text(
    input: &Input,
    out: &Output,
    t: usize,
    sim: &Sim,
) -> (Group, f64, f64) {
    let m_line = format!(
        "(i0, j0)=({}, {})  (i1, j1)=({}, {})  (i2, j2)=({}, {})",
        out.m[0].0, out.m[0].1, out.m[1].0, out.m[1].1, out.m[2].0, out.m[2].1
    );

    let stats_line = format!(
        "Total Cost: {} | Visited Cells: {} / {}",
        sim.total_cost(t),
        sim.unique_count,
        input.N * input.N
    );

    let loc_line = if t > 0 {
        format!(
            "Move: ({}, {}) → ({}, {}) | Target: ({}, {}) | Cost: {}",
            sim.prev_pos.0,
            sim.prev_pos.1,
            sim.cur_pos.0,
            sim.cur_pos.1,
            sim.cur_target.0,
            sim.cur_target.1,
            sim.stepwise_costs[t - 1],
        )
    } else {
        "".to_owned()
    };

    let mut g = Group::new();

    g = g.add(Style::new(
        "text { font-family: monospace; dominant-baseline: middle; } .center { text-anchor: middle; } .left { text-anchor: start; }",
    ));

    g = g.add(
        Text::new(m_line)
            .set("y", 14)
            .set("font-size", 14)
            .set("class", "left")
            .set("fill", "#222222"),
    );
    g = g.add(
        Text::new(stats_line)
            .set("y", 40)
            .set("font-size", 14)
            .set("class", "left")
            .set("fill", "#0066cc")
            .set("font-weight", "bold"),
    );
    g = g.add(
        Text::new(loc_line)
            .set("y", 66)
            .set("font-size", 14)
            .set("class", "left")
            .set("fill", "#0066cc")
            .set("font-weight", "bold"),
    );

    (g, 600.0, 94.0)
}

fn compute_split_ratio(from: f64, to: f64, width: f64) -> f64 {
    let len = width - (from - to);
    (width - from) / len
}

fn split_point(a: f64, b: f64, ratio: f64) -> f64 {
    a + (b - a) * ratio
}

fn vis_board(input: &Input, t: usize, sim: &Sim) -> (Group, f64, f64) {
    let board_w = 600.0;
    let D = board_w / input.N as f64;

    // 現在のターンのターゲット座標を取得
    let current_target = if t > 0 {
        Some(input.targets[t - 1])
    } else {
        None
    };

    let mut g = Group::new();

    g = g.add(
        Definitions::new().add(
            Marker::new()
                .set("id", "move-arrow")
                .set("viewBox", (0, 0, 10, 10))
                .set("refX", 10)
                .set("refY", 5)
                .set("markerWidth", 12)
                .set("markerHeight", 12)
                .set("markerUnits", "userSpaceOnUse")
                .set("orient", "auto")
                .add(
                    Path::new()
                        .set("d", "M 0 0 L 10 5 L 0 10")
                        .set("fill", "none")
                        .set("stroke", "black")
                        .set("stroke-width", 2),
                ),
        ),
    );

    // 100x100 のマスの描画
    for i in 0..input.N {
        for j in 0..input.N {
            let last_turn_opt = sim.visited_at[i][j];
            let is_target = current_target == Some((i, j));

            // マスの色決定ロジック
            let fill = if is_target {
                "#ff4d4d".to_string() // 現在のターゲットは最優先で目立つ「赤」（固定値）
            } else if let Some(turn) = last_turn_opt {
                let max_turn = (input.N * input.N - 1).max(1) as f64;
                color(turn as f64 / max_turn)
            } else {
                "#ffffff".to_string() // 一度も訪れていないマスは白
            };

            // セルを描画
            let mut r_cell = rect(j as f64 * D, i as f64 * D, D, D, &fill);

            if is_target && last_turn_opt.is_some() {
                r_cell = r_cell.set("stroke", "#000000").set("stroke-width", 2);
            }

            g = g.add(r_cell);
        }
    }

    // グリッド線の描画
    for i in 0..=input.N {
        if i % 5 == 0 {
            let is_major = i % 10 == 0 || i == 0 || i == input.N;
            let stroke_color = if i == 0 || i == input.N {
                "black"
            } else if is_major {
                "#808080"
            } else {
                "#cccccc"
            };
            let stroke_w = if i == 0 || i == input.N { 2 } else { 1 };

            // 横線
            g = g.add(
                Line::new()
                    .set("x1", 0)
                    .set("y1", i as f64 * D)
                    .set("x2", input.N as f64 * D)
                    .set("y2", i as f64 * D)
                    .set("stroke", stroke_color)
                    .set("stroke-width", stroke_w),
            );
            // 縦線
            g = g.add(
                Line::new()
                    .set("x1", i as f64 * D)
                    .set("y1", 0)
                    .set("x2", i as f64 * D)
                    .set("y2", input.N as f64 * D)
                    .set("stroke", stroke_color)
                    .set("stroke-width", stroke_w),
            );
        }
    }

    fn vis_move(x1: f64, y1: f64, x2: f64, y2: f64) -> Path {
        let (mx, my) = ((x1 + x2) / 2.0, (y1 + y2) / 2.0);
        Path::new()
            .set("d", format!("M {x1} {y1} L {mx} {my} L {x2} {y2}"))
            .set("fill", "none")
            .set("stroke", "black")
            .set("stroke-width", 1)
            .set("marker-mid", "url(#move-arrow)")
    }

    let cell_center = |(x, y): (usize, usize)| -> (f64, f64) {
        (y as f64 * D + D / 2.0, x as f64 * D + D / 2.0)
    };

    // 移動を表す線
    if t > 0 && sim.prev_pos != sim.cur_pos {
        let (x1, y1) = cell_center(sim.prev_pos);
        let (x2, y2) = cell_center(sim.cur_pos);
        if x1 <= x2 && y1 <= y2 {
            g = g.add(vis_move(x1, y1, x2, y2));
        } else if x1 <= x2 && y1 > y2 {
            // 下端を通る
            let r = compute_split_ratio(y1, y2, board_w);
            let xx = split_point(x1, x2, r);
            g = g
                .add(vis_move(x1, y1, xx, board_w))
                .add(vis_move(xx, 0.0, x2, y2))
        } else if x1 > x2 && y1 <= y2 {
            // 右端を通る
            let r = compute_split_ratio(x1, x2, board_w);
            let yy = split_point(y1, y2, r);
            g = g
                .add(vis_move(x1, y1, board_w, yy))
                .add(vis_move(0.0, yy, x2, y2))
        } else {
            // 右端と下端を通る

            // 下端を通る時刻 (0..1)
            let rx = compute_split_ratio(y1, y2, board_w);
            // 右端を通る時刻 (0..1)
            let ry = compute_split_ratio(x1, x2, board_w);

            let eps = 0.005;
            if rx + eps < ry {
                // 下→右の順
                let xx = split_point(x1, x2 + board_w, rx);
                let yy = {
                    let r = (ry - rx) / (1.0 - rx);
                    split_point(0.0, y2, r)
                };
                g = g
                    .add(vis_move(x1, y1, xx, board_w))
                    .add(vis_move(xx, 0.0, board_w, yy))
                    .add(vis_move(0.0, yy, x2, y2))
            } else if ry + eps < rx {
                // 右→下の順
                let yy = split_point(y1, y2 + board_w, ry);
                let xx = {
                    let r = (rx - ry) / (1.0 - ry);
                    split_point(0.0, x2, r)
                };
                g = g
                    .add(vis_move(x1, y1, board_w, yy))
                    .add(vis_move(0.0, yy, xx, board_w))
                    .add(vis_move(xx, 0.0, x2, y2))
            } else {
                // ほぼ同時なので同時とみなして描画
                g = g
                    .add(vis_move(x1, y1, board_w, board_w))
                    .add(vis_move(0.0, 0.0, x2, y2))
            }
        }
    }

    // 直前の位置
    if t > 0 {
        let (cx, cy) = cell_center(sim.prev_pos);
        let radius = D;
        g = g.add(
            Circle::new()
                .set("cx", cx)
                .set("cy", cy)
                .set("r", radius)
                .set("fill", "none")
                .set("stroke", "blue")
                .set("stroke-width", 2),
        );
    }

    // 現在位置
    if t > 0 {
        let (cx, cy) = cell_center(sim.cur_pos);
        let radius = D;
        g = g.add(
            Circle::new()
                .set("cx", cx)
                .set("cy", cy)
                .set("r", radius)
                .set("fill", "none")
                .set("stroke", "black")
                .set("stroke-width", 2),
        );
    }

    // 怪異の発生位置
    if let Some(pos) = current_target {
        let (cx, cy) = cell_center(pos);
        let radius = D;
        g = g.add(
            Circle::new()
                .set("cx", cx)
                .set("cy", cy)
                .set("r", radius)
                .set("fill", "none")
                .set("stroke", "red")
                .set("stroke-width", 2),
        );
    }

    (g, board_w, board_w)
}

fn vis_graph(
    input: &Input,
    _out: &Output,
    t: usize,
    sim: &Sim,
    final_cost: i64,
) -> (Group, f64, f64) {
    let graph_w = 600.0;
    let graph_h = 400.0;
    let margin_left = 60.0;
    let margin_right = 0.0;
    let margin_top = 20.0;
    let margin_bottom = 40.0;
    let plot_w = graph_w - margin_left - margin_right;
    let plot_h = graph_h - margin_top - margin_bottom;
    let max_turn = input.N * input.N;
    let visible_turns = t.min(sim.stepwise_costs.len());

    let mut cumulative_costs = vec![0i64];
    for &stepwise_cost in sim.stepwise_costs.iter().take(visible_turns) {
        let total =
            cumulative_costs.last().copied().unwrap_or(0) + stepwise_cost;
        cumulative_costs.push(total);
    }
    let final_cost = final_cost.max(1);

    let mut g = Group::new();

    g = g.add(Style::new(
        "text { font-family: monospace; dominant-baseline: middle; }",
    ));

    let x = |turn: usize| {
        margin_left + turn as f64 * plot_w / max_turn.max(1) as f64
    };
    let y = |cost: i64| {
        let ratio = (cost.max(0) as f64 / final_cost as f64).min(1.0);
        margin_top + plot_h - (ratio * plot_h as f64).round()
    };

    // Add five evenly spaced X-axis ticks and light vertical grid lines.
    for i in 0..=4 {
        let turn = max_turn * i / 4;
        let tick_x = x(turn);
        g = g
            .add(
                Line::new()
                    .set("x1", tick_x)
                    .set("y1", margin_top)
                    .set("x2", tick_x)
                    .set("y2", margin_top + plot_h)
                    .set("stroke", "#e0e0e0")
                    .set("stroke-width", 1),
            )
            .add(
                Line::new()
                    .set("x1", tick_x)
                    .set("y1", margin_top + plot_h)
                    .set("x2", tick_x)
                    .set("y2", margin_top + plot_h + 5.0)
                    .set("stroke", "black")
                    .set("stroke-width", 1),
            )
            .add(
                Text::new(turn.to_string())
                    .set("x", tick_x)
                    .set("y", margin_top + plot_h + 16.0)
                    .set("text-anchor", "middle")
                    .set("font-size", 11),
            );
    }

    // Add five evenly spaced Y-axis ticks and light horizontal grid lines.
    for i in 0..=4 {
        let percentage = i as f64 * 25.0;
        let tick_y = margin_top + plot_h - plot_h * i as f64 / 4.0;
        g = g
            .add(
                Line::new()
                    .set("x1", margin_left)
                    .set("y1", tick_y)
                    .set("x2", margin_left + plot_w)
                    .set("y2", tick_y)
                    .set("stroke", "#e0e0e0")
                    .set("stroke-width", 1),
            )
            .add(
                Line::new()
                    .set("x1", margin_left - 5.0)
                    .set("y1", tick_y)
                    .set("x2", margin_left)
                    .set("y2", tick_y)
                    .set("stroke", "black")
                    .set("stroke-width", 1),
            )
            .add(
                Text::new(format!("{}%", percentage))
                    .set("x", margin_left - 8.0)
                    .set("y", tick_y)
                    .set("text-anchor", "end")
                    .set("font-size", 11),
            );
    }

    // Axes and labels. The X axis always spans the complete N^2 turns.
    g = g
        .add(
            Line::new()
                .set("x1", margin_left)
                .set("y1", margin_top)
                .set("x2", margin_left)
                .set("y2", margin_top + plot_h)
                .set("stroke", "black")
                .set("stroke-width", 1),
        )
        .add(
            Line::new()
                .set("x1", margin_left)
                .set("y1", margin_top + plot_h)
                .set("x2", margin_left + plot_w)
                .set("y2", margin_top + plot_h)
                .set("stroke", "black")
                .set("stroke-width", 1),
        )
        .add(
            Text::new("Turn")
                .set("x", margin_left + plot_w / 2.0)
                .set("y", graph_h - 12.0)
                .set("text-anchor", "middle")
                .set("font-size", 12),
        )
        .add(
            Text::new("Cost / Final Cost")
                .set("x", 14)
                .set("y", margin_top + plot_h / 2.0)
                .set(
                    "transform",
                    format!("rotate(-90 14 {})", margin_top + plot_h / 2.0),
                )
                .set("text-anchor", "middle")
                .set("font-size", 12),
        );

    for turn in 1..cumulative_costs.len() {
        g = g.add(
            Line::new()
                .set("x1", x(turn - 1))
                .set("y1", y(cumulative_costs[turn - 1]))
                .set("x2", x(turn))
                .set("y2", y(cumulative_costs[turn]))
                .set("stroke", "#0066cc")
                .set("stroke-width", 2)
                .set("fill", "none"),
        );
    }

    let final_cost_label = format!("Final Cost: {}", final_cost);
    let label_x = margin_left + 8.0;
    let label_y = margin_top + 16.0;
    g = g
        .add(
            Rectangle::new()
                .set("x", label_x - 4.0)
                .set("y", margin_top + 4.0)
                .set("width", final_cost_label.len() * 8 + 8)
                .set("height", 20)
                .set("fill", "white")
                .set("fill-opacity", 0.85),
        )
        .add(
            Text::new(final_cost_label)
                .set("x", label_x)
                .set("y", label_y)
                .set("text-anchor", "start")
                .set("font-size", 12)
                .set("font-weight", "bold"),
        );

    (g, graph_w, graph_h)
}

pub fn vis(
    input: &Input,
    out: &Output,
    t: usize,
    _show_number: bool,
    _show_targets: bool,
) -> (i64, String, String) {
    let (cost, score, err, sim) = compute_score_details(input, out, t);

    // テキスト情報
    let (g1, w1, h1) = vis_text(input, out, t, &sim);

    // 盤面
    let (g2, w2, h2) = vis_board(input, t, &sim);

    // グラフ
    let (g3, w3, h3) = vis_graph(input, out, t, &sim, cost);

    let mut doc = svg::Document::new();

    let mut y = 5.0;
    doc = doc.add(g1.set("transform", format!("translate(5 {})", y)));

    y += h1;
    doc = doc.add(g2.set("transform", format!("translate(5 {})", y)));

    y += h2;
    doc = doc.add(g3.set("transform", format!("translate(5 {})", y)));

    y += h3;

    let x = w1.max(w2).max(w3);

    doc = doc
        .set("id", "vis")
        .set("viewBox", (-5, -5, x + 10.0, y + 10.0))
        .set("width", x + 10.0)
        .set("height", y + 10.0)
        .set("style", "background-color:white");

    (score, err, doc.to_string())
}
