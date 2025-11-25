use std::fmt;
use std::ops::{BitAnd, BitOr};
use std::iter::zip;
use itertools::Itertools;

pub struct Dimensions {
    num_cols: usize,
    num_rows: usize,
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
    fn new(values: Vec<usize>) -> Self {
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

#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    cells: Vec<CellState>
}

impl Line {
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

    // fn sum(lines: &[Self]) -> Option<Self> {
    //     let new_line = lines
    //         .iter()
    //         .reduce(|a, b| a & b);
    //     new_line.cloned()
    // }
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
    }
}