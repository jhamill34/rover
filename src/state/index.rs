//!

use std::collections::{HashMap, HashSet};

use json_pointer::JsonPointer;

///
pub struct Doc {
    ///
    pub root: String,

    ///
    pub adj_list: HashMap<String, Vec<String>>,
}

///
pub const ROOT_PATH: &str = "#";

impl Doc {
    ///
    pub fn build_from(doc: &serde_json::Value) -> Self {
        let mut adj_list = HashMap::new();
        let mut stack = vec![(doc, ROOT_PATH.to_owned())];

        let mut seen = HashSet::<String>::new();

        while let Some((node, path)) = stack.pop() {
            if seen.contains(&path) {
                continue;
            }

            seen.insert(path.clone());

            match *node {
                serde_json::Value::Object(ref value) => {
                    let mut children = vec![];
                    for (key, child) in value {
                        let key = key.replace('/', "~1");

                        if key == *"$ref" {
                            if let &serde_json::Value::String(ref child) = child {
                                let ref_node = child
                                    .strip_prefix('#')
                                    .and_then(|path| path.parse::<JsonPointer<_, _>>().ok())
                                    .and_then(|path| path.get(doc).ok());
                                if let Some(ref_node) = ref_node {
                                    if !seen.contains(child) {
                                        stack.push((ref_node, child.clone()));
                                    }

                                    children = vec![child.clone()];

                                    break;
                                }
                            }
                        }

                        let child_path = format!("{path}/{key}");

                        if !seen.contains(&child_path) {
                            stack.push((child, child_path.clone()));
                        }

                        children.push(child_path);
                    }

                    adj_list.insert(path, children);
                }
                serde_json::Value::Array(ref value) => {
                    let mut children = vec![];
                    for (index, child) in value.iter().enumerate() {
                        let child_path = format!("{path}/{index}");

                        if !seen.contains(&child_path) {
                            stack.push((child, child_path.clone()));
                        }

                        children.push(child_path);
                    }

                    adj_list.insert(path, children);
                }
                serde_json::Value::Null |
                serde_json::Value::Bool(_) |
                serde_json::Value::Number(_) |
                serde_json::Value::String(_) => {
                    adj_list.insert(path, vec![]);
                }
            }
        }

        Self {
            adj_list,
            root: ROOT_PATH.to_owned(),
        }
    }
}
