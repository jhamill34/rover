#![allow(clippy::too_many_lines)]

//!

use std::io;

use crossterm::{
    event::EnableMouseCapture,
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};

use crate::state::{Page, State};

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
    }
}

///
pub fn nav<B: Backend>(frame: &mut Frame<B>, state: &State) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(frame.size().height - 3),
            ]
            .as_ref(),
        )
        .split(frame.size());

    let location = Block::default().title("Location").borders(Borders::ALL);

    let current_path = state
        .nav_state
        .current
        .options
        .get(state.nav_state.current.selected)
        .cloned()
        .unwrap_or_default();
    let location = Paragraph::new(Text::raw(current_path)).block(location);

    frame.render_widget(location, main_chunks[0]);

    let chunks = Layout::default()
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
        .split(main_chunks[1]);

    //
    // Previous Frame
    //

    if let Some(prev) = state.nav_state.history.last() {
        let prev_items: Vec<ListItem> = prev
            .options
            .iter()
            .filter_map(|opt| {
                opt.split('/')
                    .last()
                    .map(|name| name.replace("~1", "/"))
                    .map(|name| ListItem::new(Text::raw(name)))
            })
            .collect();
        let mut prev_items_state = ListState::default();
        prev_items_state.select(Some(prev.selected));

        let prev = Block::default().title("Previous").borders(Borders::ALL);

        let prev = List::new(prev_items)
            .block(prev)
            .highlight_style(Style::default().bg(Color::White).fg(Color::Black));

        frame.render_stateful_widget(prev, chunks[0], &mut prev_items_state);
    } else {
        let prev = Block::default().title("Previous").borders(Borders::ALL);
        frame.render_widget(prev, chunks[0]);
    }

    //
    // Current Frame
    //
    let current = Block::default().title("Current").borders(Borders::ALL);

    let current_items: Vec<ListItem> = state
        .nav_state
        .current
        .options
        .iter()
        .filter_map(|opt| {
            opt.split('/')
                .last()
                .map(|name| name.replace("~1", "/"))
                .map(|name| ListItem::new(Text::raw(name)))
        })
        .collect();
    let mut current_items_state = ListState::default();
    current_items_state.select(Some(state.nav_state.current.selected));

    let current = List::new(current_items)
        .block(current)
        .highlight_style(Style::default().bg(Color::White).fg(Color::Black));
    frame.render_stateful_widget(current, chunks[1], &mut current_items_state);

    //
    // Next / Preview Frame
    //
    let selected_path = state
        .nav_state
        .current
        .options
        .get(state.nav_state.current.selected);

    if let Some(selected_path) = selected_path {
        let selected_path = selected_path
            .strip_prefix('#')
            .and_then(|path| path.parse::<json_pointer::JsonPointer<_, _>>().ok())
            .and_then(|path| path.get(&state.doc).ok())
            .and_then(|value| serde_yaml::to_string(value).ok());
        if let Some(selected_path) = selected_path {
            let text: Vec<Spans> = selected_path
                .split('\n')
                .into_iter()
                .map(|line| Spans::from(Span::from(line)))
                .collect();

            let preview = Block::default().title("Preview").borders(Borders::ALL);

            let preview = Paragraph::new(text).block(preview);

            frame.render_widget(preview, chunks[2]);
        } else {
            let next = Block::default().title("Preview").borders(Borders::ALL);

            frame.render_widget(next, chunks[2]);
        }
    } else {
        // Selected not found...
        let next = Block::default().title("Preview").borders(Borders::ALL);
        frame.render_widget(next, chunks[2]);
    }
}

///
fn search<B: Backend>(frame: &mut Frame<B>, state: &State) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(frame.size().height - 3),
            ]
            .as_ref(),
        )
        .split(frame.size());

    let selected_path = state
        .search_state
        .filtered_paths
        .get(
            state.search_state.selected
        );

    let input = Block::default().title("Input").borders(Borders::ALL);

    let input_text = Text::raw(state.search_state.value.clone());
    let input_paragraph = Paragraph::new(input_text).block(input);
    frame.render_widget(input_paragraph, chunks[0]);
    
    let current_path = Block::default().borders(Borders::ALL);
    let current_path = Paragraph::new(Text::raw(selected_path.cloned().unwrap_or_default()))
        .block(current_path);
    frame.render_widget(current_path, chunks[1]);

    let result_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints(
            [
                Constraint::Ratio(1, 2),
                Constraint::Ratio(1, 2),
            ].as_ref()
        )
        .split(chunks[2]);

    let filtered_items: Vec<ListItem> = state
        .search_state
        .filtered_paths
        .iter()
        .map(|path| {
            ListItem::new(Text::raw(path))
        })
        .collect();

    let title = format!("Paths ({}/{})", filtered_items.len(), state.index.adj_list.len());
    let search_paths = Block::default().title(title).borders(Borders::ALL);
    let search_paths = List::new(filtered_items)
        .highlight_symbol("> ")
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::White)
        )
        .block(search_paths);

    let mut search_paths_selected = ListState::default();
    search_paths_selected.select(Some(state.search_state.selected));

    frame.render_stateful_widget(search_paths, result_chunks[0], &mut search_paths_selected);
    
    let selected_path = state
        .search_state
        .filtered_paths
        .get(
            state.search_state.selected
        );

    if let Some(selected_path) = selected_path {
        let selected_path = selected_path
            .strip_prefix('#')
            .and_then(|path| path.parse::<json_pointer::JsonPointer<_, _>>().ok())
            .and_then(|path| path.get(&state.doc).ok())
            .and_then(|value| serde_yaml::to_string(value).ok());
        if let Some(selected_path) = selected_path {
            let text: Vec<Spans> = selected_path
                .split('\n')
                .into_iter()
                .map(|line| Spans::from(Span::from(line)))
                .collect();

            let preview = Block::default().title("Preview").borders(Borders::ALL);

            let preview = Paragraph::new(text).block(preview);

            frame.render_widget(preview, result_chunks[1]);
        } else {
            let next = Block::default().title("Preview").borders(Borders::ALL);

            frame.render_widget(next, result_chunks[1]);
        }
    } else {
        // Selected not found...
        let next = Block::default().title("Preview").borders(Borders::ALL);
        frame.render_widget(next, result_chunks[1]);
    }
}
