#![allow(clippy::separated_literal_suffix)]

//!

use std::{collections::HashMap, cmp, sync::Mutex};

use json_pointer::JsonPointer;
use lazy_static::lazy_static;

lazy_static!{ 
    static ref OPT: Mutex<Vec<i16>> = Mutex::new(vec![0_i16; 102_400]);
}

///
pub struct ScoringCriteria {
    ///
    eq: i16,

    ///
    target_gap: i16,

    ///
    target_gap_extend: i16,
}

impl Default for ScoringCriteria {
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
        if query_ptr < query.len() && (*ch).to_ascii_lowercase() == query[query_ptr].to_ascii_lowercase() {
            query_ptr += 1;
        }
    }

    query_ptr == query.len()
}

///
fn score(target: &str, query: &str, criteria: &ScoringCriteria, opt: &mut [i16]) -> i16 {
    let target: Vec<char> = target.chars().collect();
    let query: Vec<char> = query.chars().collect();

    let width = target.len() + 1;
    let height = query.len() + 1;

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
        opt[row * width] = 0_i16;

        for col in 1..width {
            let left = opt[row * width + col - 1];
            let diag = opt[(row - 1) * width + col - 1];

            let gap_score = if in_gap {
                cmp::max(left + criteria.target_gap_extend, 0)
            } else {
                cmp::max(left + criteria.target_gap, 0)
            };

            let target_ch = target[col - 1].to_ascii_lowercase();
            let query_ch = query[row - 1].to_ascii_lowercase();
            let match_score = if target_ch == query_ch {
                diag + criteria.eq
            } else {
                0_i16
            };

            in_gap = match_score < gap_score;

            let score = cmp::max(match_score, gap_score);
            opt[row * width + col] = score;

            if (height - 1) == row && score > max_score {
                max_score = score;
            }
        }
    }

    max_score
}

///
pub fn filter(
    doc: &serde_json::Value, 
    graph: &HashMap<String, Vec<String>>, 
    value: &str
) -> Vec<String> {
    let mut opt = OPT.lock().unwrap();
    let criteria = ScoringCriteria::default();
    
    let mut scores: Vec<(String, i16)> = graph.iter()
        .filter_map(|(key, children)| {
            let mut key_score = score(key, value, &criteria, &mut opt);
            
            if children.is_empty() {
                let raw = key.parse::<JsonPointer<_, _>>().ok()
                    .and_then(|path| path.get(doc).ok())
                    .and_then(|node| serde_json::to_string_pretty(&node).ok());

                if let Some(raw) = raw {
                    key_score = cmp::max(score(&raw, value, &criteria, &mut opt), key_score);
                }
            }
            
            (key_score > 0).then(|| (key.to_string(), key_score))
        })
        .collect();

    scores.sort_by(|a, b| b.1.cmp(&a.1));

    scores.into_iter().map(|(key, _)| key).collect()
}

#[cfg(test)]
mod test {
    use super::*;
 
    #[test]
    fn test_missing_chars_invalid() {
        let opt = &mut [0_i16; 50];

        assert_eq!(0, score("Hello", "Halo", &ScoringCriteria::default(), opt));
    }

    #[test]
    fn test_missing_wrong_order_invalid() {
        let opt = &mut [0_i16; 50];

        assert_eq!(0, score("Hello", "elH", &ScoringCriteria::default(), opt));
    }

    #[test]
    fn test_all_match_score() {
        let opt = &mut [0_i16; 50];

        assert_eq!(80, score("Hello", "Hello", &ScoringCriteria::default(), opt));
    }
    
    #[test]
    fn test_partial_match_front() {
        let opt = &mut [0_i16; 50];

        assert_eq!(32, score("Hello", "He", &ScoringCriteria::default(), opt));
    }
    
    #[test]
    fn test_partial_match_end() {
        let opt = &mut [0_i16; 50];

        assert_eq!(48, score("Hello", "llo", &ScoringCriteria::default(), opt));
    }
    
    #[test]
    fn test_partial_match_gap() {
        let opt = &mut [0_i16; 50];

        assert_eq!(44, score("Hello", "Hlo", &ScoringCriteria::default(), opt));
    }
}

