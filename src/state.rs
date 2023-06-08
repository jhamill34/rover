//!

use std::collections::HashMap;

use crate::value::Value;

///
pub const ROOT_PATH: &str = "#";

///
pub struct State {
    ///
    pub file_name: String,

    ///
    pub doc: Value,

    ///
    pub current_page: Page,

    ///
    pub undo_stack: Vec<UndoAction>,

    ///
    pub redo_stack: Vec<UndoAction>,

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
pub enum UndoAction {
    ///
    ReplaceCurrent {
        /// 
        path: String, 

        ///
        value: Value,
    },

    ///
    SwapIndicies {
        ///
        path: String,

        ///
        from: usize,

        ///
        to: usize,
    }
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
    pub fn new(doc: Value, file_name: String) -> Self {
        Self {
            file_name,
            doc,
            current_page: Page::Nav,
            undo_stack: vec![],
            redo_stack: vec![],
            nav_state: Nav {
                current: Step {
                    selected: 0,
                    path: ROOT_PATH.to_string(),
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
    pub path: String,

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

