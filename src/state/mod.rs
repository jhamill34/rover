//!

use self::index::{Doc, ROOT_PATH};

pub mod index;

///
pub struct State {
    ///
    pub file_name: String,

    ///
    pub doc: serde_json::Value,

    ///
    pub index: Doc,

    ///
    pub current_page: Page,

    ///
    pub nav_state: Nav,

    ///
    pub search_state: Search,
}

impl State {
    ///
    pub fn new(doc: serde_json::Value, index: Doc, file_name: String) -> Self {
        Self {
            file_name,
            doc,
            index,
            current_page: Page::Nav,
            nav_state: Nav {
                current: Step {
                    selected: 0,
                    options: vec![ROOT_PATH.to_owned()],
                },
                history: vec![],
            },
            search_state: Search {
                value: String::new(),
            },
        }
    }
}

///
pub struct Step {
    ///
    pub options: Vec<String>,

    ///
    pub selected: usize,
}

///
pub struct Nav {
    ///
    pub current: Step,

    ///
    pub history: Vec<Step>,
}

///
pub struct Search {
    ///
    pub value: String,
}

///
#[derive(Clone, Copy)]
pub enum Page {
    ///
    Nav,

    ///
    Search,
}
