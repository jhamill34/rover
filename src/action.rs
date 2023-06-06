//!

use crate::state::Page;

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
}
