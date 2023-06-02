use self::index::{DocIndex, ROOT_PATH};

pub mod index;

pub struct State {
    pub file_name: String,
    pub doc: serde_json::Value,
    pub index: DocIndex,
    pub current_page: Page,
    pub nav_state: NavState,
    pub search_state: SearchState,
}

impl State {
    pub fn new(doc: serde_json::Value, index: DocIndex, file_name: String) -> Self {
        Self {
            file_name,
            doc,
            index,
            current_page: Page::Nav,
            nav_state: NavState {
                current: Step {
                    selected: 0,
                    options: vec![ROOT_PATH.to_string()],
                },
                history: vec![],
            },
            search_state: SearchState {
                value: String::new(),
            },
        }
    }
}

pub struct Step {
    pub options: Vec<String>,
    pub selected: usize,
}

pub struct NavState {
    pub current: Step,
    pub history: Vec<Step>,
}

pub struct SearchState {
    pub value: String,
}

#[derive(Clone, Copy)]
pub enum Page {
    Nav,
    Search,
}
