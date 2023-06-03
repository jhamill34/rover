#![allow(clippy::too_many_lines)]

//!

use json_pointer::JsonPointer;

use crate::{
    action::Action,
    state::{index::Doc, Search, State, Step, self}, search::filter,
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
        Action::NavGoto { path } => {
            let parts: Vec<_> = path.split('/').collect(); 
            
            let mut history = vec![];
            let mut current = Step {
                options: vec![state::index::ROOT_PATH.to_owned()],
                selected: 0,
            };
            let mut current_path = String::new();

            for part in parts {
                if !current_path.is_empty() {
                    current_path.push('/');
                }
                current_path.push_str(part);

                // Fake Move Up/Down
                current.selected = current.options
                    .iter()
                    .position(|opt| *opt == current_path)
                    .unwrap_or_default();

                // Fake Select Step
                let next_options = state.index.adj_list
                    .get(&current_path)
                    .cloned()
                    .unwrap_or_default();

                if next_options.is_empty() {
                    break;
                }

                let mut next_step = Step {
                    selected: 0,
                    options: next_options,
                };

                core::mem::swap(&mut current, &mut next_step);

                history.push(next_step);
            }

            state.nav_state.current = current;
            state.nav_state.history = history;

            state
        }
        Action::SearchSetValue { value } => {
            // TODO: Use FZF on path and value algorithm
            let filtered_list: Vec<_> = filter(&state.doc, &state.index.adj_list, &value);
            state.search_state.filtered_paths = filtered_list;
            state.search_state.value = value;

            state
        }
        Action::SearchUp => {
            let filtered_count = state.search_state.filtered_paths.len();
            let new_selected = if state.search_state.selected > 0 {
                state.search_state.selected - 1
            } else {
                filtered_count - 1
            };

            state.search_state.selected = new_selected;

            state
        }
        Action::SearchDown => {
            let filtered_count = state.search_state.filtered_paths.len();
            let new_selected = (state.search_state.selected + 1) % filtered_count;
            state.search_state.selected = new_selected;

            state
        }
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
                    for (other_key, other_value) in index.adj_list {
                        let other_key = other_key.replace('#', path);
                        let other_value = other_value.iter().map(|child_path| child_path.replace('#', path)).collect();
                        state.index.adj_list.insert(other_key, other_value);
                    }

                    *existing = value;
                }
            }

            state
        }
    }
}
