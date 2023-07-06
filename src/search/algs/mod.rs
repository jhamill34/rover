#![allow(
    clippy::separated_literal_suffix,
    clippy::integer_arithmetic,
    clippy::arithmetic_side_effects,
    clippy::too_many_lines,

    // TODO: REMOVE!
    clippy::string_slice,
    clippy::indexing_slicing,
    clippy::redundant_else,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::as_conversions,
)]

//!

use core::cmp;
use std::collections::HashMap;

use crate::{pointer::ValuePointer, value::Value};

use super::{
    parser::{Operation, Pattern},
    PatternCache,
};

mod fuzzy;

///
pub trait Algorithm {
    ///
    fn score(&self, target: &str, query: &str) -> i16;
}

///
pub fn score(
    pattern: &Pattern,
    cache: &mut PatternCache,
    deref_cache: &mut PatternCache,
    doc: &Value,
) -> HashMap<String, i16> {
    match pattern {
        &Pattern::Group {
            ref op,
            ref left,
            ref right,
        } => {
            let left_score = score(left, cache, deref_cache, doc);
            let right_score = score(right, cache, deref_cache, doc);

            match op {
                &Operation::And => left_score
                    .iter()
                    .filter_map(|(key, left_score)| {
                        let right_score = right_score.get(key).unwrap_or(&0);

                        if left_score <= &0 || right_score <= &0 {
                            return None;
                        }

                        let score = left_score + right_score;
                        (score > 0).then_some((key.clone(), score))
                    })
                    .collect(),
                &Operation::Or => {
                    let mut result = left_score;
                    for (key, right_score) in right_score {
                        let left_score = result.entry(key).or_insert(0);
                        *left_score = cmp::max(*left_score, right_score);
                    }

                    result
                }
            }
        }
        &Pattern::Fuzzy(ref pattern) => {
            if let Some(existing) = cache.fuzzy.get(pattern) {
                return existing.clone();
            }

            let mut existing = &cache.core;
            for idx in 1..pattern.len() {
                let prefix = &pattern[..(pattern.len() - idx)];
                let suffix = &pattern[idx..];

                if let Some(prefix) = cache.fuzzy.get(prefix) {
                    existing = prefix;
                    break;
                } else if let Some(suffix) = cache.fuzzy.get(suffix) {
                    existing = suffix;
                    break;
                } else {
                }
            }

            let result: HashMap<String, i16> = existing
                .keys()
                .filter_map(|key| {
                    let fuzzy = fuzzy::Fuzzy::default();
                    let score = fuzzy.score(key, pattern);
                    (score > 0).then_some((key.clone(), score))
                })
                .collect();

            cache.fuzzy.set(pattern, result.clone());

            result
        }
        &Pattern::Exact(ref pattern) => {
            if let Some(existing) = cache.exact.get(pattern) {
                return existing.clone();
            }

            let mut existing = &cache.core;
            for idx in 1..pattern.len() {
                let prefix = &pattern[..(pattern.len() - idx)];
                let suffix = &pattern[idx..];

                if let Some(prefix) = cache.exact.get(prefix) {
                    existing = prefix;
                    break;
                } else if let Some(suffix) = cache.exact.get(suffix) {
                    existing = suffix;
                    break;
                } else {
                }
            }

            let result: HashMap<String, i16> = existing
                .keys()
                .filter_map(|x| {
                    let potential_score = (pattern.len() * 16) as i16;
                    x.contains(pattern).then_some((x.clone(), potential_score))
                })
                .collect();

            cache.exact.set(pattern, result.clone());

            result
        }
        &Pattern::Prefix(ref pattern) => {
            if let Some(existing) = cache.prefix.get(pattern) {
                return existing.clone();
            }

            let mut existing = &cache.core;
            for idx in 1..pattern.len() {
                let prefix = &pattern[..(pattern.len() - idx)];
                let suffix = &pattern[idx..];

                if let Some(prefix) = cache.prefix.get(prefix) {
                    existing = prefix;
                    break;
                } else if let Some(suffix) = cache.prefix.get(suffix) {
                    existing = suffix;
                    break;
                } else {
                }
            }

            let result: HashMap<String, i16> = existing
                .keys()
                .filter_map(|x| {
                    let potential_score = (pattern.len() * 16) as i16;
                    x.starts_with(pattern)
                        .then_some((x.clone(), potential_score))
                })
                .collect();

            cache.prefix.set(pattern, result.clone());

            result
        }
        &Pattern::Suffix(ref pattern) => {
            if let Some(existing) = cache.suffix.get(pattern) {
                return existing.clone();
            }

            let mut existing = &cache.core;
            for idx in 1..pattern.len() {
                let prefix = &pattern[..(pattern.len() - idx)];
                let suffix = &pattern[idx..];

                if let Some(suffix) = cache.prefix.get(prefix) {
                    existing = suffix;
                    break;
                } else if let Some(suffix) = cache.suffix.get(suffix) {
                    existing = suffix;
                    break;
                } else {
                }
            }

            let result: HashMap<String, i16> = existing
                .keys()
                .filter_map(|x| {
                    let potential_score = (pattern.len() * 16) as i16;
                    x.ends_with(pattern).then_some((x.clone(), potential_score))
                })
                .collect();

            cache.suffix.set(pattern, result.clone());

            result
        }
        &Pattern::Negated(ref pattern) => {
            let mut result = HashMap::new();
            let scored = score(pattern, cache, deref_cache, doc);
            let max_score = scored.values().max().unwrap_or(&0);

            for key in cache.core.keys() {
                if !scored.contains_key(key) {
                    result.insert(key.clone(), *max_score);
                }
            }

            result
        }
        &Pattern::Deref(ref pattern) => score_deref(pattern, deref_cache, doc),
    }
}

///
fn score_deref(
    pattern: &Pattern,
    deref_cache: &mut PatternCache,
    doc: &Value,
) -> HashMap<String, i16> {
    match pattern {
        &Pattern::Group {
            ref op,
            ref left,
            ref right,
        } => {
            let left_score = score_deref(left, deref_cache, doc);
            let right_score = score_deref(right, deref_cache, doc);

            match op {
                &Operation::And => left_score
                    .iter()
                    .filter_map(|(key, left_score)| {
                        let right_score = right_score.get(key).unwrap_or(&0);

                        if left_score <= &0 || right_score <= &0 {
                            return None;
                        }
                        let score = left_score + right_score;
                        (score > 0).then_some((key.clone(), score))
                    })
                    .collect(),
                &Operation::Or => {
                    let mut result = left_score;
                    for (key, right_score) in right_score {
                        let left_score = result.entry(key).or_insert(0);
                        *left_score = cmp::max(*left_score, right_score);
                    }

                    result
                }
            }
        }
        &Pattern::Fuzzy(ref pattern) => {
            if let Some(existing) = deref_cache.fuzzy.get(pattern) {
                return existing.clone();
            }

            let mut existing = &deref_cache.core;
            for idx in 1..pattern.len() {
                let prefix = &pattern[..(pattern.len() - idx)];
                let suffix = &pattern[idx..];

                if let Some(prefix) = deref_cache.fuzzy.get(prefix) {
                    existing = prefix;
                    break;
                } else if let Some(suffix) = deref_cache.fuzzy.get(suffix) {
                    existing = suffix;
                    break;
                } else {
                }
            }

            let result: HashMap<String, i16> = existing
                .keys()
                .filter_map(|key| {
                    let fuzzy = fuzzy::Fuzzy::default();

                    key.parse::<ValuePointer>()
                        .ok()
                        .and_then(|pointer| pointer.get(doc).ok())
                        .and_then(|node| match node {
                            &Value::String(ref value) => Some(fuzzy.score(value, pattern)),
                            &Value::Bool(value) => Some(fuzzy.score(&value.to_string(), pattern)),
                            &Value::Number(ref value) => {
                                Some(fuzzy.score(&value.to_string(), pattern))
                            }
                            &Value::Array(ref arr) => {
                                let mut score = 0;
                                for i in 0..arr.len() {
                                    score = cmp::max(score, fuzzy.score(&i.to_string(), pattern));
                                }

                                Some(score)
                            }
                            &Value::Object(ref obj) => {
                                let mut score = 0;
                                for key in obj.keys() {
                                    score = cmp::max(score, fuzzy.score(key, pattern));
                                }

                                Some(score)
                            }
                            &Value::Null => None,
                        })
                        .and_then(|score| (score > 0).then_some((key.clone(), score)))
                })
                .collect();

            deref_cache.fuzzy.set(pattern, result.clone());

            result
        }
        &Pattern::Exact(ref pattern) => {
            if let Some(existing) = deref_cache.exact.get(pattern) {
                return existing.clone();
            }

            let mut existing = &deref_cache.core;
            for idx in 1..pattern.len() {
                let prefix = &pattern[..(pattern.len() - idx)];
                let suffix = &pattern[idx..];

                if let Some(prefix) = deref_cache.exact.get(prefix) {
                    existing = prefix;
                    break;
                } else if let Some(suffix) = deref_cache.exact.get(suffix) {
                    existing = suffix;
                    break;
                } else {
                }
            }

            let result: HashMap<String, i16> = existing
                .keys()
                .filter_map(|key| {
                    key.parse::<ValuePointer>()
                        .ok()
                        .and_then(|pointer| pointer.get(doc).ok())
                        .and_then(|node| match node {
                            &Value::String(ref value) => {
                                let potential_score = (pattern.len() * 16) as i16;
                                value.contains(pattern).then_some(potential_score)
                            }
                            &Value::Bool(value) => {
                                let potential_score = (pattern.len() * 16) as i16;
                                value
                                    .to_string()
                                    .contains(pattern)
                                    .then_some(potential_score)
                            }
                            &Value::Number(ref value) => {
                                let potential_score = (pattern.len() * 16) as i16;
                                value
                                    .to_string()
                                    .contains(pattern)
                                    .then_some(potential_score)
                            }
                            &Value::Array(ref arr) => {
                                let mut score = 0;
                                for i in 0..arr.len() {
                                    let potential_score = (pattern.len() * 16) as i16;
                                    if i.to_string().contains(pattern) {
                                        score = cmp::max(score, potential_score);
                                    }
                                }

                                Some(score)
                            }
                            &Value::Object(ref obj) => {
                                let mut score = 0;
                                for key in obj.keys() {
                                    let potential_score = (pattern.len() * 16) as i16;
                                    if key.contains(pattern) {
                                        score = cmp::max(score, potential_score);
                                    }
                                }

                                Some(score)
                            }
                            &Value::Null => None,
                        })
                        .and_then(|score| (score > 0).then_some((key.clone(), score)))
                })
                .collect();

            deref_cache.exact.set(pattern, result.clone());

            result
        }
        &Pattern::Prefix(ref pattern) => {
            if let Some(existing) = deref_cache.prefix.get(pattern) {
                return existing.clone();
            }

            let mut existing = &deref_cache.core;
            for idx in 1..pattern.len() {
                let prefix = &pattern[..(pattern.len() - idx)];
                let suffix = &pattern[idx..];

                if let Some(prefix) = deref_cache.prefix.get(prefix) {
                    existing = prefix;
                    break;
                } else if let Some(suffix) = deref_cache.prefix.get(suffix) {
                    existing = suffix;
                    break;
                } else {
                }
            }

            let result: HashMap<String, i16> = existing
                .keys()
                .filter_map(|key| {
                    key.parse::<ValuePointer>()
                        .ok()
                        .and_then(|pointer| pointer.get(doc).ok())
                        .and_then(|node| match node {
                            &Value::String(ref value) => {
                                let potential_score = (pattern.len() * 16) as i16;
                                value.starts_with(pattern).then_some(potential_score)
                            }
                            &Value::Bool(value) => {
                                let potential_score = (pattern.len() * 16) as i16;
                                value
                                    .to_string()
                                    .starts_with(pattern)
                                    .then_some(potential_score)
                            }
                            &Value::Number(ref value) => {
                                let potential_score = (pattern.len() * 16) as i16;
                                value
                                    .to_string()
                                    .starts_with(pattern)
                                    .then_some(potential_score)
                            }
                            &Value::Array(ref arr) => {
                                let mut score = 0;
                                for i in 0..arr.len() {
                                    let potential_score = (pattern.len() * 16) as i16;
                                    if i.to_string().starts_with(pattern) {
                                        score = cmp::max(score, potential_score);
                                    }
                                }

                                Some(score)
                            }
                            &Value::Object(ref obj) => {
                                let mut score = 0;
                                for key in obj.keys() {
                                    let potential_score = (pattern.len() * 16) as i16;
                                    if key.starts_with(pattern) {
                                        score = cmp::max(score, potential_score);
                                    }
                                }

                                Some(score)
                            }
                            &Value::Null => None,
                        })
                        .and_then(|score| (score > 0).then_some((key.clone(), score)))
                })
                .collect();

            deref_cache.prefix.set(pattern, result.clone());

            result
        }
        &Pattern::Suffix(ref pattern) => {
            if let Some(existing) = deref_cache.suffix.get(pattern) {
                return existing.clone();
            }

            let mut existing = &deref_cache.core;
            for idx in 1..pattern.len() {
                let prefix = &pattern[..(pattern.len() - idx)];
                let suffix = &pattern[idx..];

                if let Some(prefix) = deref_cache.suffix.get(prefix) {
                    existing = prefix;
                    break;
                } else if let Some(suffix) = deref_cache.suffix.get(suffix) {
                    existing = suffix;
                    break;
                } else {
                }
            }

            let result: HashMap<String, i16> = existing
                .keys()
                .filter_map(|key| {
                    key.parse::<ValuePointer>()
                        .ok()
                        .and_then(|pointer| pointer.get(doc).ok())
                        .and_then(|node| match node {
                            &Value::String(ref value) => {
                                let potential_score = (pattern.len() * 16) as i16;
                                value.ends_with(pattern).then_some(potential_score)
                            }
                            &Value::Bool(value) => {
                                let potential_score = (pattern.len() * 16) as i16;
                                value
                                    .to_string()
                                    .ends_with(pattern)
                                    .then_some(potential_score)
                            }
                            &Value::Number(ref value) => {
                                let potential_score = (pattern.len() * 16) as i16;
                                value
                                    .to_string()
                                    .ends_with(pattern)
                                    .then_some(potential_score)
                            }
                            &Value::Array(ref arr) => {
                                let mut score = 0;
                                for i in 0..arr.len() {
                                    let potential_score = (pattern.len() * 16) as i16;
                                    if i.to_string().ends_with(pattern) {
                                        score = cmp::max(score, potential_score);
                                    }
                                }

                                Some(score)
                            }
                            &Value::Object(ref obj) => {
                                let mut score = 0;
                                for key in obj.keys() {
                                    let potential_score = (pattern.len() * 16) as i16;
                                    if key.ends_with(pattern) {
                                        score = cmp::max(score, potential_score);
                                    }
                                }

                                Some(score)
                            }
                            &Value::Null => None,
                        })
                        .and_then(|score| (score > 0).then_some((key.clone(), score)))
                })
                .collect();

            deref_cache.suffix.set(pattern, result.clone());

            result
        }
        &Pattern::Negated(ref pattern) => {
            let mut result = HashMap::new();
            let scored = score_deref(pattern, deref_cache, doc);
            let max_score = scored.values().max().unwrap_or(&0);

            for key in deref_cache.core.keys() {
                if !scored.contains_key(key) {
                    result.insert(key.clone(), *max_score);
                }
            }

            result
        }
        &Pattern::Deref(_) => HashMap::new(),
    }
}
