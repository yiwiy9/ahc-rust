use super::*;
use svg::node::element::{Circle, Group, Line, Rectangle, Style, Text};

#[derive(Clone, Debug)]
pub struct VisOption {
    pub t: usize,
    pub L: usize,
}

impl Default for VisOption {
    fn default() -> Self {
        Self {
            t: usize::MAX,
            L: 16,
        }
    }
}

impl VisOption {
    #[allow(unused_variables)]
    pub fn set_bool(&mut self, name: &str, value: bool) -> Result<(), String> {
        match name {
            _ => Err(format!("Unknown visualizer option: {}", name)),
        }
    }

    #[allow(unused_variables)]
    pub fn set_i32(&mut self, name: &str, value: i32) -> Result<(), String> {
        match name {
            "L" if (1..=256).contains(&value) => {
                self.L = value as usize;
                Ok(())
            }
            "L" => Err("L must be between 1 and 256".to_owned()),
            _ => Err(format!("Unknown visualizer option: {}", name)),
        }
    }

    #[allow(unused_variables)]
    pub fn set_string(&mut self, name: &str, value: &str) -> Result<(), String> {
        match name {
            _ => Err(format!("Unknown visualizer option: {}", name)),
        }
    }
}

#[allow(dead_code)]
fn group(title: String) -> Group {
    Group::new().set("data-tooltip", title)
}

fn text_left(x: i32, y: i32, s: String, size: usize) -> Text {
    Text::new(s)
        .set("x", x)
        .set("y", y)
        .set("font-size", size)
        .set("fill", "black")
        .set("style", "text-anchor:start;user-select:text;cursor:text")
}

fn error_text(x: i32, y: i32, s: String) -> Text {
    text_left(x, y, s, 15)
        .set("fill", "red")
        .set("style", "text-anchor:start;user-select:text;cursor:text")
}

fn info_line(x: i32, y: i32, col_w: i32, size: usize, items: &[String]) -> Group {
    let mut g = Group::new().set("style", "user-select:text;cursor:text");
    for (i, s) in items.iter().enumerate() {
        g = g.add(text_left(x + col_w * i as i32, y, s.clone(), size));
    }
    g
}

pub struct VisData {
    output: Output,
    evaluation: Evaluation,
    states: Vec<State>,
    error: Option<String>,
    board: String,
}
impl VisData {
    pub fn new(input: &Input, output: Output) -> Self {
        let evaluation = evaluate(input, &output);
        let mut states = vec![State::new(input, &output)];
        let mut error = None;
        for action in &output.actions {
            let mut state = states.last().unwrap().clone();
            if let Err(e) = state.apply(input, action) {
                error = Some(e);
                break;
            }
            states.push(state);
        }
        let board = board_background(input);
        Self {
            output,
            evaluation,
            states,
            error,
            board,
        }
    }
    pub fn verdict(&self) -> &Result<i64, String> {
        self.evaluation.verdict()
    }
}
pub fn get_max_turn(_input: &Input, data: &VisData, _option: &VisOption) -> usize {
    data.states.len() - 1 + usize::from(data.error.is_some())
}
const W: i32 = 1040;
const H: i32 = 810;
const BOARD: f64 = 768.0;
const TOP: f64 = 65.0;
const LEFT: f64 = 10.0;
fn xy(input: &Input, p: Point) -> (f64, f64) {
    (
        LEFT + (p.1 as f64 + 0.5) * BOARD / input.N as f64,
        TOP + (p.0 as f64 + 0.5) * BOARD / input.N as f64,
    )
}
fn board_background(input: &Input) -> String {
    let cell = BOARD / input.N as f64;
    let mut path = String::new();
    for k in 0..=input.N {
        let x = LEFT + k as f64 * cell;
        let y = TOP + k as f64 * cell;
        path += &format!(
            "M {} {} V {} M {} {} H {} ",
            x,
            TOP,
            TOP + BOARD,
            LEFT,
            y,
            LEFT + BOARD
        );
    }
    format!(
        "<g id=\"board-grid\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"white\"/><path d=\"{}\" fill=\"none\" stroke=\"#eceff1\" stroke-width=\"0.4\"/></g>",
        LEFT, TOP, BOARD, BOARD, path
    )
}
const CELL_HOVER: &str = r#"
const tip = this.querySelector('#cell-tooltip');
const matrix = this.getScreenCTM();
if (!matrix) return;
const p = this.createSVGPoint();
p.x = event.clientX;
p.y = event.clientY;
const q = p.matrixTransform(matrix.inverse());
const left = Number(this.dataset.boardLeft);
const top = Number(this.dataset.boardTop);
const size = Number(this.dataset.boardSize);
const n = Number(this.dataset.boardN);
if (q.x < left || q.x >= left + size || q.y < top || q.y >= top + size) {
    tip.setAttribute('visibility', 'hidden');
    return;
}
const i = Math.floor((q.y - top) * n / size);
const j = Math.floor((q.x - left) * n / size);
const target = event.target.closest('[data-tooltip]');
const label = tip.querySelector('text');
const cell = '(' + i + ', ' + j + ')';
label.textContent = target ? target.dataset.tooltip + ' | cell ' + cell : cell;
const width = Math.ceil(label.getComputedTextLength()) + 14;
tip.querySelector('rect').setAttribute('width', width);
tip.setAttribute('transform', 'translate(' + Math.max(0, Math.min(q.x + 12, this.viewBox.baseVal.width - width - 6)) + ' ' + Math.min(q.y + 12, this.viewBox.baseVal.height - 30) + ')');
tip.setAttribute('visibility', 'visible');
"#;
fn cell_tooltip() -> Group {
    Group::new()
        .set("id", "cell-tooltip")
        .set("visibility", "hidden")
        .set("pointer-events", "none")
        .add(
            Rectangle::new()
                .set("width", 94)
                .set("height", 24)
                .set("rx", 4)
                .set("fill", "#fff")
                .set("stroke", "#64748b"),
        )
        .add(text_left(7, 12, String::new(), 14))
}
fn panel(x: i32, y: i32, width: i32, height: i32) -> Rectangle {
    Rectangle::new()
        .set("x", x)
        .set("y", y)
        .set("width", width)
        .set("height", height)
        .set("rx", 6)
        .set("fill", "#f5f7fa")
        .set("stroke", "#cfd8dc")
        .set("stroke-width", 1)
}
fn base_document(
    input: &Input,
    t: usize,
    score: i64,
    error: &str,
    visited: usize,
    total: usize,
) -> svg::Document {
    let mut doc = svg::Document::new()
        .set("id", "vis")
        .set("data-board-left", LEFT)
        .set("data-board-top", TOP)
        .set("data-board-size", BOARD)
        .set("data-board-n", input.N)
        .set("onpointermove", CELL_HOVER)
        .set(
            "onpointerleave",
            "this.querySelector('#cell-tooltip').setAttribute('visibility', 'hidden')",
        )
        .set("width", W)
        .set("height", H + 40)
        .set("viewBox", (0, 0, W, H + 40))
        .set("style", "background-color:white")
        .add(Style::new(
            "text {text-anchor:middle;dominant-baseline:central;font-family:sans-serif}",
        ))
        .add(panel(4, 3, 248, 27))
        .add(panel(264, 3, 248, 27))
        .add(panel(524, 3, 248, 27))
        .add(panel(786, 64, 244, 64))
        .add(panel(786, 134, 244, 112))
        .add(panel(786, 254, 244, 52))
        .add(info_line(
            10,
            16,
            260,
            16,
            &[
                format!("t = {}", t),
                format!("score = {}", score),
                format!("visited = {} / {}", visited, total),
            ],
        ));
    if !error.is_empty() {
        doc = doc.add(error_text(10, 42, error.to_owned()));
    }
    doc
}
fn board_state(input: &Input, state: &State, trail: Group, previous: Option<&Action>) -> Group {
    let mut board = Group::new();
    for (r, &p) in input.goals.iter().enumerate() {
        let (x, y) = xy(input, p);
        let color = if r < state.visited {
            "white"
        } else if r == state.visited {
            "#e53935"
        } else {
            "#1976d2"
        };
        board = board.add(
            Circle::new()
                .set("cx", x)
                .set("cy", y)
                .set("r", if r == state.visited { 5 } else { 3 })
                .set("fill", color)
                .set("stroke", if r < state.visited { "#94a3b8" } else { color })
                .set("stroke-width", 1)
                .set(
                    "data-tooltip",
                    format!(
                        "g_{} = ({}, {}) [{}]",
                        r,
                        p.0,
                        p.1,
                        if r < state.visited {
                            "visited"
                        } else if r == state.visited {
                            "next"
                        } else {
                            "unvisited"
                        }
                    ),
                ),
        );
    }
    board = board.add(trail);
    let mut counts = std::collections::BTreeMap::new();
    for &p in &state.circles {
        *counts.entry(p).or_insert(0usize) += 1;
    }
    board = board.add(text_left(795, 148, "Magic circles".to_owned(), 16));
    for (&p, &count) in &counts {
        let (x, y) = xy(input, p);
        board = board.add(
            group(format!("magic circles at ({}, {}): {}", p.0, p.1, count)).add(
                Circle::new()
                    .set("cx", x)
                    .set("cy", y)
                    .set("r", 5)
                    .set("fill", "none")
                    .set("pointer-events", "all")
                    .set("stroke", "#8e24aa")
                    .set("stroke-width", 2),
            ),
        );
    }
    let mut circles = state.circles.clone();
    circles.sort_unstable();
    for (row, p) in circles.iter().enumerate() {
        board = board.add(
            text_left(
                795,
                174 + 26 * row as i32,
                format!("({}, {})", p.0, p.1),
                16,
            )
            .set("style", "text-anchor:start;user-select:text"),
        );
    }
    let (x, y) = xy(input, state.position);
    board = board.add(
        Circle::new()
            .set("cx", x)
            .set("cy", y)
            .set("r", 3.5)
            .set("fill", "#111")
            .set(
                "data-tooltip",
                format!("Takahashi = ({}, {})", state.position.0, state.position.1),
            ),
    );
    board = board.add(text_left(
        795,
        80,
        format!("p = ({}, {})", state.position.0, state.position.1),
        16,
    ));
    let next = if state.visited < input.M {
        let p = input.goals[state.visited];
        format!("g{} = ({}, {})", state.visited, p.0, p.1)
    } else {
        "Complete".to_owned()
    };
    board = board.add(text_left(795, 110, next, 16));
    if let Some(action) = previous {
        let label = match action {
            Action::Teleport((a, b)) => format!("J {} {}", a, b),
            _ => format!("{:?}", action),
        };
        board = board.add(text_left(795, 280, format!("op = {}", label), 16));
    }
    board
}
fn draw_trail(input: &Input, data: &VisData, end: usize, length: usize) -> Group {
    let mut trail = Group::new().set("id", "trail");
    for step in end.saturating_sub(length)..end {
        let prev = &data.states[step];
        let state = &data.states[step + 1];
        let action = &data.output.actions[step];
        let mut segment = group(format!(
            "Operation {}: ({}, {}) to ({}, {})",
            step + 1,
            prev.position.0,
            prev.position.1,
            state.position.0,
            state.position.1
        ));
        let (x, y) = xy(input, prev.position);
        let (nx, ny) = xy(input, state.position);
        segment = segment.add(
            Line::new()
                .set("x1", x)
                .set("y1", y)
                .set("x2", nx)
                .set("y2", ny)
                .set("stroke", "#263238")
                .set("stroke-width", 2)
                .set(
                    "stroke-dasharray",
                    if matches!(action, Action::Teleport(_)) {
                        "6 4"
                    } else {
                        "none"
                    },
                ),
        );
        if let Action::Teleport(p) = *action {
            let (px, py) = xy(input, p);
            segment = segment.add(
                Line::new()
                    .set("x1", px)
                    .set("y1", py)
                    .set("x2", x)
                    .set("y2", y)
                    .set("stroke", "#8e24aa")
                    .set("stroke-width", 2)
                    .set("stroke-dasharray", "2 4"),
            );
        }
        trail = trail.add(segment);
    }
    trail
}
pub struct VisResult {
    pub svg: String,
}
pub fn vis(input: &Input, data: &VisData, option: &VisOption) -> VisResult {
    let max = get_max_turn(input, data, option);
    let t = option.t.min(max);
    let index = t.min(data.states.len() - 1);
    let state = &data.states[index];
    let (score, error) = if t == max {
        match data.verdict() {
            Ok(s) => (*s, String::new()),
            Err(e) => (0, e.clone()),
        }
    } else {
        (state.score(input), String::new())
    };
    let previous = if index > 0 && t == index {
        Some(&data.output.actions[index - 1])
    } else {
        None
    };
    let svg = base_document(input, t, score, &error, state.visited, input.M)
        .add(svg::node::Blob::new(data.board.clone()))
        .add(board_state(
            input,
            state,
            draw_trail(input, data, index, option.L),
            previous,
        ))
        .add(cell_tooltip())
        .to_string();
    VisResult { svg }
}
pub fn draw_parse_error(input: &Input, error: &str, _option: &VisOption) -> String {
    let state = State {
        position: input.s,
        circles: vec![],
        visited: 0,
        steps: 0,
    };
    base_document(input, 0, 0, error, 0, input.M)
        .add(svg::node::Blob::new(board_background(input)))
        .add(board_state(input, &state, Group::new(), None))
        .add(cell_tooltip())
        .to_string()
}
