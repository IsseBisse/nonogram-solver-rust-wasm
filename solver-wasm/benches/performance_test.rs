use itertools::{max, min};
use serde::{Deserialize, Serialize};
use serde_json;
use statistical::{mean, standard_deviation};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::process::Command;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use solver_wasm::solve;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct TestData {
    solution: Vec<Vec<i32>>,
    #[serde(rename = "hintsX")]
    hints_x: Vec<Vec<i32>>,
    #[serde(rename = "hintsY")]
    hints_y: Vec<Vec<i32>>,
}

impl TestData {
    fn hints_to_str(hint: &Vec<Vec<i32>>) -> String {
        hint
            .iter()
            .map(|line| {
                line.iter()
                    .map(|&v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(",") 
                })
            .collect::<Vec<_>>()
            .join(";")
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct JsonLine {
    data: TestData,
}

fn read_test_data(path: &str) -> Vec<TestData> {
    let file = File::open(path).expect("Failed to open file");
    let reader = BufReader::new(file);

    let testData = reader
        .lines()
        .map(|line| line.expect("Failed to read line"))
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let parsed: JsonLine = serde_json::from_str(&line).expect("Failed to parse line");
            parsed.data
        })
        .collect::<Vec<TestData>>();
    testData
}

fn get_git_commit_hash() -> String {
    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .expect("Failed to get git commit hash");
    
    String::from_utf8(output.stdout)
        .expect("Invalid UTF-8")
        .trim()
        .to_string()
}

#[derive(Debug)]
struct TestResults {
    dim: String,
    num_samples: usize,
    max_us: u32,
    min_us: u32,
    mean_us: f64,
    std_us: f64
}

impl TestResults {
    fn from_times(dim: &str, num_samples: usize, times: &Vec<u128>) -> TestResults {
        let times_f = times.iter().map(|&v| v as f64).collect::<Vec<_>>();

        TestResults {
            dim: dim.to_string(),
            num_samples: num_samples,
            max_us: *max(times).unwrap() as u32,
            min_us: *min(times).unwrap() as u32,
            mean_us: mean(&times_f),
            std_us: standard_deviation(&times_f, None)
        }
    }

    fn save(results: &[Self]) {
        let commit_hash = get_git_commit_hash();
        let seconds_since_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time should go forward")
            .as_secs();
        let filename = format!("benches/results/bench_results_{}.txt", seconds_since_epoch);
        
        let mut file = File::options()
            .create(true)
            .append(true)
            .open(&filename)
            .expect("Failed to create benchmark results file");
        
        writeln!(file, "Commit: {}", commit_hash).unwrap();
        writeln!(file, "---").unwrap();
        for res in results {
            writeln!(file, "Result: {}", res).unwrap();
        } 
        
        println!("Saved results to {}", filename);
    }
}

fn print_time<T>(time_us: T) -> String
where
    T: Into<f64> + Copy,
{
    let time = time_us.into();
    
    if time >= 1_000_000.0 {
        format!("{:.3} s", time / 1_000_000.0)
    } else if time >= 1_000.0 {
        format!("{:.3} ms", time / 1_000.0)
    } else {
        format!("{:.3} μs", time)
    }
}

impl std::fmt::Display for TestResults {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let min = print_time(self.min_us);
        let max = print_time(self.max_us);
        let mean = print_time(self.mean_us);
        let std = print_time(self.std_us);
        
        write!(f, "TestResults ({} and {} samples). Mean: {}, Min: {}, Max: {}, Std: {}", self.dim, self.num_samples, mean, min, max, std)
    }
}

fn run_performance_test(dimensions: &str, test_data: &Vec<TestData>) -> TestResults {
    let mut execution_time = Vec::new();
    for data in test_data {
        let constraints_x = TestData::hints_to_str(&data.hints_x);
        let constraints_y = TestData::hints_to_str(&data.hints_y);

        let now = Instant::now();
        solve(&constraints_x, &constraints_y, dimensions);
        let elapsed = now.elapsed();
        execution_time.push(elapsed.as_micros());
    }

    TestResults::from_times(dimensions, test_data.len(), &execution_time)
}

fn main() {
    // TODO: There is an issue with non-square boards.
    let dimensions = vec![
        ("5x5", 0), 
        ("10x10", 0),
        // ("15x10", -1),
        ("15x15", 0),
        ("20x20", 20),
        ("25x25", 20),
        ("30x30", 20),
        ];

    let mut results = Vec::new(); 
    for (dim, num_samples) in dimensions {
        let mut test_data = read_test_data(&format!("data/{}.json", dim));
        if num_samples != 0 {
            test_data = test_data[..num_samples].to_vec();
        }

        let res = run_performance_test(dim, &test_data);
        println!("{}", &res);
        results.push(res);
    }

    if std::env::var("SAVE_BENCH").is_ok() {
        TestResults::save(&results);
    }
}