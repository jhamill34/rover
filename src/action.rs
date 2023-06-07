//!

use crate::state::{Page, StatusMessage};

///
#[non_exhaustive]
pub enum Action {
    ///
    SetCurrentPage { 
        ///
        page: Page 
    },

    ///
    NavBack,

    ///
    NavSelect,

    ///
    NavUp,

    ///
    NavDown,

    ///
    NavTop,

    ///
    NavGoto { 
        ///
        path: String 
    },

    ///
    NavBottom,

    ///
    DocumentReplaceCurrent { 
        ///
        value: serde_json::Value 
    },
    
    ///
    SearchUp,

    ///
    SearchDown,

    ///
    SearchSetValue { 
        ///
        value: String 
    },

    ///
    ImportPromptSetValue { 
        ///
        value: String 
    },

    ///
    ExportPromptSetValue { 
        ///
        value: String 
    },

    ///
    SetStatus { 
        ///
        message: StatusMessage,

        ///
        timeout: Option<std::time::Duration>,
    },
}
