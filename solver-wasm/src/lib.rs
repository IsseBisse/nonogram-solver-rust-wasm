use wasm_bindgen::prelude::*;

mod model;

#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);
}

#[wasm_bindgen]
pub fn solve(hints_x_str: &str, hints_y_str: &str) {
    let hints_x = parse_array_string(hints_x_str);
    let hints_y = parse_array_string(hints_y_str);

    alert(&format!(
        "X: {:?}, Y: {:?}", 
        hints_x, 
        hints_y
    )); 
}

fn parse_array_string(s: &str) -> Vec<Vec<i32>> {
    s.split(';')
        .map(|row| {
            row.split(',')
                .filter_map(|n| n.parse::<i32>().ok())
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
}