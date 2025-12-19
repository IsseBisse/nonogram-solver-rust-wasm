use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Deserialize, Serialize)]
struct Data {
    solution: Vec<Vec<i32>>,
    #[serde(rename = "hintsX")]
    hints_x: Vec<Vec<i32>>,
    #[serde(rename = "hintsY")]
    hints_y: Vec<Vec<i32>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct JsonLine {
    data: Data,
}

struct TestInput {
    constraints_x_str: &str, 
    constraints_y_str: &str, 
    dimensions: &str,
    solution: String
}

fn read_test_data(path: &str) -> TestData {
    let file = File::open(path).expect("Failed to open file");
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.expect("Failed to read line");
            
        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }

        // Parse the JSON line
        let parsed: JsonLine = serde_json::from_str(&line)
            .expect("Failed to parse JSON");

        // Access the variables
        let solution = &parsed.data.solution;
        let hints_x = &parsed.data.hints_x;
        let hints_y = &parsed.data.hints_y;

        // Use the variables in your test
        println!("Solution: {:?}", solution);
        println!("Hints X: {:?}", hints_x);
        println!("Hints Y: {:?}", hints_y);
            
        // Your test assertions here
        assert_eq!(solution.len(), 5);
        assert_eq!(hints_x.len(), 5);
        assert_eq!(hints_y.len(), 5);
    }
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    }
}