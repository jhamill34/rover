#![allow(clippy::too_many_lines)]

//!

extern crate alloc;
use alloc::sync::Arc;
use anyhow::anyhow;

use core::time::Duration;

use std::{
    io::{self, Write as _},
    sync::Mutex, env, fs::{self, File}, path::PathBuf,
};

use crossterm::event::{self, poll, Event, KeyCode, KeyEvent, KeyModifiers};
use json_pointer::JsonPointer;
use redux_rs::{Reducer, Store};
use tui::backend::Backend;

use crate::{
    action::Action,
    lifecycle::Application,
    state::{Page, State, StatusMessage},
    util::{editor, save_doc},
};

///
pub async fn event_listener<R, B>(
    store: Store<State, Action, R>,
    lifecycle: Arc<Mutex<Application<B>>>,
) -> anyhow::Result<()>
where
    R: Reducer<State, Action> + Send + Sync + 'static,
    B: Backend + io::Write + Send + Sync + 'static,
{
    // TODO: Event buffering for multi key commands
    loop {
        let status_timeout = store.select(|state: &State| state.status.timeout).await;
        if let Some(status_timeout) = status_timeout {
            if status_timeout < std::time::Instant::now() {
                store.dispatch(Action::SetStatus { message: StatusMessage::Empty, timeout: None }).await;
            }
        }


        if poll(Duration::from_millis(100))? {
            let read_event = event::read()?;
            if let Event::Resize(height, width) = read_event {
                let lifecycle = Arc::clone(&lifecycle);
                store
                    .select(move |state: &State| -> anyhow::Result<()> {
                        let mut lifecycle = lifecycle.lock().map_err(|e| anyhow!("Unable to get lifecycle lock: {e}"))?;
                        lifecycle.resize(width, height)?;
                        lifecycle.refresh(state)?;
                        Ok(())
                    })
                    .await?;
                continue;
            }

            let current_view = store.select(|state: &State| state.current_page).await;
            match current_view {
                Page::Nav => {
                    if let Event::Key(key) = read_event {
                        match key {
                            KeyEvent {
                                code: KeyCode::Char('h'),
                                ..
                            } => store.dispatch(Action::NavBack).await,
                            KeyEvent {
                                code: KeyCode::Char('j'),
                                ..
                            } => store.dispatch(Action::NavDown).await,
                            KeyEvent {
                                code: KeyCode::Char('k'),
                                ..
                            } => store.dispatch(Action::NavUp).await,
                            KeyEvent {
                                code: KeyCode::Char('l') | KeyCode::Enter,
                                ..
                            } => {
                                let children = store
                                    .select(|state: &State| {
                                        state
                                            .nav_state
                                            .current
                                            .options
                                            .get(state.nav_state.current.selected)
                                            .and_then(|path| {
                                                path.strip_prefix('#')
                                                    .and_then(|path| {
                                                        path.parse::<JsonPointer<_, _>>().ok()
                                                            .and_then(|pointer| pointer.get(&state.doc).ok())
                                                            .map(|value| {
                                                                match value {
                                                                    &serde_json::Value::Array(ref arr) => arr.len(),
                                                                    &serde_json::Value::Object(ref obj) => obj.len(),
                                                                    &serde_json::Value::Null |
                                                                    &serde_json::Value::Bool(_) |
                                                                    &serde_json::Value::Number(_) |
                                                                    &serde_json::Value::String(_) => 0 
                                                                }
                                                            })
                                                    })
                                            })
                                            .unwrap_or(0)
                                    })
                                    .await;

                                if children > 0 {
                                    store.dispatch(Action::NavSelect).await;
                                } else {
                                    store.dispatch(Action::SetStatus { 
                                        message: StatusMessage::Warn("No children to select, use ^e to edit this value".to_owned()), 
                                        timeout: Some(Duration::from_secs(2)), 
                                    }).await;
                                }
                            }
                            KeyEvent {
                                code: KeyCode::Char('I'),
                                ..
                            } => {
                                let cwd = env::current_dir()?.to_string_lossy().to_string();
                                store.dispatch(Action::ImportPromptSetValue { value: cwd }).await;
                                store.dispatch(Action::SetCurrentPage { page: Page::ImportPrompt }).await;
                            }
                            KeyEvent {
                                code: KeyCode::Char('O'),
                                ..
                            } => {
                                let cwd = env::current_dir()?.to_string_lossy().to_string();
                                store.dispatch(Action::ExportPromptSetValue { value: cwd }).await;
                                store.dispatch(Action::SetCurrentPage { page: Page::ExportPrompt }).await;
                            }
                            KeyEvent {
                                code: KeyCode::Char('e'),
                                modifiers: KeyModifiers::CONTROL,
                                ..
                            } => {
                                let existing_value = store
                                    .select(|state: &State| {
                                        state
                                            .nav_state
                                            .current
                                            .options
                                            .get(state.nav_state.current.selected)
                                            .and_then(|path| path.strip_prefix('#'))
                                            .and_then(|path| path.parse::<JsonPointer<_, _>>().ok())
                                            .and_then(|path| path.get(&state.doc).ok())
                                            .cloned()
                                            .unwrap_or(serde_json::Value::Null)
                                    })
                                    .await;
                                
                                {
                                    let mut lifecycle = lifecycle.lock().map_err(|e| anyhow!("Unable to get lifecycle lock: {e}"))?;
                                    lifecycle.suspend()?;
                                }

                                let file_name = store.select(|state: &State| state.file_name.clone()).await;
                                let new_value = editor(&existing_value, &file_name);

                                {
                                    let mut lifecycle = lifecycle.lock().map_err(|e| anyhow!("Unable to get lifecycle lock: {e}"))?;
                                    lifecycle.resume()?;
                                }

                                match new_value {
                                    Ok(new_value) => {
                                        store.dispatch(Action::DocumentReplaceCurrent { value: new_value }).await;
                                        store.dispatch(Action::SetStatus { 
                                            message: StatusMessage::Ok("Successfully edited value".to_owned()), 
                                            timeout: Some(Duration::from_secs(2)) 
                                        }).await;
                                    }
                                    Err(e) => {
                                        store.dispatch(Action::SetStatus { 
                                            message: StatusMessage::Err(format!("Unable to edit value: {e}")), 
                                            timeout: None,
                                        }).await;
                                    }
                                }
                            }
                            KeyEvent {
                                code: KeyCode::Char('s'),
                                modifiers: KeyModifiers::CONTROL,
                                ..
                            } => {
                                let file_name =
                                    store.select(|state: &State| state.file_name.clone()).await;
                                let doc = store.select(|state: &State| state.doc.clone()).await;
                                let result = save_doc(&file_name, &doc);

                                match result {
                                    Ok(_) => {
                                        store.dispatch(Action::SetStatus { 
                                            message: StatusMessage::Ok("Successfully saved file".to_owned()), 
                                            timeout: Some(Duration::from_secs(2)) 
                                        }).await;
                                    }
                                    Err(e) => {
                                        store.dispatch(Action::SetStatus { 
                                            message: StatusMessage::Err(format!("Unable to save file: {e}")), 
                                            timeout: None,
                                        }).await;
                                    }
                                }
                            }
                            KeyEvent {
                                code: KeyCode::Char('g'),
                                ..
                            } => store.dispatch(Action::NavTop).await,
                            KeyEvent {
                                code: KeyCode::Char('G'),
                                ..
                            } => store.dispatch(Action::NavBottom).await,
                            KeyEvent {
                                code: KeyCode::Char('/'),
                                ..
                            } => {
                                store.dispatch(Action::SearchSetAllPaths).await;
                                store
                                    .dispatch(Action::SetCurrentPage { page: Page::Search })
                                    .await;
                            }
                            KeyEvent {
                                code: KeyCode::Char('q') | KeyCode::Esc,
                                ..
                            } => return Ok(()),
                            KeyEvent {
                                code: KeyCode::Char('c'),
                                modifiers: KeyModifiers::CONTROL,
                                ..
                            } => {
                                // TODO: Show warning if changes have been made before exiting
                                
                                let empty_status = store.select(|state: &State| matches!(state.status.message, StatusMessage::Empty)).await;
                                if empty_status {
                                    return Ok(())
                                }

                                store.dispatch(Action::SetStatus { 
                                    message: StatusMessage::Empty, 
                                    timeout: None,
                                }).await;
                            },
                            _ => {}
                        }
                    }
                }
                Page::ImportPrompt => {
                    if let Event::Key(key) = read_event {
                        match key {
                            KeyEvent { code: KeyCode::Char(ch), modifiers: KeyModifiers::SHIFT | KeyModifiers::NONE, .. } => { 
                                let mut current = store
                                    .select(|state: &State| state.import_prompt_state.value.clone())
                                    .await;
                                current.push(ch);
                                store.dispatch(Action::ImportPromptSetValue { value: current }).await;
                            },
                            KeyEvent { code: KeyCode::Backspace, .. } => {
                                let mut current = store
                                    .select(|state: &State| state.import_prompt_state.value.clone())
                                    .await;
                                current.pop();
                                store.dispatch(Action::ImportPromptSetValue { value: current }).await;
                            },
                            KeyEvent { code: KeyCode::Enter, .. } => {
                                let current_path = store
                                    .select(|state: &State| state.import_prompt_state.value.clone())
                                    .await;

                                let existing_value = match fs::read_to_string(&current_path) {
                                    Ok(existing_value) => existing_value,
                                    Err(e) => {
                                        store.dispatch(Action::SetStatus { 
                                            message: StatusMessage::Err(format!("Unable to read file: {e}")), 
                                            timeout: None,
                                        }).await;
                                        store.dispatch(Action::SetCurrentPage { page: Page::Nav }).await;
                                        continue;
                                    }
                                };

                                let extention = PathBuf::from(&current_path);
                                let Some(extention) = extention.extension().map(std::ffi::OsStr::to_string_lossy) else {
                                    store.dispatch(Action::SetStatus { 
                                        message: StatusMessage::Err("Unable to determine file type".to_owned()), 
                                        timeout: None,
                                    }).await;
                                    store.dispatch(Action::SetCurrentPage { page: Page::Nav }).await;
                                    continue;
                                };

                                let existing_value = match extention.as_ref() {
                                    "yaml" | "yml" =>  match serde_yaml::from_str(&existing_value) {
                                        Ok(existing_value) => existing_value,
                                        Err(e) => {
                                            store.dispatch(Action::SetStatus { 
                                                message: StatusMessage::Err(format!("Unable to parse file: {e}")), 
                                                timeout: None,
                                            }).await;
                                            store.dispatch(Action::SetCurrentPage { page: Page::Nav }).await;
                                            continue;
                                    }
                                    },
                                    "json" => match serde_json::from_str(&existing_value) {
                                        Ok(existing_value) => existing_value,
                                        Err(e) => {
                                            store.dispatch(Action::SetStatus { 
                                                message: StatusMessage::Err(format!("Unable to parse file: {e}")), 
                                                timeout: None,
                                            }).await;
                                            store.dispatch(Action::SetCurrentPage { page: Page::Nav }).await;
                                            continue;
                                        }
                                    },
                                    _ => {
                                        store.dispatch(Action::SetStatus { 
                                            message: StatusMessage::Err(format!("Unsupported file type: {extention}")), 
                                            timeout: None,
                                        }).await;
                                        store.dispatch(Action::SetCurrentPage { page: Page::Nav }).await;
                                        continue;
                                    }
                                };

                                {
                                    let mut lifecycle = lifecycle.lock().map_err(|e| anyhow!("Unable to get lifecycle lock: {e}"))?;
                                    lifecycle.suspend()?;
                                }

                                let file_name = store.select(|state: &State| state.file_name.clone()).await;
                                let new_value = editor(&existing_value, &file_name);

                                {
                                    let mut lifecycle = lifecycle.lock().map_err(|e| anyhow!("Unable to get lifecycle lock: {e}"))?;
                                    lifecycle.resume()?;
                                }

                                match new_value {
                                    Ok(new_value) => {
                                        store.dispatch(Action::DocumentReplaceCurrent { value: new_value }).await;
                                        store.dispatch(Action::SetStatus { 
                                            message: StatusMessage::Ok("Successfully edited value".to_owned()), 
                                            timeout: Some(Duration::from_secs(2)) 
                                        }).await;
                                    }
                                    Err(e) => {
                                        store.dispatch(Action::SetStatus { 
                                            message: StatusMessage::Err(format!("Unable to edit value: {e}")), 
                                            timeout: None,
                                        }).await;
                                    }

                                }

                                store.dispatch(Action::SetCurrentPage { page: Page::Nav }).await;
                            },
                            KeyEvent { code: KeyCode::Char('c'), modifiers: KeyModifiers::CONTROL, .. } |
                            KeyEvent { code: KeyCode::Esc, .. } => {
                                store.dispatch(Action::SetCurrentPage { page: Page::Nav }).await;
                            },
                            _ => {}
                        }
                    }
                },
                Page::ExportPrompt => {
                    if let Event::Key(key) = read_event {
                        match key {
                            KeyEvent { code: KeyCode::Char(ch), modifiers: KeyModifiers::SHIFT | KeyModifiers::NONE, .. } => { 
                                let mut current = store
                                    .select(|state: &State| state.export_prompt_state.value.clone())
                                    .await;
                                current.push(ch);
                                store.dispatch(Action::ExportPromptSetValue { value: current }).await;
                            },
                            KeyEvent { code: KeyCode::Backspace, .. } => {
                                let mut current = store
                                    .select(|state: &State| state.export_prompt_state.value.clone())
                                    .await;
                                current.pop();
                                store.dispatch(Action::ExportPromptSetValue { value: current }).await;
                            },
                            KeyEvent { code: KeyCode::Enter, .. } => {
                                let existing_value = store
                                    .select(|state: &State| {
                                        state
                                            .nav_state
                                            .current
                                            .options
                                            .get(state.nav_state.current.selected)
                                            .and_then(|path| path.strip_prefix('#'))
                                            .and_then(|path| path.parse::<JsonPointer<_, _>>().ok())
                                            .and_then(|path| path.get(&state.doc).ok())
                                            .cloned()
                                            .unwrap_or(serde_json::Value::Null)
                                    })
                                    .await;

                                let current_path = store
                                    .select(|state: &State| state.export_prompt_state.value.clone())
                                    .await;
    
                                let extention = PathBuf::from(&current_path);
                                let Some(extention) = extention.extension().map(std::ffi::OsStr::to_string_lossy) else {
                                    store.dispatch(Action::SetStatus { 
                                        message: StatusMessage::Err("Unable to determine file type".to_owned()), 
                                        timeout: None,
                                    }).await;
                                    store.dispatch(Action::SetCurrentPage { page: Page::Nav }).await;
                                    continue;
                                };

                                let existing_value = match extention.as_ref() {
                                    "yaml" | "yml" =>  match serde_yaml::to_string(&existing_value) {
                                        Ok(existing_value) => existing_value,
                                        Err(e) => {
                                            store.dispatch(Action::SetStatus { 
                                                message: StatusMessage::Err(format!("Unable to serialize value: {e}")), 
                                                timeout: None,
                                            }).await;
                                            store.dispatch(Action::SetCurrentPage { page: Page::Nav }).await;
                                            continue;
                                    }
                                    },
                                    "json" => match serde_json::to_string_pretty(&existing_value) {
                                        Ok(existing_value) => existing_value,
                                        Err(e) => {
                                            store.dispatch(Action::SetStatus { 
                                                message: StatusMessage::Err(format!("Unable to serialize value: {e}")), 
                                                timeout: None,
                                            }).await;
                                            store.dispatch(Action::SetCurrentPage { page: Page::Nav }).await;
                                            continue;
                                        }
                                    },
                                    _ => {
                                        store.dispatch(Action::SetStatus { 
                                            message: StatusMessage::Err(format!("Unsupported file type: {extention}")), 
                                            timeout: None,
                                        }).await;
                                        store.dispatch(Action::SetCurrentPage { page: Page::Nav }).await;
                                        continue;
                                    }
                                };

                                let mut file = match File::create(&current_path) {
                                    Ok(file) => file,
                                    Err(e) => {
                                        store.dispatch(Action::SetStatus { 
                                            message: StatusMessage::Err(format!("Unable to create file: {e}")), 
                                            timeout: None,
                                        }).await;
                                        store.dispatch(Action::SetCurrentPage { page: Page::Nav }).await;
                                        continue;
                                    }
                                };

                                match file.write_all(existing_value.as_bytes()) {
                                    Ok(_) => {
                                        store.dispatch(Action::SetStatus { 
                                            message: StatusMessage::Ok("Successfully exported value".to_owned()), 
                                            timeout: Some(Duration::from_secs(2)) 
                                        }).await;
                                    }
                                    Err(e) => {
                                        store.dispatch(Action::SetStatus { 
                                            message: StatusMessage::Err(format!("Unable to write file: {e}")), 
                                            timeout: None,
                                        }).await;
                                    }
                                }

                                store.dispatch(Action::SetCurrentPage { page: Page::Nav }).await;
                            },
                            KeyEvent { code: KeyCode::Char('c'), modifiers: KeyModifiers::CONTROL, .. } |
                            KeyEvent { code: KeyCode::Esc, .. } => {
                                store.dispatch(Action::SetCurrentPage { page: Page::Nav }).await;
                            },
                            _ => {}
                        }
                    }
                },
                Page::Search => {
                    if let Event::Key(key) = read_event {
                        match key {
                            KeyEvent {
                                code: KeyCode::Char(ch),
                                modifiers: KeyModifiers::SHIFT | KeyModifiers::NONE,
                                ..
                            } => {
                                let mut current = store
                                    .select(|state: &State| state.search_state.value.clone())
                                    .await;
                                current.push(ch);
                                store
                                    .dispatch(Action::SearchSetValue { value: current })
                                    .await;
                            }
                            KeyEvent {
                                code: KeyCode::Backspace,
                                ..
                            } => {
                                let mut current = store
                                    .select(|state: &State| state.search_state.value.clone())
                                    .await;
                                current.pop();
                                store
                                    .dispatch(Action::SearchSetValue { value: current })
                                    .await;
                            }
                            KeyEvent {
                                code: KeyCode::Enter,
                                ..
                            } => {
                                let selected_path = store.select(|state: &State| {
                                    state.search_state.filtered_paths.get(state.search_state.selected).cloned()
                                }).await;

                                if let Some(selected_path) = selected_path {
                                    store.dispatch(Action::NavGoto { path: selected_path }).await;
                                }

                                store
                                    .dispatch(Action::SetCurrentPage { page: Page::Nav })
                                    .await;
                            }
                            KeyEvent {
                                code: KeyCode::Char('n'),
                                modifiers: KeyModifiers::CONTROL,
                                ..
                            } => {
                                store.dispatch(Action::SearchDown).await;
                            }
                            KeyEvent {
                                code: KeyCode::Char('p'),
                                modifiers: KeyModifiers::CONTROL,
                                ..
                            } => {
                                store.dispatch(Action::SearchUp).await;
                            }
                            KeyEvent {
                                code: KeyCode::Esc, ..
                            }
                            | KeyEvent {
                                code: KeyCode::Char('c'),
                                modifiers: KeyModifiers::CONTROL,
                                ..
                            } => {
                                let empty_status = store.select(|state: &State| matches!(state.status.message, StatusMessage::Empty)).await;
                                if empty_status {
                                    store.dispatch(Action::SetCurrentPage { page: Page::Nav }).await;
                                    continue;
                                }

                                store.dispatch(Action::SetStatus { 
                                    message: StatusMessage::Empty, 
                                    timeout: None,
                                }).await;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}
