use super::*;
use svg::node::element::{
    Circle, ClipPath, Definitions, Group, Line, Path, Pattern, Rectangle, Style, Text, Title,
};

#[derive(Clone, Debug)]
pub struct VisOption {
    pub t: usize,
}
impl Default for VisOption {
    fn default() -> Self {
        Self { t: usize::MAX }
    }
}
impl VisOption {
    pub fn set_bool(&mut self, name: &str, _value: bool) -> Result<(), String> {
        Err(format!("Unknown visualizer option: {}", name))
    }
    pub fn set_i32(&mut self, name: &str, _value: i32) -> Result<(), String> {
        Err(format!("Unknown visualizer option: {}", name))
    }
    pub fn set_string(&mut self, name: &str, _value: &str) -> Result<(), String> {
        Err(format!("Unknown visualizer option: {}", name))
    }
}

pub struct VisData {
    output: Output,
    evaluation: Evaluation,
}
impl VisData {
    pub fn new(input: &Input, output: Output) -> Self {
        let evaluation = evaluate(input, &output);
        Self { output, evaluation }
    }
    pub fn verdict(&self) -> &Result<i64, String> {
        self.evaluation.verdict()
    }
}

pub fn get_max_turn(_input: &Input, data: &VisData, _option: &VisOption) -> usize {
    data.output.solutions.len().saturating_sub(1)
}

fn text_left(x: f64, y: f64, s: String, size: usize) -> Text {
    Text::new(s)
        .set("x", x)
        .set("y", y)
        .set("font-size", size)
        .set("fill", "#263442")
}
fn error_text(s: &str) -> Text {
    text_left(36.0, 54.0, s.to_owned(), 15)
        .set("fill", "#bd2736")
        .set("style", "user-select:text;cursor:text")
}
fn info_line(items: &[String]) -> Group {
    let mut g = Group::new().set("style", "user-select:text;cursor:text");
    for (i, item) in items.iter().enumerate() {
        g = g.add(text_left(36.0 + 200.0 * i as f64, 27.0, item.clone(), 17));
    }
    g
}
fn rect(x: f64, y: f64, w: f64, h: f64, fill: &str) -> Rectangle {
    Rectangle::new()
        .set("x", x)
        .set("y", y)
        .set("width", w)
        .set("height", h)
        .set("fill", fill)
}

fn draw(
    input: &Input,
    bricks: &[Brick],
    placement: Option<&Placement>,
    t: usize,
    max_t: usize,
    verdict: &Result<i64, String>,
) -> String {
    let scale = (960.0 / input.W as f64).min(640.0 / input.H as f64);
    let bw = scale * input.W as f64;
    let bh = scale * input.H as f64;
    let left = 36.0;
    let top = 75.0;
    let mut doc = svg::Document::new()
        .set("id", "vis")
        .set("width", 1032)
        .set("height", 760)
        .set("viewBox", (0, 0, 1032, 760))
        .set("style", "background-color:white")
        .add(Style::new("text {font-family: sans-serif;}"))
        .add(info_line(&[
            format!("t = {} / {}", t, max_t),
            format!("Score = {}", verdict.as_ref().copied().unwrap_or(0)),
            format!("Cost = {}", placement.map_or(0, |p| p.cost)),
            format!("Bricks = {}", bricks.len()),
            format!(
                "Covered = {} / {}",
                placement.map_or(0, |p| p.covered),
                input.holes.len()
            ),
        ]));
    if let Err(error) = verdict {
        doc = doc.add(error_text(error));
    }
    doc = doc.add(
        Definitions::new()
            .add(
                ClipPath::new()
                    .set("id", "wall-clip")
                    .add(rect(left, top, bw, bh, "white")),
            )
            .add(
                Pattern::new()
                    .set("id", "brick-grain")
                    .set("patternUnits", "userSpaceOnUse")
                    .set("width", 17)
                    .set("height", 13)
                    .add(
                        Path::new()
                            .set("d", "M2 3h2 M11 2h1 M7 9h2 M14 11h1")
                            .set("stroke", "#54291e")
                            .set("stroke-width", 0.8)
                            .set("opacity", 0.19),
                    )
                    .add(
                        Path::new()
                            .set("d", "M5 5h1 M13 6h2 M2 11h1")
                            .set("stroke", "#ffe0bc")
                            .set("stroke-width", 0.7)
                            .set("opacity", 0.23),
                    ),
            ),
    );
    let mut board = Group::new()
        .set("clip-path", "url(#wall-clip)")
        .add(rect(left, top, bw, bh, "#f2f0eb"));
    for brick in bricks {
        let x = left + brick.x as f64 * scale;
        let y = top + (input.H as f64 - brick.y as f64 - 1.0) * scale;
        let center = brick.x + brick.width / 2;
        let supported = brick.y == 0
            || (brick.y < input.H
                && center < input.W
                && placement.is_some_and(|p| p.cells[(brick.y - 1) * input.W + center].is_some()));
        let colors = ["#b96149", "#c37355", "#ad533e", "#c17a5c", "#b5684e"];
        let color = colors[(brick.x * 7 + brick.y * 11 + brick.width * 3) % colors.len()];
        let inset = 1.0;
        let width = brick.width as f64 * scale - 2.0 * inset;
        let height = scale - 2.0 * inset;
        board = board.add(
            rect(x + inset, y + inset, width, height, color)
                .set("rx", 1.1)
                .set("stroke", "#854330")
                .set("stroke-width", 0.6),
        );
        board = board
            .add(rect(x + inset, y + inset, width, height, "url(#brick-grain)").set("rx", 1.1));
        board = board.add(
            Path::new()
                .set(
                    "d",
                    format!(
                        "M{} {}v{}h{}",
                        x + 1.7,
                        y + scale - 2.0,
                        -(scale - 3.7),
                        width - 1.4
                    ),
                )
                .set("fill", "none")
                .set("stroke", "#f1b18a")
                .set("stroke-opacity", 0.55)
                .set("stroke-width", 0.7),
        );
        board = board.add(
            Line::new()
                .set("x1", x + (brick.width as f64 / 2.0 - 0.24) * scale)
                .set("x2", x + (brick.width as f64 / 2.0 + 0.24) * scale)
                .set("y1", y + scale - 1.4)
                .set("y2", y + scale - 1.4)
                .set("stroke", if supported { "#334b55" } else { "#db203a" })
                .set("stroke-width", 3),
        );
    }
    let mut holes = vec![false; input.W * input.H];
    for &(x, y) in &input.holes {
        holes[y * input.W + x] = true;
        let covered = placement.is_some_and(|p| p.cells[y * input.W + x].is_some());
        board = board.add(
            Circle::new()
                .set("cx", left + (x as f64 + 0.5) * scale)
                .set("cy", top + (input.H as f64 - y as f64 - 0.5) * scale)
                .set("r", scale * 0.25)
                .set("fill", if covered { "#167f86" } else { "#bf2340" })
                .set("stroke", "white")
                .set("stroke-width", 1),
        );
    }
    for y in 0..input.H {
        for x in 0..input.W {
            let index = placement.and_then(|p| p.cells[y * input.W + x]);
            let mut title = format!("({}, {})\nHole: {}", x, y, holes[y * input.W + x]);
            if let Some(i) = index {
                let b = &bricks[i];
                title += &format!(
                    "\nBrick {}: ({}, {}), width {}, cost {}",
                    i,
                    b.x,
                    b.y,
                    b.width,
                    input.costs[b.width / 2]
                );
            } else {
                title += "\nEmpty";
            }
            board = board.add(
                rect(
                    left + x as f64 * scale,
                    top + (input.H - 1 - y) as f64 * scale,
                    scale,
                    scale,
                    "transparent",
                )
                .set("stroke", "#334155")
                .set("stroke-opacity", if index.is_some() { 0.0 } else { 0.08 })
                .set("stroke-width", 0.5)
                .add(Title::new(title)),
            );
        }
    }
    doc = doc
        .add(board)
        .add(rect(left, top, bw, bh, "none").set("stroke", "#59636b"))
        .add(rect(left - 2.0, top + bh, bw + 4.0, 5.0, "#47545e"));
    for x in (0..input.W).step_by(5) {
        doc = doc.add(
            text_left(
                left + (x as f64 + 0.5) * scale,
                top + bh + 23.0,
                x.to_string(),
                12,
            )
            .set("text-anchor", "middle"),
        );
    }
    for y in (0..input.H).step_by(5) {
        doc = doc.add(
            text_left(
                left - 9.0,
                top + (input.H as f64 - y as f64 - 0.5) * scale + 4.0,
                y.to_string(),
                12,
            )
            .set("text-anchor", "end"),
        );
    }
    doc.to_string()
}

pub struct VisResult {
    pub svg: String,
}

pub fn vis(input: &Input, data: &VisData, option: &VisOption) -> VisResult {
    let max_t = get_max_turn(input, data, option);
    let t = option.t.min(max_t);
    let bricks = data
        .output
        .solutions
        .get(t)
        .map_or(&[][..], |b| b.as_slice());
    let placement = evaluate_solution(input, bricks);
    let verdict = if t == max_t {
        data.evaluation.verdict()
    } else {
        &placement.verdict
    };
    VisResult {
        svg: draw(input, bricks, Some(&placement), t, max_t, verdict),
    }
}

pub fn draw_parse_error(input: &Input, error: &str, _option: &VisOption) -> String {
    draw(input, &[], None, 0, 0, &Err(error.to_owned()))
}
