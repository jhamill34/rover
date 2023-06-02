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

pub fn configure_terminal() -> anyhow::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    Ok(terminal)
}

pub fn ui<B: Backend>(frame: &mut Frame<B>, state: &State) {
    match state.current_page {
        Page::Nav => nav_ui(frame, state),
        Page::Search => search_ui(frame, state),
    }
}

pub fn nav_ui<B: Backend>(frame: &mut Frame<B>, state: &State) {
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
            .and_then(|value| serde_json::to_string_pretty(value).ok());
        if let Some(selected_path) = selected_path {
            let text: Vec<Spans> = selected_path
                .split('\n')
                .into_iter()
                .map(|line| Spans::from(Span::from(line)))
                .collect();

            let next = Block::default().title("Preview").borders(Borders::ALL);

            let next = Paragraph::new(text).block(next);

            frame.render_widget(next, chunks[2]);
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

fn search_ui<B: Backend>(frame: &mut Frame<B>, state: &State) {
    let chunks = Layout::default()
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

    let input = Block::default().title("Input").borders(Borders::ALL);

    let input_text = Text::raw(state.search_state.value.clone());
    let input_paragraph = Paragraph::new(input_text).block(input);
    frame.render_widget(input_paragraph, chunks[0]);

    let results = Block::default().title("Results").borders(Borders::ALL);
    frame.render_widget(results, chunks[1]);
}
