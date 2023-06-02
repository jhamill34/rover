#![allow(clippy::too_many_lines)]

//!

extern crate alloc;
use alloc::sync::Arc;

use core::time::Duration;

use std::{
    io,
    sync::Mutex,
};

use crossterm::event::{self, poll, Event, KeyCode, KeyEvent, KeyModifiers};
use json_pointer::JsonPointer;
use redux_rs::{Reducer, Store};
use tui::backend::Backend;

use crate::{
    action::Action,
    lifecycle::Application,
    state::{Page, State},
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
        if poll(Duration::from_millis(100))? {
            let read_event = event::read()?;
            if let Event::Resize(height, width) = read_event {
                let lifecycle = Arc::clone(&lifecycle);
                store
                    .select(move |state: &State| -> anyhow::Result<()> {
                        let mut lifecycle = lifecycle.lock().unwrap();
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
                                code: KeyCode::Char('l'),
                                ..
                            } => {
                                let children = store
                                    .select(|state: &State| {
                                        state
                                            .nav_state
                                            .current
                                            .options
                                            .get(state.nav_state.current.selected)
                                            .and_then(|path| state.index.adj_list.get(path))
                                            .map_or(0, Vec::len)
                                    })
                                    .await;

                                if children > 0 {
                                    store.dispatch(Action::NavSelect).await;
                                }
                            }
                            KeyEvent {
                                code: KeyCode::Char('e'),
                                ..
                            } => {
                                {
                                    let mut lifecycle = lifecycle.lock().unwrap();
                                    lifecycle.suspend()?;
                                }

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

                                let new_value = editor(&existing_value)?;

                                {
                                    let mut lifecycle = lifecycle.lock().unwrap();
                                    lifecycle.resume()?;
                                }

                                store
                                    .dispatch(Action::DocumentReplaceCurrent { value: new_value })
                                    .await;
                            }
                            KeyEvent {
                                code: KeyCode::Char('s'),
                                modifiers: KeyModifiers::CONTROL,
                                ..
                            } => {
                                let file_name =
                                    store.select(|state: &State| state.file_name.clone()).await;
                                let doc = store.select(|state: &State| state.doc.clone()).await;
                                save_doc(&file_name, &doc)?;
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
                                store
                                    .dispatch(Action::SetCurrentPage { page: Page::Search })
                                    .await;
                            }
                            KeyEvent {
                                code: KeyCode::Char('q') | KeyCode::Esc,
                                ..
                            }
                            | KeyEvent {
                                code: KeyCode::Char('c'),
                                modifiers: KeyModifiers::CONTROL,
                                ..
                            } => return Ok(()),
                            _ => {}
                        }
                    }
                }
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
                                store
                                    .dispatch(Action::SearchSetValue {
                                        value: String::new(),
                                    })
                                    .await;
                                store
                                    .dispatch(Action::SetCurrentPage { page: Page::Nav })
                                    .await;
                            }
                            KeyEvent {
                                code: KeyCode::Esc, ..
                            }
                            | KeyEvent {
                                code: KeyCode::Char('c'),
                                modifiers: KeyModifiers::CONTROL,
                                ..
                            } => return Ok(()),
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}
