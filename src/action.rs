//!

use crate::{state::{Page, StatusMessage}, value::Value};

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
        value: Value 
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
    SearchSetAllPaths,

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
        timeout: Option<core::time::Duration>,
    },
}
