//!

use std::collections::HashMap;

use crate::value::Value;

use self::cache::SimpleStore;

mod algs;
pub mod cache;
mod parser;

///
#[derive(Debug)]
pub struct PatternCache {
    ///
    pub core: HashMap<String, i16>,
    ///
    pub exact: SimpleStore<HashMap<String, i16>>,
    ///
    pub prefix: SimpleStore<HashMap<String, i16>>,
    ///
    pub suffix: SimpleStore<HashMap<String, i16>>,
    ///
    pub fuzzy: SimpleStore<HashMap<String, i16>>,
}

impl PatternCache {
    ///
    pub fn new(capacity: usize) -> Self {
        Self {
            core: HashMap::new(),
            exact: SimpleStore::new(capacity),
            prefix: SimpleStore::new(capacity),
            suffix: SimpleStore::new(capacity),
            fuzzy: SimpleStore::new(capacity),
        }
    }

    ///
    pub fn reset(&mut self, core: HashMap<String, i16>) {
        self.core = core;

        self.exact.clear();
        self.prefix.clear();
        self.suffix.clear();
        self.fuzzy.clear();
    }
}

///
pub fn search(
    doc: &Value,
    cache: &mut PatternCache,
    deref_cache: &mut PatternCache,
    query: &str,
) -> Vec<String> {
    let patterns = query.parse::<parser::Pattern>();

    if let Ok(patterns) = patterns {
        let mut result = algs::score(&patterns, cache, deref_cache, doc)
            .into_iter()
            .collect::<Vec<_>>();
        result.sort_by(|&(_, ref a), &(_, ref b)| b.cmp(a));
        result.into_iter().map(|(key, _)| key).collect()
    } else {
        Vec::new()
    }
}
