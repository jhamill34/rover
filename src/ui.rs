#![allow(clippy::too_many_lines)]

//!

use alloc::borrow::Cow;
use std::{ffi::OsStr, io, path::PathBuf};

use crossterm::{
    event::EnableMouseCapture,
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};

use crate::{
    pointer::ValuePointer,
    state::{Page, State, Step},
    value::Value,
};

///
pub fn configure_terminal() -> anyhow::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    Ok(terminal)
}

///
pub fn ui<B: Backend>(frame: &mut Frame<B>, state: &State) {
    match state.current_page {
        Page::Nav => nav(frame, state),
        Page::Search => search(frame, state),
        Page::ImportPrompt => import_prompt(frame, state),
        Page::ExportPrompt => export_prompt(frame, state),
    }
}

///
pub fn import_prompt<B: Backend>(frame: &mut Frame<B>, state: &State) {
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Ratio(1, 4),
                Constraint::Ratio(2, 4),
                Constraint::Ratio(1, 4),
            ]
            .as_ref(),
        )
        .split(frame.size());

    if let Some(column) = horizontal.get(1) {
        let vertical_margin = column.height.saturating_sub(4).saturating_div(2);

        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(vertical_margin),
                    Constraint::Length(4),
                    Constraint::Length(vertical_margin),
                ]
                .as_ref(),
            )
            .split(*column);

        if let Some(row) = vertical.get(1) {
            let prompt = Block::default().title("Import").borders(Borders::ALL);

            let text = Paragraph::new(Text::from(vec![
                Spans::from(Span::styled(
                    "  Select the file path to import into the current document:",
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::White),
                )),
                Spans::from(vec![
                    Span::raw(" > "),
                    Span::raw(state.import_prompt_state.value.clone()),
                    Span::styled(
                        "_",
                        Style::default()
                        .bg(Color::White)
                    ),
                ])
            ]))
            .block(prompt);

            frame.render_widget(text, *row);
        }
    }
}

///
pub fn export_prompt<B: Backend>(frame: &mut Frame<B>, state: &State) {
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Ratio(1, 4),
                Constraint::Ratio(2, 4),
                Constraint::Ratio(1, 4),
            ]
            .as_ref(),
        )
        .split(frame.size());

    if let Some(column) = horizontal.get(1) {
        let vertical_margin = column.height.saturating_sub(4).saturating_div(2);

        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(vertical_margin),
                    Constraint::Length(4),
                    Constraint::Length(vertical_margin),
                ]
                .as_ref(),
            )
            .split(*column);

        if let Some(row) = vertical.get(1) {
            let prompt = Block::default().title("Export").borders(Borders::ALL);

            let text = Paragraph::new(Text::from(vec![
                Spans::from(Span::styled(
                    "  Select the file path to export the current document to:",
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::White),
                )),
                Spans::from(vec![
                    Span::raw(" > "),
                    Span::raw(state.export_prompt_state.value.clone()),
                    Span::styled(
                        "_",
                        Style::default()
                        .bg(Color::White)
                    ),
                ])
            ]))
            .block(prompt);

            frame.render_widget(text, *row);
        }
    }
}

///
pub fn nav<B: Backend>(frame: &mut Frame<B>, state: &State) {
    let mut main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(frame.size().height.saturating_sub(8)),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(frame.size())
        .into_iter();

    if let Some(rect) = main_chunks.next() {
        let location = current_path(state);
        frame.render_widget(location, rect);
    }

    if let Some(rect) = main_chunks.next() {
        let mut chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints(
                [
                    Constraint::Ratio(1, 3),
                    Constraint::Ratio(1, 3),
                    Constraint::Ratio(1, 3),
                ]
                .as_ref(),
            )
            .split(rect)
            .into_iter();

        if let Some(rect) = chunks.next() {
            let previous = Block::default().title("Previous").borders(Borders::ALL);
            if let Some(prev) = state.nav_state.history.last() {
                let (list, mut state) = step_list(prev, &state.doc, previous);
                frame.render_stateful_widget(list, rect, &mut state);
            } else {
                frame.render_widget(previous, rect);
            }
        }

        if let Some(rect) = chunks.next() {
            let current = Block::default().title("Current").borders(Borders::ALL);
            let (current, mut current_state) =
                step_list(&state.nav_state.current, &state.doc, current);
            frame.render_stateful_widget(current, rect, &mut current_state);
        }

        if let Some(rect) = chunks.next() {
            let selected_path = state
                .nav_state
                .current
                .path
                .parse::<ValuePointer>()
                .ok()
                .and_then(|path| path.get(&state.doc).ok());

            let extension = PathBuf::from(&state.file_name);
            let extension = extension
                .extension()
                .map_or(Cow::Borrowed("json"), OsStr::to_string_lossy);

            if let Some(selected_path) = selected_path {
                let selected_path = match selected_path {
                    &Value::Array(ref array) => array.get(state.nav_state.current.selected),
                    &Value::Object(ref object) => object
                        .get_index(state.nav_state.current.selected)
                        .map(|(_, value)| value),
                    &Value::Null | &Value::Bool(_) | &Value::String(_) | &Value::Number(_) => None,
                };
                let selected_path = selected_path.and_then(|value| match extension.as_ref() {
                    "yaml" | "yml" => serde_yaml::to_string(value).ok(),
                    "json" => serde_json::to_string_pretty(value).ok(),
                    _ => None,
                });

                if let Some(selected_path) = selected_path {
                    let text: Vec<Spans> = selected_path
                        .split('\n')
                        .into_iter()
                        .map(|line| Spans::from(Span::from(line)))
                        .collect();

                    let preview = Block::default().title("Preview").borders(Borders::ALL);

                    let preview = Paragraph::new(text).block(preview);

                    frame.render_widget(preview, rect);
                } else {
                    let next = Block::default().title("Preview").borders(Borders::ALL);

                    frame.render_widget(next, rect);
                }
            } else {
                // Selected not found...
                let next = Block::default().title("Preview").borders(Borders::ALL);
                frame.render_widget(next, rect);
            }
        }
    }

    if let Some(rect) = main_chunks.next() {
        frame.render_widget(status(state), rect);
    }
}

///
fn status<'status>(state: &State) -> Paragraph<'status> {
    let location = Block::default().title("Status").borders(Borders::ALL);

    let message = match &state.status.message {
        &crate::state::StatusMessage::Ok(ref msg) => Text::from(Spans::from(Span::styled(
            format!("[Ok] {msg}"),
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Green),
        ))),
        &crate::state::StatusMessage::Warn(ref msg) => Text::from(Spans::from(Span::styled(
            format!("[Warning] {msg}"),
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Yellow),
        ))),
        &crate::state::StatusMessage::Err(ref msg) => Text::from(Spans::from(Span::styled(
            format!("[Error] {msg}"),
            Style::default().add_modifier(Modifier::BOLD).fg(Color::Red),
        ))),
        &crate::state::StatusMessage::Empty => Text::raw(""),
    };

    Paragraph::new(message).block(location)
}

///
fn current_path<'path>(state: &State) -> Paragraph<'path> {
    let location = Block::default().title("Location").borders(Borders::ALL);

    Paragraph::new(Text::raw(state.nav_state.current.path.clone())).block(location)
}

///
fn step_list<'list>(step: &Step, doc: &Value, parent: Block<'list>) -> (List<'list>, ListState) {
    let prev_items: Vec<ListItem> = step
        .path
        .parse::<ValuePointer>()
        .ok()
        .and_then(|pointer| pointer.get(doc).ok())
        .map(|value| match value {
            &Value::Array(ref array) => array
                .iter()
                .enumerate()
                .map(|(i, _)| i.to_string())
                .collect(),
            &Value::Object(ref object) => object.keys().cloned().collect(),
            &Value::Null | &Value::Bool(_) | &Value::String(_) | &Value::Number(_) => vec![],
        })
        .map(|name| {
            name.into_iter()
                .map(|name| ListItem::new(Text::raw(name)))
                .collect()
        })
        .unwrap_or_default();

    let mut prev_items_state = ListState::default();
    prev_items_state.select(Some(step.selected));

    let prev = List::new(prev_items)
        .block(parent)
        .highlight_symbol(" > ")
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                // .bg(Color::White)
                .fg(Color::Yellow),
        );

    (prev, prev_items_state)
}

///
fn search<B: Backend>(frame: &mut Frame<B>, state: &State) {
    let mut chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(frame.size().height.saturating_sub(11)),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(frame.size())
        .into_iter();

    if let Some(rect) = chunks.next() {
        let input = Block::default().title("Input").borders(Borders::ALL);
        let input_text = Spans::from(vec![
            Span::raw(" > "),
            Span::raw(state.search_state.value.clone()),
            Span::styled(
                "_",
                Style::default()
                    .bg(Color::White)
            ),
        ]);
        let input_paragraph = Paragraph::new(input_text).block(input);
        frame.render_widget(input_paragraph, rect);
    }

    if let Some(rect) = chunks.next() {
        let selected_path = state
            .search_state
            .filtered_paths
            .get(state.search_state.selected);
        let current_path = Block::default().borders(Borders::ALL);
        let current_path = Paragraph::new(Text::raw(selected_path.cloned().unwrap_or_default()))
            .block(current_path);
        frame.render_widget(current_path, rect);
    }

    if let Some(rect) = chunks.next() {
        let mut result_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)].as_ref())
            .split(rect)
            .into_iter();

        if let Some(rect) = result_chunks.next() {
            let filtered_items: Vec<ListItem> = state
                .search_state
                .filtered_paths
                .iter()
                .map(|path| ListItem::new(Text::raw(path)))
                .collect();

            let title = format!(
                "Paths ({}/{})",
                filtered_items.len(),
                state.search_state.all_paths.len()
            );
            let search_paths = Block::default().title(title).borders(Borders::ALL);
            let search_paths = List::new(filtered_items)
                .highlight_symbol("> ")
                .highlight_style(
                    Style::default()
                        // .bg(Color::White)
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::Yellow),
                )
                .block(search_paths);

            let mut search_paths_selected = ListState::default();
            search_paths_selected.select(Some(state.search_state.selected));

            frame.render_stateful_widget(search_paths, rect, &mut search_paths_selected);
        }

        if let Some(rect) = result_chunks.next() {
            let selected_path = state
                .search_state
                .filtered_paths
                .get(state.search_state.selected);

            let extension = PathBuf::from(&state.file_name);
            let extension = extension
                .extension()
                .map_or(Cow::Borrowed("json"), OsStr::to_string_lossy);

            if let Some(selected_path) = selected_path {
                let selected_path = selected_path
                    .strip_prefix('#')
                    .and_then(|path| path.parse::<ValuePointer>().ok())
                    .and_then(|path| path.get(&state.doc).ok())
                    .and_then(|value| match extension.as_ref() {
                        "yaml" | "yml" => serde_yaml::to_string(value).ok(),
                        "json" => serde_json::to_string_pretty(value).ok(),
                        _ => None,
                    });
                if let Some(selected_path) = selected_path {
                    let text: Vec<Spans> = selected_path
                        .split('\n')
                        .into_iter()
                        .map(|line| Spans::from(Span::from(line)))
                        .collect();

                    let preview = Block::default().title("Preview").borders(Borders::ALL);

                    let preview = Paragraph::new(text).block(preview);

                    frame.render_widget(preview, rect);
                } else {
                    let next = Block::default().title("Preview").borders(Borders::ALL);

                    frame.render_widget(next, rect);
                }
            } else {
                // Selected not found...
                let next = Block::default().title("Preview").borders(Borders::ALL);
                frame.render_widget(next, rect);
            }
        }
    }

    if let Some(rect) = chunks.next() {
        frame.render_widget(status(state), rect);
    }
}
