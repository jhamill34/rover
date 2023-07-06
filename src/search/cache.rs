//!

///
#[derive(Debug, Default)]
pub struct SimpleStore<T> {
    ///
    pub capacity: usize,

    ///
    pub cache: indexmap::IndexMap<String, T>,
}

impl<T> SimpleStore<T> {
    ///
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            cache: indexmap::IndexMap::with_capacity(capacity),
        }
    }
}

impl<T> SimpleStore<T> {
    ///
    pub fn get(&self, key: &str) -> Option<&T> {
        self.cache.get(key)
    }

    ///
    pub fn set(&mut self, key: &str, value: T) {
        self.cache.insert(key.to_owned(), value);
    }

    ///
    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_used, clippy::similar_names)]

    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_basic() {
        let root = HashSet::from([String::from("a"), String::from("b"), String::from("c")]);

        let mut cache = SimpleStore::new(100);

        let root_view: HashSet<&String> = root.iter().collect();
        cache.set("", root_view);

        let root_view = cache.get("").unwrap();
        assert_eq!(3, root_view.len());

        let sub_view = root_view
            .iter()
            .filter_map(|x| x.starts_with('a').then_some(*x))
            .collect::<HashSet<_>>();
        cache.set("sub_view", sub_view);

        let sub_view = cache.get("sub_view").unwrap();
        assert_eq!(1, sub_view.len());
    }

    #[test]
    fn test_merge() {
        let root = HashSet::from([String::from("a"), String::from("b"), String::from("c")]);

        let mut cache = SimpleStore::new(100);

        let root_view: HashSet<&String> = root.iter().collect();
        cache.set("", root_view);

        let root_view = cache.get("").unwrap();
        let a_view = root_view
            .iter()
            .filter_map(|x| x.starts_with('a').then_some(*x))
            .collect::<HashSet<_>>();
        cache.set("a_view", a_view);

        let root_view = cache.get("").unwrap();
        let b_view = root_view
            .iter()
            .filter_map(|x| x.starts_with('b').then_some(*x))
            .collect::<HashSet<_>>();
        cache.set("b_view", b_view);

        let a_view = cache.get("a_view").unwrap();
        let b_view = cache.get("b_view").unwrap();

        let ab_view = a_view.union(b_view).copied().collect::<HashSet<_>>();
        cache.set("ab_view", ab_view);
    }
}
