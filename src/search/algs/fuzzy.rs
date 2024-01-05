#![allow(
    clippy::separated_literal_suffix,
    clippy::arithmetic_side_effects,
)]

//!

use core::cmp;

use std::sync::Mutex;

use lazy_static::lazy_static;

use super::Algorithm;

lazy_static! {
    static ref OPT: Mutex<Vec<i16>> = Mutex::new(vec![0_i16; 102_400]);
}

///
pub struct Fuzzy {
    ///
    eq: i16,

    ///
    target_gap: i16,

    ///
    target_gap_extend: i16,
}

impl Default for Fuzzy {
    fn default() -> Self {
        Self {
            eq: 16,
            target_gap: -3,
            target_gap_extend: -1,
        }
    }
}

/// Makes sure that all characters in query
/// exist in the query string in the correct order.
fn validate(target: &[char], query: &[char]) -> bool {
    let mut query_ptr = 0;
    for ch in target {
        if let Some(query_ch) = query.get(query_ptr) {
            if (*ch).to_ascii_lowercase() == query_ch.to_ascii_lowercase() {
                query_ptr = query_ptr.saturating_add(1);
            }
        }
    }

    query_ptr == query.len()
}

impl Algorithm for Fuzzy {
    fn score(&self, target: &str, query: &str) -> i16 {
        if query.len() == 1 {
            if target.contains(query) {
                return self.eq;
            }

            return 0_i16;
        }

        let Ok(mut opt) = OPT.lock() else {
            return 0_i16;
        };

        let target: Vec<char> = target.chars().collect();
        let query: Vec<char> = query.chars().collect();

        let Some(width) = target.len().checked_add(1) else {
            return 0_i16;
        };

        let Some(height) = query.len().checked_add(1) else {
            return 0_i16;
        };

        if !validate(&target, &query) {
            return 0_i16;
        }

        // Reset the "first row" to zero
        for space in opt.iter_mut().take(width) {
            *space = 0_i16;
        }

        let mut max_score = 0_i16;
        for row in 1..height {
            let mut in_gap = false;
            let Some(start) = opt.get_mut(row * width) else {
                return 0_i16;
            };
            *start = 0_i16;

            for col in 1..width {
                let Some(left) = opt.get(row * width + col - 1) else {
                    return 0_i16;
                };
                let Some(diag) = opt.get((row - 1) * width + col - 1) else {
                    return 0_i16;
                };

                let gap_score = if in_gap {
                    cmp::max(left + self.target_gap_extend, 0)
                } else {
                    cmp::max(left + self.target_gap, 0)
                };

                let Some(target_ch) = target.get(col - 1).map(char::to_ascii_lowercase) else {
                    return 0_i16;
                };
                let Some(query_ch) = query.get(row - 1).map(char::to_ascii_lowercase) else {
                    return 0_i16;
                };

                let match_score = if target_ch == query_ch {
                    diag + self.eq
                } else {
                    0_i16
                };

                in_gap = match_score < gap_score;

                let score = cmp::max(match_score, gap_score);
                let Some(score_location) = opt.get_mut(row * width + col) else {
                    return 0_i16;
                };
                *score_location = score;

                if (height - 1) == row && score > max_score {
                    max_score = score;
                }
            }
        }

        max_score
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_missing_chars_invalid() {
        let alg = Fuzzy::default();
        assert_eq!(0, alg.score("Hello", "Halo"));
    }

    #[test]
    fn test_missing_wrong_order_invalid() {
        let alg = Fuzzy::default();

        assert_eq!(0, alg.score("Hello", "elH"));
    }

    #[test]
    fn test_all_match_score() {
        let alg = Fuzzy::default();

        assert_eq!(80, alg.score("Hello", "Hello"));
    }

    #[test]
    fn test_partial_match_front() {
        let alg = Fuzzy::default();

        assert_eq!(32, alg.score("Hello", "He"));
    }

    #[test]
    fn test_partial_match_end() {
        let alg = Fuzzy::default();

        assert_eq!(48, alg.score("Hello", "llo"));
    }

    #[test]
    fn test_partial_match_gap() {
        let alg = Fuzzy::default();

        assert_eq!(44, alg.score("Hello", "Hlo"));
    }
}
