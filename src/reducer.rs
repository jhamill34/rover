#![allow(clippy::too_many_lines)]

//!

use std::collections::HashMap;

use crate::{
    action::Action,
    state::{State, Step, self, ROOT_PATH}, search::filter, pointer::ValuePointer, value::Value,
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
            let selected_path = state.nav_state.current.path.clone();

            if let Some(selected) = selected_path.parse::<ValuePointer>().ok().and_then(|pointer| pointer.get(&state.doc).ok()) {
                let index = state.nav_state.current.selected;
                let path = match selected {
                    &Value::Object(ref map) => {
                        map.get_index(index)
                            .map(|(key, _)| {
                                let key = key.replace('/', "~1");

                                format!("{selected_path}/{key}")
                            })
                    }
                    &Value::Array(_) => {
                        Some(format!("{selected_path}/{index}"))
                    }
                    &Value::Null |
                        &Value::Bool(_) |
                        &Value::Number(_) |
                        &Value::String(_) => None,
                };

                if let Some(path) = path {
                    let mut step = Step {
                        path,
                        selected: 0,
                    };

                    core::mem::swap(&mut state.nav_state.current, &mut step);
                    state.nav_state.history.push(step);
                }
            }

            state
        }
        Action::NavUp => {
            let option_count = state.nav_state.current.path.parse::<ValuePointer>().ok()
                .and_then(|pointer| pointer.get(&state.doc).ok())
                .map_or(0, |value| {
                    match value {
                        &Value::Object(ref map) => map.len(),
                        &Value::Array(ref array) => array.len(),
                        &Value::Null |
                            &Value::Bool(_) |
                            &Value::Number(_) |
                            &Value::String(_) => 0
                    }

                })
                .saturating_sub(1);

            let new_selected = state.nav_state.current.selected
                .checked_sub(1)
                .unwrap_or(option_count);

            state.nav_state.current.selected = new_selected;

            state
        }
        Action::NavMoveUp => {
            if let Some(previous) = state.nav_state.current.path.parse::<ValuePointer>().ok().and_then(|pointer| pointer.get_mut(&mut state.doc).ok()) {
                let option_count = match previous {
                    &mut Value::Object(ref map) => map.len(),
                    &mut Value::Array(ref array) => array.len(),
                    &mut (Value::Null |
                        Value::Bool(_) |
                        Value::Number(_) |
                        Value::String(_)) => 0
                    }.saturating_sub(1);

                let cur = state.nav_state.current.selected;
                let new_selected = cur
                    .checked_sub(1)
                    .unwrap_or(option_count);

                match previous {
                    &mut Value::Object(ref mut obj) => {
                        obj.swap_indices(cur, new_selected);
                        state.undo_stack.push(state::UndoAction::SwapIndicies { 
                            path: state.nav_state.current.path.clone(), 
                            from: new_selected, 
                            to: cur
                        });
                        state.redo_stack.clear();
                    },
                    &mut Value::Array(ref mut arr) => {
                        arr.swap(cur, new_selected);
                        state.undo_stack.push(state::UndoAction::SwapIndicies { 
                            path: state.nav_state.current.path.clone(), 
                            from: new_selected, 
                            to: cur
                        });
                        state.redo_stack.clear();
                    },
                    &mut (
                        Value::Null |
                        Value::Bool(_) |
                        Value::Number(_) |
                        Value::String(_)
                        ) => {},
                }

                state.nav_state.current.selected = new_selected;
            }

            state
        }
        Action::NavDown => {
            let option_count = state.nav_state.current.path.parse::<ValuePointer>().ok()
                .and_then(|pointer| pointer.get(&state.doc).ok())
                .map_or(0, |value| {
                    match value {
                        &Value::Object(ref map) => map.len(),
                        &Value::Array(ref array) => array.len(),
                        &Value::Null |
                            &Value::Bool(_) |
                            &Value::Number(_) |
                            &Value::String(_) => 0
                    }

                });

            let new_selected = state.nav_state.current.selected
                .wrapping_add(1)
                .checked_rem_euclid(option_count)
                .unwrap_or_default();

            state.nav_state.current.selected = new_selected;

            state
        }
        Action::NavMoveDown => {
            if let Some(previous) = state.nav_state.current.path.parse::<ValuePointer>().ok().and_then(|pointer| pointer.get_mut(&mut state.doc).ok()) {
                let option_count = match previous {
                    &mut Value::Object(ref map) => map.len(),
                    &mut Value::Array(ref array) => array.len(),
                    &mut (Value::Null |
                        Value::Bool(_) |
                        Value::Number(_) |
                        Value::String(_)) => 0
                    };

                let cur = state.nav_state.current.selected;
                let new_selected = cur 
                    .wrapping_add(1)
                    .checked_rem_euclid(option_count)
                    .unwrap_or_default();

                match previous {
                    &mut Value::Object(ref mut obj) => {
                        obj.swap_indices(cur, new_selected);
                        state.undo_stack.push(state::UndoAction::SwapIndicies { 
                            path: state.nav_state.current.path.clone(), 
                            from: new_selected, 
                            to: cur
                        });
                        state.redo_stack.clear();
                    },
                    &mut Value::Array(ref mut arr) => {
                        arr.swap(cur, new_selected);
                        state.undo_stack.push(state::UndoAction::SwapIndicies { 
                            path: state.nav_state.current.path.clone(), 
                            from: new_selected, 
                            to: cur
                        });
                        state.redo_stack.clear();
                    },
                    &mut (Value::Null |
                          Value::Bool(_) |
                          Value::String(_) |
                          Value::Number(_)) => {},
                }

                state.nav_state.current.selected = new_selected;
            }

            state
        }
        Action::NavTop => {
            state.nav_state.current.selected = 0;
            state
        }
        Action::NavBottom => {
            let option_count = state.nav_state.current.path.parse::<ValuePointer>().ok()
                .and_then(|pointer| pointer.get(&state.doc).ok())
                .map_or(0, |value| {
                    match value {
                        &Value::Object(ref map) => map.len(),
                        &Value::Array(ref array) => array.len(),
                        &Value::Null |
                            &Value::Bool(_) |
                            &Value::Number(_) |
                            &Value::String(_) => 0
                    }

                });

            state.nav_state.current.selected = option_count.saturating_sub(1);

            state
        }
        Action::NavGoto { path } => {
            // let parts: Vec<_> = path.split('/').collect(); 
            // 
            // let mut history = Vec::with_capacity(parts.len());
            // let mut current = Step {
            //     options: vec![state::ROOT_PATH.to_owned()],
            //     selected: 0,
            // };
            // let mut current_path = String::new();

            // for part in parts {
            //     if !current_path.is_empty() {
            //         current_path.push('/');
            //     }
            //     current_path.push_str(part);

            //     // Fake Move Up/Down
            //     current.selected = current.options
            //         .iter()
            //         .position(|opt| *opt == current_path)
            //         .unwrap_or_default();

            //     // Fake Select Step
            //     let next_options = current_path.parse::<ValuePointer>().ok()
            //         .and_then(|pointer| pointer.get(&state.doc).ok())
            //         .map_or(vec![], |value| {
            //             match value {
            //                 &Value::Object(ref map) => {
            //                     if let Some(&Value::String(ref value)) = map.get("$ref") {
            //                         vec![value.clone()]
            //                     } else {
            //                         map.keys().map(|key| {
            //                             let key = key.replace('/', "~1");
            //                             format!("{current_path}/{key}")
            //                         }).collect()
            //                     }
            //                 },
            //                 &Value::Array(ref array) => array.iter().enumerate().map(|(index, _)| format!("{current_path}/{index}")).collect(),
            //                 &Value::Null |
            //                     &Value::Bool(_) |
            //                     &Value::Number(_) |
            //                     &Value::String(_) => vec![],

            //             }
            //         });

            //     if next_options.is_empty() {
            //         break;
            //     }

            //     let mut next_step = Step {
            //         selected: 0,
            //         options: next_options,
            //     };

            //     core::mem::swap(&mut current, &mut next_step);

            //     history.push(next_step);
            // }

            // state.nav_state.current = current;
            // state.nav_state.history = history;

            state
        }
        Action::SearchSetValue { value } => {
            let filtered_list: Vec<_> = filter(&state.doc, &state.search_state.all_paths, &value);
            state.search_state.filtered_paths = filtered_list;
            state.search_state.value = value;

            state
        }
        Action::SearchUp => {
            let filtered_count = state.search_state.filtered_paths
                .len()
                .saturating_sub(1);

            let new_selected = state.search_state.selected
                .checked_sub(1)
                .unwrap_or(filtered_count);

            state.search_state.selected = new_selected;

            state
        }
        Action::SearchDown => {
            let filtered_count = state.search_state.filtered_paths.len();
            let new_selected = state.search_state.selected
                .wrapping_add(1)
                .checked_rem_euclid(filtered_count)
                .unwrap_or_default();

            state.search_state.selected = new_selected;

            state
        }
        Action::DocumentReplaceCurrent { value } => {
            let existing = state.nav_state.current.path
                .parse::<ValuePointer>().ok()
                .and_then(|pointer| pointer.get_mut(&mut state.doc).ok());

            if let Some(existing) = existing {
                let existing = match existing {
                    &mut Value::Object(ref mut map) => map.get_index_mut(state.nav_state.current.selected).map(|(k, v)| (format!("{}/{k}", state.nav_state.current.path), v)),
                    &mut Value::Array(ref mut arr) => arr.get_mut(state.nav_state.current.selected).map(|v| (format!("{}/{}", state.nav_state.current.path, state.nav_state.current.selected), v)),
                    &mut (Value::Null |
                          Value::Bool(_) |
                          Value::String(_) |
                          Value::Number(_)) => {
                        None
                    },
                };


                if let Some((path, existing)) = existing {
                    state.undo_stack.push(state::UndoAction::ReplaceCurrent { 
                        path, 
                        value: existing.clone() 
                    });
                    state.redo_stack.clear();

                    *existing = value;
                }
            }

            state
        }
        Action::ImportPromptSetValue { value } => {
            state.import_prompt_state.value = value;
            state
        },
        Action::ExportPromptSetValue { value } => {
            state.export_prompt_state.value = value;
            state
        },
        Action::SetStatus { message, timeout  } => {
            state.status.message = message;
            state.status.timeout = timeout.and_then(|dur| std::time::Instant::now().checked_add(dur));
            state
        },
        Action::SearchSetAllPaths => {
            let mut stack = vec![(ROOT_PATH.to_owned(), &state.doc)];
            let mut paths = HashMap::new();

            while let Some((path, value)) = stack.pop() {
                let mut child_count: usize = 0;
                match value {
                    &Value::Object(ref map) => {
                        if !map.contains_key("$ref") {
                            for (key, value) in map {
                                let key = key.replace('/', "~1");
                                let path = format!("{path}/{key}");
                                stack.push((path, value));
                                child_count = child_count.saturating_add(1);
                            }
                        }
                    }
                    &Value::Array(ref array) => {
                        for (index, value) in array.iter().enumerate() {
                            let path = format!("{path}/{index}");
                            stack.push((path, value));
                            child_count = child_count.saturating_add(1);
                        }
                    }
                    &Value::Null |
                    &Value::Bool(_) |
                    &Value::Number(_) |
                    &Value::String(_) => {},
                };
                
                paths.insert(path, child_count);
            }

            state.search_state.all_paths = paths;

            state
        },
        Action::Undo => {
            if let Some(top) = state.undo_stack.pop() {
                match top {
                    state::UndoAction::ReplaceCurrent { path, value } => {
                        if let Some(node) = path.parse::<ValuePointer>().ok()
                            .and_then(|pointer| pointer.get_mut(&mut state.doc).ok()) {
                                state.redo_stack.push(state::UndoAction::ReplaceCurrent { path, value: node.clone() });

                                *node = value;

                                state.status.message = state::StatusMessage::Ok("Successfully undid value replacement".to_owned());
                                state.status.timeout = Some(core::time::Duration::from_secs(2)).and_then(|dur| {
                                    std::time::Instant::now().checked_add(dur)
                                });
                        } else {
                            state.undo_stack.clear(); 
                            state.redo_stack.clear();
                            state.status.message = state::StatusMessage::Err("Corrupted undo stack, try reloading the document".to_owned());
                            state.status.timeout = None;
                        }
                    },
                    state::UndoAction::SwapIndicies { path, from, to } => {
                        if let Some(node) = path.parse::<ValuePointer>().ok()
                            .and_then(|pointer| pointer.get_mut(&mut state.doc).ok()) {
                                match node {
                                    &mut Value::Array(ref mut arr) => {
                                        state.redo_stack.push(state::UndoAction::SwapIndicies { path, from, to });

                                        arr.swap(from, to);
                                        state.status.message = state::StatusMessage::Ok("Successfully undid array index move".to_owned());
                                        state.status.timeout = Some(core::time::Duration::from_secs(2)).and_then(|dur| {
                                            std::time::Instant::now().checked_add(dur)
                                        });
                                    },
                                    &mut Value::Object(ref mut obj) => {
                                        state.redo_stack.push(state::UndoAction::SwapIndicies { path, from, to });

                                        obj.swap_indices(from, to);
                                        state.status.message = state::StatusMessage::Ok("Successfully undid object key move".to_owned());
                                        state.status.timeout = Some(core::time::Duration::from_secs(2)).and_then(|dur| {
                                            std::time::Instant::now().checked_add(dur)
                                        });
                                    },
                                    &mut (Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_)) => {
                                        state.undo_stack.clear(); 
                                        state.redo_stack.clear();
                                        state.status.message = state::StatusMessage::Err("Corrupted undo stack, try reloading the document".to_owned());
                                        state.status.timeout = None;
                                    },
                                }
                        } else {
                            state.undo_stack.clear(); 
                            state.redo_stack.clear();
                            state.status.message = state::StatusMessage::Err("Corrupted undo stack, try reloading the document".to_owned());
                            state.status.timeout = None;
                        }
                    },
                }
            } else {
                state.status.message = state::StatusMessage::Warn("Nothing to undo".to_owned());
                state.status.timeout = Some(core::time::Duration::from_secs(2)).and_then(|dur| {
                    std::time::Instant::now().checked_add(dur)
                });
            }

            state
        }
        Action::Redo => {
            if let Some(top) = state.redo_stack.pop() {
                match top {
                    state::UndoAction::ReplaceCurrent { path, value } => {
                        if let Some(node) = path.parse::<ValuePointer>().ok()
                            .and_then(|pointer| pointer.get_mut(&mut state.doc).ok()) {
                                state.undo_stack.push(state::UndoAction::ReplaceCurrent { path, value: node.clone() });

                                *node = value;

                                state.status.message = state::StatusMessage::Ok("Successfully redid value replacement".to_owned());
                                state.status.timeout = Some(core::time::Duration::from_secs(2)).and_then(|dur| {
                                    std::time::Instant::now().checked_add(dur)
                                });
                        } else {
                            state.undo_stack.clear();
                            state.redo_stack.clear();
                            state.status.message = state::StatusMessage::Err("Corrupted redo stack, try reloading the document".to_owned());
                            state.status.timeout = None;
                        }
                    },
                    state::UndoAction::SwapIndicies { path, from, to } => {
                        if let Some(node) = path.parse::<ValuePointer>().ok()
                            .and_then(|pointer| pointer.get_mut(&mut state.doc).ok()) {
                                match node {
                                    &mut Value::Array(ref mut arr) => {
                                        state.undo_stack.push(state::UndoAction::SwapIndicies { path, from, to });

                                        arr.swap(from, to);
                                        state.status.message = state::StatusMessage::Ok("Successfully redid array index move".to_owned());
                                        state.status.timeout = Some(core::time::Duration::from_secs(2)).and_then(|dur| {
                                            std::time::Instant::now().checked_add(dur)
                                        });
                                    },
                                    &mut Value::Object(ref mut obj) => {
                                        state.undo_stack.push(state::UndoAction::SwapIndicies { path, from, to });

                                        obj.swap_indices(from, to);
                                        state.status.message = state::StatusMessage::Ok("Successfully redid object key move".to_owned());
                                        state.status.timeout = Some(core::time::Duration::from_secs(2)).and_then(|dur| {
                                            std::time::Instant::now().checked_add(dur)
                                        });
                                    },
                                    &mut (Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_)) => {
                                        state.undo_stack.clear(); 
                                        state.redo_stack.clear();
                                        state.status.message = state::StatusMessage::Err("Corrupted redo stack, try reloading the document".to_owned());
                                        state.status.timeout = None;
                                    },
                                }
                        } else {
                            state.undo_stack.clear(); 
                            state.redo_stack.clear();
                            state.status.message = state::StatusMessage::Err("Corrupted redo stack, try reloading the document".to_owned());
                            state.status.timeout = None;
                        }
                    },
                }
            } else {
                state.status.message = state::StatusMessage::Warn("Nothing to redo".to_owned());
                state.status.timeout = Some(core::time::Duration::from_secs(2)).and_then(|dur| {
                    std::time::Instant::now().checked_add(dur)
                });
            }

            state
        }
    }
}
