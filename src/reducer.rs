//!

use json_pointer::JsonPointer;

use crate::{
    action::Action,
    state::{index::Doc, Search, State, Step},
};

///
pub fn reducer(mut state: State, action: Action) -> State {
    match action {
        Action::SetCurrentPage { page } => {
            state.current_page = page;

            state
        }
        Action::NavBack => {
            if let Some(step) = state.nav_state.history.pop() {
                state.nav_state.current = step;
            }

            state
        }
        Action::NavSelect => {
            if let Some(selected_path) = state
                .nav_state
                .current
                .options
                .get(state.nav_state.current.selected)
            {
                if let Some(selected_children) = state.index.adj_list.get(selected_path) {
                    if !selected_children.is_empty() {
                        let mut step = Step {
                            options: selected_children.clone(),
                            selected: 0,
                        };

                        core::mem::swap(&mut state.nav_state.current, &mut step);
                        state.nav_state.history.push(step);
                    }
                }
            }

            state
        }
        Action::NavUp => {
            let option_count = state.nav_state.current.options.len();
            let new_selected = if state.nav_state.current.selected > 0 {
                state.nav_state.current.selected - 1
            } else {
                option_count - 1
            };

            state.nav_state.current.selected = new_selected;

            state
        }
        Action::NavDown => {
            let option_count = state.nav_state.current.options.len();
            let new_selected = (state.nav_state.current.selected + 1) % option_count;
            state.nav_state.current.selected = new_selected;

            state
        }
        Action::NavTop => {
            state.nav_state.current.selected = 0;
            state
        }
        Action::NavBottom => {
            let option_count = state.nav_state.current.options.len();
            state.nav_state.current.selected = option_count - 1;
            state
        }
        Action::SearchSetValue { value } => State {
            search_state: Search { value },
            ..state
        },
        Action::DocumentReplaceCurrent { value } => {
            if let Some(path) = state
                .nav_state
                .current
                .options
                .get(state.nav_state.current.selected)
            {
                let existing = path
                    .parse::<JsonPointer<_, _>>()
                    .ok()
                    .and_then(|pointer| pointer.get_mut(&mut state.doc).ok());

                if let Some(existing) = existing {
                    let index = Doc::build_from(&value);
                    let root = index.adj_list.get(&index.root).cloned().unwrap_or_default();
                    state.index.adj_list.insert(path.clone(), root);
                    *existing = value;
                }
            }

            state
        }
    }
}
