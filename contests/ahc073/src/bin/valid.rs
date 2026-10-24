mod problem {
    include!("../problem.rs");
}

use problem::{print_answer, read_input, validate_output, Output};

fn main() {
    let input = read_input();
    let output = Output { lines: Vec::new() };
    validate_output(&input, &output).expect("invalid output");
    print_answer(&output);
}
