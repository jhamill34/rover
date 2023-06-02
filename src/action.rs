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
    NavBottom,

    ///
    DocumentReplaceCurrent { 
        ///
        value: serde_json::Value 
    },
    
    ///
    SearchSetValue { 
        ///
        value: String 
    },
}
