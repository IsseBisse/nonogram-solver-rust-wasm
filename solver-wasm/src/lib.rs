use wasm_bindgen::prelude::*;

use crate::model::{Constraint, Constraints, Dimensions, Board};

mod model;

#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);
}

#[wasm_bindgen]
pub fn solve(constraints_x_str: &str, constraints_y_str: &str, dimensions: &str) -> String {
    let constraints_row = parse_array_string(constraints_y_str)
        .into_iter()
        .map(|values|{
            Constraint::new(values)
        })
        .collect();
    let constraints_col = parse_array_string(constraints_x_str)
        .into_iter()
        .map(|values|{
            Constraint::new(values)
        })
        .collect();
    let constraints = Constraints::new(constraints_row, constraints_col);

    let dimensions = parse_dim_string(dimensions);

    let mut board = Board::new(constraints, dimensions);
    board.solve()

    // board.to_string()
}

fn parse_dim_string(s: &str) -> Dimensions {
    let parts = s.split("x").collect::<Vec<&str>>();

    let num_cols = parts[0].parse().unwrap();
    let num_rows = parts[1].parse().unwrap();
    Dimensions::new(num_rows, num_cols)
}

fn parse_array_string(s: &str) -> Vec<Vec<usize>> {
    s.split(';')
        .map(|row| {
            row.split(',')
                .filter_map(|n| n.parse::<usize>().ok())
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_array_string() {
        let input1 = "1,2,3;4,5,6";
        let input2 = "7,8;9,10,11";
        
        let arr1 = parse_array_string(input1);
        let arr2 = parse_array_string(input2);
        
        println!("Array 1: {:?}", arr1);
        println!("Array 2: {:?}", arr2);
        
        assert_eq!(arr1, vec![vec![1, 2, 3], vec![4, 5, 6]]);
        assert_eq!(arr2, vec![vec![7, 8], vec![9, 10, 11]]);
    }

    #[test]
    fn test_solve() {
        let hints_x_str = "1,2;4;2,1;1,1;1"; 
        let hints_y_str = ";5;2;2,1;3";
        let dimensions_str = "5x5";

        solve(hints_x_str, hints_y_str, dimensions_str);
    }
}