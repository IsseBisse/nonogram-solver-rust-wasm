use std::fmt;
use std::ops::{BitAnd, BitOr};
use std::iter::zip;
use itertools::Itertools;

pub struct Dimensions {
    num_cols: usize,
    num_rows: usize,
}

impl Dimensions {
    pub fn new(num_rows: usize, num_cols: usize) -> Self {
        Dimensions { num_cols, num_rows }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CellState {
    Full,
    Empty,
    Unknown,
    Invalid
}

impl fmt::Display for CellState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CellState::Full => write!(f, "█"),
            CellState::Empty => write!(f, "░"),
            CellState::Unknown => write!(f, "-"),
            CellState::Invalid => write!(f, "x")
        }
    }
}

impl BitAnd for CellState {
    type Output = CellState;

    fn bitand(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (CellState::Full, CellState::Full) => CellState::Full,
            (CellState::Empty, CellState::Empty) => CellState::Empty,
            _ => CellState::Unknown, // Default case
        }
    }
}

impl BitOr for CellState {
    type Output = CellState;

    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (CellState::Full, CellState::Empty) | (CellState::Empty, CellState::Full) => CellState::Invalid, // We can't combine two differing states
            (_, CellState::Invalid) | (CellState::Invalid, _) => CellState::Invalid,
            (_, CellState::Empty) | (CellState::Empty, _) => CellState::Empty,
            (_, CellState::Full) | (CellState::Full, _) => CellState::Full,
            _ => CellState::Unknown
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Constraint {
    values: Vec<usize>
}

impl Constraint {
    pub fn new(values: Vec<usize>) -> Self {
        Constraint { values }
    }

    fn filter(&self, candidates: &[Line]) -> Vec<Line> {
        return candidates
            .iter()
            .filter(|line| line.to_constraint() == *self)
            .cloned()
            .collect::<Vec<Line>>();
    }
}

pub struct Constraints {
    cols: Vec<Constraint>,
    rows: Vec<Constraint>
}

impl Constraints {
    pub fn new(cols: Vec<Constraint>, rows: Vec<Constraint>) -> Self {
        Constraints { cols, rows }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    cells: Vec<CellState>
}

impl Line {
    fn empty(length: usize) -> Self {
        Line {
            cells: vec![CellState::Empty; length]
        }
    }
    
    fn unknown(length: usize) -> Self {
        Line {
            cells: vec![CellState::Unknown; length]
        }
    }

    fn new(cells: Vec<CellState>) -> Self {
        Line { cells }
    }

    fn to_constraint(&self) -> Constraint {
        let values = self.cells
            .iter()
            .chunk_by(|&cell| *cell)
            .into_iter()
            .filter_map(|(key, group)| {
                if key == CellState::Full {
                    Some(group.count())
                } else {
                    None
                }
            })
            .collect();

        Constraint::new(values)
    }

    fn equivalient(&self, rhs: &Self) -> bool {
        zip(&self.cells, &rhs.cells)
        .all(|(first, second)| {
            match (first, second) {
                (CellState::Unknown, _) | (_, CellState::Unknown) => true,
                _ => first == second
            }
        })
    }

    fn filter(&self, candidates: &[Self]) -> Vec<Self> {
        candidates
            .iter()
            .filter(|line| self.equivalient(line))
            .cloned()
            .collect::<Vec<Line>>()
    }

    fn sum(lines: &[Self]) -> Option<Self> {
        if lines.is_empty() {
            return None
        }

        // TODO: Re-write this using function chaining
        let mut new_line = lines[0].clone();
        for line in &lines[1..] {
            new_line = &new_line & line
        }

        return Some(new_line)
    }

    fn generate_combinations(blocks: &[Vec<CellState>], free_empty_spaces: usize) -> Vec<Self> {
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
                    result.push(vec![CellState::Empty]);
                }
                // Add the block
                result.push(block.clone());
            }
            
            // Add trailing empty spaces
            for _ in 0..counts[n_blocks] {
                result.push(vec![CellState::Empty]);
            }

            let line = Line::new(result
                .into_iter()
                .flatten()
                .collect::<Vec<CellState>>());
            
            results.push(line);
        }
        
        results
    }

    fn generate_initial_candidates(length: usize, constraint: &Constraint) -> Vec<Self> {
        if constraint.values.len() == 0 {
            return vec![Line::empty(length)]
        }

        let mut blocks = constraint.values
            .iter()
            .map(|value| {
                let mut block = vec![CellState::Full; *value];
                block.push(CellState::Empty);
                block
            })
            .collect::<Vec<Vec<CellState>>>();

        let last_idx = blocks.len() - 1;
        let last_item_idx = blocks[last_idx].len() - 1;
        blocks[last_idx] = blocks[last_idx][..last_item_idx].to_vec();

        let block_occupied_spaces: usize = blocks
            .iter()
            .map(|block| block.len())
            .sum();
        let free_empty_spaces = length - block_occupied_spaces;

        Line::generate_combinations(&blocks, free_empty_spaces)
    }

}


impl fmt::Display for &Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.cells.iter().join(""))
    }
}

impl BitAnd for &Line {
    type Output = Line;

    fn bitand(self, rhs: Self) -> Self::Output {
        let cells = zip(&self.cells, &rhs.cells)
        .map(|(&first, &second)| {
            first & second
        })
        .collect::<Vec<CellState>>();
        Line::new(cells)
    }
}

impl BitOr for &Line {
    type Output = Line;

    fn bitor(self, rhs: Self) -> Self::Output {
        let cells = zip(&self.cells, &rhs.cells)
        .map(|(&first, &second)| {
            first | second
        })
        .collect::<Vec<CellState>>();
        Line::new(cells)
    }
}
 
pub struct Board {
    dimensions: Dimensions,
    cells: Vec<CellState>,
    row_constraints: Vec<Constraint>,
    col_constraints: Vec<Constraint>,
    row_candidates: Vec<Vec<Line>>,
    col_candidates: Vec<Vec<Line>>
}

impl Board {
    pub fn new(constraints: Constraints, dimensions: Dimensions) -> Self {
        let length = dimensions.num_cols;
        
        let row_candidates = constraints.rows
            .iter()
            .map(|constraint| Line::generate_initial_candidates(length, constraint))
            .collect::<Vec<Vec<Line>>>();
        let col_candidates = constraints.cols
            .iter()
            .map(|constraint| Line::generate_initial_candidates(length, constraint))
            .collect::<Vec<Vec<Line>>>();

        let cells = vec![CellState::Unknown; dimensions.num_cols * dimensions.num_rows];

        Board { 
            dimensions: dimensions, 
            cells: cells,
            row_constraints: constraints.rows, 
            col_constraints: constraints.cols, 
            row_candidates: row_candidates, 
            col_candidates: col_candidates
        }
    }

    pub fn solve(&mut self) -> String {
        let mut prev_num_unknown = self.num_unknown();
        
        let mut solve_rows = true;
        let mut state_string = String::new();
        while !self.is_solved() {
            self.update_candidates(solve_rows);
            self.update_cells(solve_rows);

            if self.num_unknown() == prev_num_unknown {
                // Board has multiple solutions
                break
            }

            // state_string.push_str(&self.to_string());

            solve_rows = !solve_rows;
            prev_num_unknown = self.num_unknown();

            println!("{}", self.to_string());
        }

        state_string.push_str(&self.to_string());
        state_string
    }

    fn to_line(&self, idx: usize, is_row: bool) -> Line {
        if is_row {
            let start = idx * self.dimensions.num_cols;
            let end = start + self.dimensions.num_cols;
            let cells = self.cells[start..end].to_vec();
            return Line::new(cells)
        } else {
            let cells = (0..self.dimensions.num_rows)
                .map(|row_idx| self.cells[row_idx * self.dimensions.num_cols + idx])
                .collect();
            Line::new(cells)
        }
    }

    fn or_line(&mut self, idx: usize, is_row: bool, line: &Line) {
        if is_row {
            let start = idx * self.dimensions.num_cols;
            let end = start + self.dimensions.num_cols;
            self.cells[start..end].copy_from_slice(&line.cells[..]);
        } else {
            for (row_idx, cell) in line.cells.iter().enumerate() {
                self.cells[row_idx * self.dimensions.num_cols + idx] = self.cells[row_idx * self.dimensions.num_cols + idx] | *cell
            }
        }
    }

    pub fn to_string(&self) -> String {
        let mut s = String::new();

        for row in self.cells.chunks(self.dimensions.num_cols) {
            for cell in row {
                s.push_str(&cell.to_string())
            }
            s.push('\n')
        }
        s.push('\n');
        s
    }

    fn update_candidates(&mut self, is_row: bool) {
        // TODO: Re-write this to re-use code for rows and cols
        if is_row {
            let rows = (0..self.row_candidates.len())
                .map(|idx| self.to_line(idx, is_row))
                .collect::<Vec<Line>>();

            self.row_candidates
                .iter_mut()
                .zip(rows.iter())
                .for_each(|(candidates, row)| {
                    candidates.retain(|line| {
                        line.equivalient(row)
                    })
                });
        } else {
            let cols = (0..self.col_candidates.len())
                .map(|idx| self.to_line(idx, is_row))
                .collect::<Vec<Line>>();

            self.col_candidates
                .iter_mut()
                .zip(cols.iter())
                .for_each(|(candidates, col)| {
                    candidates.retain(|line| {
                        line.equivalient(col)
                    })
                });
        }
    }
        
    fn update_cells(&mut self, is_row: bool) {
        // TODO: Re-write this to re-use code for rows and cols
        let length = if is_row {
            self.dimensions.num_cols
        } else {
            self.dimensions.num_rows
        };

        let line_candidates = if is_row {
            &self.row_candidates
        } else {
            &self.col_candidates
        };
         
        let summed_lines = line_candidates
            .iter()
            .map(|candidates| {
                match Line::sum(candidates) {
                    Some(line) => line,
                    None => Line::empty(length)
                }
            })
            .collect::<Vec<Line>>();

        for (idx, line) in summed_lines.iter().enumerate() {
            self.or_line(idx, is_row, line);
        }
    }

    fn num_unknown(&self) -> usize {
        self.cells
            .iter()
            .filter(|&&cell| cell == CellState::Unknown)
            .count()
    }

    fn is_solved(&self) -> bool {
        self.num_unknown() == 0
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    mod cell_state {
        use super::*;

        #[test]
        fn test_print() {
            let full = CellState::Full;
            let empty = CellState::Empty;
            let unknown = CellState::Unknown;

            println!("{} {} {}", &full, &empty, &unknown);
        }

        #[test]
        fn test_and() {
            let full = CellState::Full;
            let empty = CellState::Empty;
            let unknown = CellState::Unknown;

            assert_eq!(full & unknown, unknown);
            assert_eq!(empty & unknown, unknown);
            assert_eq!(full & empty, unknown);
            assert_eq!(full & full, full);
            assert_eq!(empty & empty, empty);
        }
    
        #[test]
        fn test_or() {
            let full = CellState::Full;
            let empty = CellState::Empty;
            let unknown = CellState::Unknown;
            let invalid = CellState::Invalid;

            assert_eq!(full | unknown, full);
            assert_eq!(full | full, full);
            assert_eq!(empty | unknown, empty);
            assert_eq!(empty | empty, empty);
            assert_eq!(full | empty, invalid);
        }
    }

    mod constraint {
        use super::*;

        #[test]
        fn test_filter() {
            let values = vec![1, 2];
            let constraint = Constraint::new(values);
            
            let ok_line = Line::new(vec![CellState::Full, CellState::Empty, CellState::Full, CellState::Full]);
            let nok_line = Line::new(vec![CellState::Full, CellState::Full, CellState::Empty, CellState::Full]);
            let lines = vec![ok_line.clone(), nok_line.clone()];

            let filtered_lines = constraint.filter(&lines);

            assert_eq!(filtered_lines.len(), 1);
            assert!(filtered_lines.contains(&ok_line));
            assert!(!filtered_lines.contains(&nok_line))
        }
    }

    mod line {
        use super::*;

        #[test]
        fn test_to_constraint() {
            let line = Line::new(vec![CellState::Full, CellState::Empty, CellState::Full, CellState::Full]);
            let constraint = Constraint::new(vec![1, 2]);

            assert_eq!(line.to_constraint(), constraint)
        }

        #[test]
        fn test_equivalent() {
            let line = Line::new(vec![CellState::Full, CellState::Empty, CellState::Full, CellState::Full]);
            let equiv_line= Line::new(vec![CellState::Full, CellState::Empty, CellState::Unknown, CellState::Unknown]);
            let nequiv_line = Line::new(vec![CellState::Unknown, CellState::Full, CellState::Unknown, CellState::Unknown]);

            assert!(line.equivalient(&equiv_line));
            assert!(!line.equivalient(&nequiv_line));
        }

        #[test]
        fn test_filter() {
            let line = Line::new(vec![CellState::Full, CellState::Empty, CellState::Full, CellState::Full]);
            let equiv_line= Line::new(vec![CellState::Full, CellState::Empty, CellState::Unknown, CellState::Unknown]);
            let nequiv_line = Line::new(vec![CellState::Unknown, CellState::Full, CellState::Unknown, CellState::Unknown]);            
            let line_candidates = vec![line.clone(), equiv_line.clone(), nequiv_line.clone()];

            let filtered_lines = line.filter(&line_candidates);

            assert_eq!(filtered_lines.len(), 2);
            assert!(filtered_lines.contains(&line));
            assert!(filtered_lines.contains(&equiv_line));
            assert!(!filtered_lines.contains(&nequiv_line));
        }

        #[test]
        fn test_print() {
            let line = Line::new(vec![CellState::Full, CellState::Empty, CellState::Full, CellState::Full]);
            println!("{}", &line)
        }

        #[test]
        fn test_and() {
            let a = Line::new(vec![CellState::Empty, CellState::Empty, CellState::Empty, CellState::Full, CellState::Full, CellState::Full]);
            let b = Line::new(vec![CellState::Empty, CellState::Full, CellState::Unknown, CellState::Empty, CellState::Full, CellState::Unknown]);
            let a_and_b = Line::new(vec![CellState::Empty, CellState::Unknown, CellState::Unknown, CellState::Unknown, CellState::Full, CellState::Unknown]);

            let res = &a & &b;
            assert_eq!(res, a_and_b)    
        }
        
        #[test]
        fn test_or() {
            let a = Line::new(vec![CellState::Empty, CellState::Empty, CellState::Empty, CellState::Full, CellState::Full, CellState::Full]);
            let b = Line::new(vec![CellState::Empty, CellState::Full, CellState::Unknown, CellState::Empty, CellState::Full, CellState::Unknown]);
            let a_or_b = Line::new(vec![CellState::Empty, CellState::Invalid, CellState::Empty, CellState::Invalid, CellState::Full, CellState::Full]);

            let res = &a | &b;
            assert_eq!(res, a_or_b)    
        }

        #[test]
        fn test_sum() {
            let a = Line::new(vec![CellState::Empty, CellState::Empty, CellState::Empty, CellState::Full, CellState::Full, CellState::Full]);
            let b = Line::new(vec![CellState::Empty, CellState::Full, CellState::Unknown, CellState::Empty, CellState::Full, CellState::Unknown]);
            let a_and_b = Line::new(vec![CellState::Empty, CellState::Unknown, CellState::Unknown, CellState::Unknown, CellState::Full, CellState::Unknown]);

            let res = Line::sum(&vec![a, b]).unwrap();
            assert_eq!(res, a_and_b)    
        }
    }

    mod Board {
        use crate::model::Board;

        use super::*;

        #[test]
        fn test_is_solved() {
            let dimensions = Dimensions::new(2, 4);
            
            let row_constraints = vec![
                Constraint::new(vec![1, 2]),
                Constraint::new(vec![1, 2])
            ];
            let col_constraints = vec![
                Constraint::new(vec![2]),
                Constraint::new(vec![]),
                Constraint::new(vec![2]),
                Constraint::new(vec![2])
            ];
            let constraints = Constraints::new(col_constraints, row_constraints);
            
            let board = Board::new(constraints, dimensions);

            // TODO: Implement this
        }
    }
}