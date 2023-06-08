//!

use crate::{
    state::{Page, StatusMessage},
    value::Value,
};

///
#[non_exhaustive]
pub enum Action {
    ///
    SetCurrentPage {
        ///
        page: Page,
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
    NavMoveDown,

    ///
    NavMoveUp,

    ///
    NavTop,

    ///
    NavGoto {
        ///
        path: String,
    },

    ///
    NavBottom,

    ///
    DocumentReplaceCurrent {
        ///
        value: Value,
    },

    ///
    Undo,

    ///
    Redo,

    ///
    Snapshot,

    ///
    SearchUp,

    ///
    SearchDown,

    ///
    SearchSetValue {
        ///
        value: String,
    },

    ///
    SearchSetAllPaths,

    ///
    ImportPromptSetValue {
        ///
        value: String,
    },

    ///
    ExportPromptSetValue {
        ///
        value: String,
    },

    ///
    SetStatus {
        ///
        message: StatusMessage,

        ///
        timeout: Option<core::time::Duration>,
    },
}
