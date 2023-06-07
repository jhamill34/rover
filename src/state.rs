//!

use std::collections::HashMap;

///
pub const ROOT_PATH: &str = "#";

///
pub struct State {
    ///
    pub file_name: String,

    /// 
    /// TODO: we need to implement our own data structure to 
    ///  have control over the key ordering (like IndexMap)
    pub doc: serde_json::Value,

    ///
    pub current_page: Page,

    ///
    pub nav_state: Nav,

    ///
    pub search_state: Search,

    ///
    pub import_prompt_state: ImportPrompt,

    ///
    pub export_prompt_state: ExportPrompt,

    ///
    pub status: Status,
}

///
pub enum StatusMessage {
    ///
    Ok(String),

    ///
    Warn(String),

    ///
    Err(String),

    ///
    Empty,
}

///
pub struct Status {
    ///
    pub message: StatusMessage,

    ///
    pub timeout: Option<std::time::Instant>,

}

impl State {
    ///
    pub fn new(doc: serde_json::Value, file_name: String) -> Self {
        Self {
            file_name,
            doc,
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
                filtered_paths: vec![],
                all_paths: HashMap::new(),
                selected: 0,
            },
            import_prompt_state: ImportPrompt {
                value: String::new(),
            },
            export_prompt_state: ExportPrompt {
                value: String::new(),
            },
            status: Status {
                message: StatusMessage::Empty,
                timeout: None,
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

    ///
    pub all_paths: HashMap<String, usize>,

    ///
    pub filtered_paths: Vec<String>,

    ///
    pub selected: usize,
}

///
pub struct ImportPrompt {
    ///
    pub value: String,
}

///
pub struct ExportPrompt {
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

    ///
    ImportPrompt,

    ///
    ExportPrompt,
}

