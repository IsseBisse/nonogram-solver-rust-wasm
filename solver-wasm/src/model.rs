use std::u32;
use std::fmt;

use itertools::{Itertools, izip};

#[derive(Debug, Clone)]
pub struct Constraints {
    num_cols: usize,
    num_rows: usize,
    row_constraints: Vec<Vec<usize>>,
    col_constraints: Vec<Vec<usize>>,
}

fn parse_dim_string(s: &str) -> (usize, usize) {
    let parts = s.split("x").collect::<Vec<&str>>();

    let num_cols = parts[0].parse().unwrap();
    let num_rows = parts[1].parse().unwrap();
    (num_rows, num_cols)
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

impl Constraints {
    pub fn parse(constraints_x_str: &str, constraints_y_str: &str, dimensions: &str) -> Self {
        let row_constraints = parse_array_string(constraints_y_str);
        let col_constraints = parse_array_string(constraints_x_str);

        let (num_rows, num_cols) = parse_dim_string(dimensions);

        Constraints { num_cols, num_rows, row_constraints, col_constraints }
    }
}

#[derive(Debug, Clone)]
struct LineCandidate {
    filled: u32,
}

impl LineCandidate {
    fn is_compatible(&self, filled: u32, known: u32) -> bool {
        let known_filled = filled & known;
        let known_empty = !filled & known;

        if (known_filled & !self.filled) != 0 {
            return false
        }

        if (known_empty & self.filled) != 0 {
            return false
        }

        true
    }

    fn sum(candidates: &Vec<LineCandidate>) -> (u32, u32) {
        let mut all_empty = u32::MAX;
        let mut all_filled = u32::MAX;

        for cand in candidates {
            all_empty &= !cand.filled;
            all_filled &= cand.filled;
        }

        let known = all_empty | all_filled;
        let filled = all_filled;

        (filled, known)
    }

    // TODO: Re-write this old generation implementation
    fn generate_combinations(blocks: &[Vec<u8>], free_empty_spaces: usize) -> Vec<Self> {
        // NOTE: Claude's translation of python code
        let n_blocks = blocks.len();
        let n_positions = n_blocks + 1;
        
        let mut results = Vec::new();
        
        // Generate all combinations of indices
        let total_range = free_empty_spaces + n_positions - 1;
        
        for indices in (0..total_range).combinations(n_positions - 1) {
            // Convert combination indices to counts per position
            let mut counts = Vec::with_capacity(n_positions);
            let mut prev = -1_i32;
            
            for &idx in &indices {
                counts.push((idx as i32 - prev - 1) as usize);
                prev = idx as i32;
            }
            counts.push((total_range as i32 - prev - 1) as usize);
            
            // Build the result list
            let mut result = Vec::new();
            
            for (i, block) in blocks.iter().enumerate() {
                // Add empty spaces before this block
                for _ in 0..counts[i] {
                    result.push(vec![0u8]);
                }
                // Add the block
                result.push(block.clone());
            }
            
            // Add trailing empty spaces
            for _ in 0..counts[n_blocks] {
                result.push(vec![0u8]);
            }

            let mut filled = 0u32;
            for (idx, bit) in result.iter().flatten().rev().enumerate() {
                if *bit == 1 {
                    filled += 1 << idx
                }
            };

            let line = LineCandidate { filled };
            
            results.push(line);
        }
        
        results
    }

    fn generate_initial_candidates(length: usize, constraint: &Vec<usize>) -> Vec<Self> {
        let mut blocks = constraint
            .iter()
            .map(|value| {
                let mut block = vec![1u8; *value];
                block.push(0u8);
                block
            })
            .collect::<Vec<Vec<u8>>>();

        let last_idx = blocks.len() - 1;
        let last_item_idx = blocks[last_idx].len() - 1;
        blocks[last_idx] = blocks[last_idx][..last_item_idx].to_vec();

        let block_occupied_spaces: usize = blocks
            .iter()
            .map(|block| block.len())
            .sum();
        let free_empty_spaces = length - block_occupied_spaces;

        LineCandidate::generate_combinations(&blocks, free_empty_spaces)
    }
}

fn transpose_32x32(rows: &[u32; 32]) -> [u32; 32] {
    let mut cols = [0u32; 32];
    
    for row in 0..32 {
        for col in 0..32 {
            if (rows[row] & (1 << col)) != 0 {
                cols[col] |= 1 << row;
            }
        }
    }
    
    cols
}

#[derive(Debug, Clone)]
pub struct Board {
    constraints: Constraints,

    // We treat every board as 32x32 to be able to pre-allocate this
    rows_filled: [u32; 32],
    rows_known: [u32; 32],

    row_candidates: Vec<Vec<LineCandidate>>,
    col_candidates: Vec<Vec<LineCandidate>>,
}

fn board_to_string(filled: &[u32; 32], known: &[u32; 32]) -> String {
    let mut s = String::new();
    for (f, k) in filled.iter().zip(known) {
        s.push_str(&format!("{:032b}\n", f)); 
        s.push_str(&format!("{:032b}\n", k)); 
    };
    s
}

impl Board {
    pub fn new(constraints: Constraints) -> Self {
        let row_candidates = constraints.row_constraints
            .iter()
            .map(|c| LineCandidate::generate_initial_candidates(constraints.num_cols, c))
            .collect::<Vec<_>>();

        let col_candidates = constraints.col_constraints
            .iter()
            .map(|c| LineCandidate::generate_initial_candidates(constraints.num_rows, c))
            .collect::<Vec<_>>();

        Board { 
            constraints, 
            rows_filled: [0; 32], 
            rows_known: [0; 32],
            row_candidates,
            col_candidates,
        }
    }

    fn update(&mut self, rowwise: bool) {
        let candidates = if rowwise {
            &mut self.row_candidates
        } else {
            &mut self.col_candidates
        };

        let (lines_filled, lines_known) = if rowwise {
            (self.rows_filled, self.rows_known) 
        } else {
            (transpose_32x32(&self.rows_filled), transpose_32x32(&self.rows_known))
        };

        let lines_mask = if rowwise {
            (1u32 << self.constraints.num_cols) - 1
        } else {
            (1u32 << self.constraints.num_rows) - 1
        };

        println!("Before");
        println!("{}", board_to_string(&lines_filled, &lines_known));

        // Update candidates
        let mut new_filled = [0 as u32; 32];
        let mut new_known = [0 as u32; 32];
        for (idx, cand) in candidates.iter_mut().enumerate() {
            let f = lines_filled[idx];
            let k = lines_known[idx];
            cand.retain(|c| c.is_compatible(f, k));

            // Update rows
            let (new_f, new_k) = LineCandidate::sum(&cand);

            // TODO: 
            new_filled[idx] = new_f & lines_mask;
            new_known[idx] = new_k & lines_mask;
        }

        println!("After");
        println!("{}", board_to_string(&new_filled, &new_known));

        if rowwise {
            self.rows_filled = new_filled;
            self.rows_known = new_known;

        } else {
            self.rows_filled = transpose_32x32(&new_filled);
            self.rows_known = transpose_32x32(&new_known)
        }
    }

    fn num_unsolved(&self) -> usize {
        let total_squares = self.constraints.num_rows * self.constraints.num_cols;
        let set_squares = self.rows_known
            .iter()
            .map(|row| row.count_ones() as usize)
            .sum::<usize>();

        println!("{}", board_to_string(&self.rows_known, &self.rows_known));

        total_squares - set_squares
    }

    fn is_solved(&self) -> bool {
        return self.num_unsolved() == 0
    }

    // fn generate_guesses(&self) -> Vec<Board> {

    // }

    pub fn solve(mut self) -> Vec<Board> {
        let mut num_unsolved = self.num_unsolved();
        while !self.is_solved() {
            println!("Rowwise");
            self.update(true);
            println!("Colwise");
            self.update(false);

            let new_num_unsolved = self.num_unsolved();
            if new_num_unsolved == num_unsolved {
                // We can't get further and need to guess
                break
            } else {
                num_unsolved = new_num_unsolved
            }
        }

        // guessed_boards = generate_guesses(&board);
        // let solutions = guessed_board
        //     .iter()
        //     .flat_map(|b| solve(b, constraints))
        //     .collect();

        vec![self.clone()]
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    mod constraints {
        use super::*;

        #[test]
        fn test_parse() {
            let constraints_x_str = "2;1;2;1";
            let constraints_y_str = "1,2;3";
            let dimensions = "4x2";

            let constraints = Constraints::parse(constraints_x_str, constraints_y_str, dimensions);

            assert_eq!(constraints.num_cols, 4);
            assert_eq!(constraints.num_rows, 2);

            assert_eq!(
                constraints.row_constraints, 
                vec![
                    vec![1, 2], 
                    vec![3]
                    ]);

            assert_eq!(
                constraints.col_constraints, 
                vec![
                    vec![2],
                    vec![1],
                    vec![2],
                    vec![1],
                    ]);
        }
    }
    
    mod line_candidate {
        use super::*;

        #[test]
        fn test_is_compatible() {
            let filled = 0x0000FFFFu32;
            let line = LineCandidate { filled };

            let incompatible_filled = 0xF0F0F0F0u32;
            let compatible_filled = 0x0000FFFFu32;

            let strict_known = 0xFFFFFFFFu32;
            let permissive_known = 0x0F0FF0F0u32;

            assert_eq!(line.is_compatible(compatible_filled, permissive_known), true);
            assert_eq!(line.is_compatible(compatible_filled, strict_known), true);
            assert_eq!(line.is_compatible(incompatible_filled, permissive_known), true);
            assert_eq!(line.is_compatible(incompatible_filled, strict_known), false);
        }

        #[test]
        fn test_sum() {
            let first_filled = 0x0000FFFFu32;
            let first_line = LineCandidate { filled: first_filled };
            
            let second_filled = 0xFFFF0000u32;
            let second_line = LineCandidate { filled: second_filled };
            
            let third_filled = 0x0F0F0F0Fu32;
            let third_line = LineCandidate { filled: third_filled };

            let (fs_filled, fs_known) = LineCandidate::sum(&vec![first_line.clone(), second_line]);
            assert_eq!(fs_filled, 0u32);
            assert_eq!(fs_known, 0u32);

            let (ft_filled, ft_known) = LineCandidate::sum(&vec![first_line, third_line]);
            assert_eq!(ft_filled, 0x00000F0Fu32);
            assert_eq!(ft_known, 0xF0F00F0Fu32);
        }

        #[test]
        fn test_initial_candidate() {
            let constraints_x_str = "2;1;2;1";
            let constraints_y_str = "1,2;3";
            let dimensions = "4x2";

            let constraints = Constraints::parse(constraints_x_str, constraints_y_str, dimensions);
            let board = Board::new(constraints);

            assert_eq!(board.row_candidates[0].len(), 1);
            assert_eq!(board.row_candidates[1].len(), 2);

            assert_eq!(board.col_candidates[0].len(), 1);
            assert_eq!(board.col_candidates[1].len(), 2);
            assert_eq!(board.col_candidates[2].len(), 1);
            assert_eq!(board.col_candidates[3].len(), 2);
        }
    }

    mod board {
        use super::*;

        #[test]
        fn test_update() {

        }

        fn test_num_unsolved() {

        }

        fn test_is_solved() {

        }

        fn test_solve() {

        }
    }
}