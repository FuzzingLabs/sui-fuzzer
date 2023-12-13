use itertools::Itertools;

use crate::fuzzer::{coverage::Coverage, error::Error};

use super::detector::Detector;

#[derive(Default)]
pub struct BasicOpCodeDetector {}

impl Detector for BasicOpCodeDetector {
    fn detect(&self, coverage: &Coverage, _error: Option<Error>) -> (bool, Option<String>) {
        let counts = coverage.data.clone().into_iter().map(|d| d.pc).counts();

        let sorted: Vec<(&u64, &usize)> = counts.iter().sorted_by(|a, b| b.1.cmp(&a.1)).collect();
        let sum: usize = sorted[0..sorted.len()/4].iter().map(|a| a.1).sum();
        let count = coverage.data.clone().into_iter().count();
        if sum as f64 / count as f64 > 0.8 {
            return (
                true,
                Some(
                    format!(
                        "A few instructions are called more 80% more than the others, which means that there probably is recursivity or a loop in the contract.",
                    )
                    .to_string(),
                ),
            );
        }

        (false, None)
    }

    fn get_type(&self) -> super::detector::AvailableDetector {
        super::detector::AvailableDetector::BasicOpCodeDetector
    }
}
