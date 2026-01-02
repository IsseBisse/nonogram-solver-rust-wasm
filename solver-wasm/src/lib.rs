use wasm_bindgen::prelude::*;

use crate::model::{Constraints, Board};

mod model;

// TODO: Add support for sending state
pub fn solve(constraints_x_str: &str, constraints_y_str: &str, dimensions: &str) -> bool {
    let constraints = Constraints::parse(constraints_x_str, constraints_y_str, dimensions);

    let board = Board::new(constraints);
    let solutions = board.solve();

    solutions.len() > 1
}

