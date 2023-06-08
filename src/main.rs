#![warn(clippy::restriction, clippy::pedantic)]
#![allow(
    clippy::blanket_clippy_restriction_lints,
    clippy::mod_module_files,
    clippy::self_named_module_files,
    clippy::implicit_return,
    clippy::shadow_reuse,
    clippy::match_ref_pats,
    clippy::shadow_unrelated,
    clippy::shadow_same,
    clippy::print_stdout,
    // clippy::too_many_lines
)]

//!

extern crate alloc;
use alloc::sync::Arc;

use std::{
    env,
    sync::Mutex,
};

use anyhow::anyhow;
use events::event_listener;
use lifecycle::Application;
use redux_rs::Store;
use state::State;
use ui::configure_terminal;
use util::fetch_document;

mod action;
mod events;
mod lifecycle;
mod reducer;
mod state;
mod ui;
mod util;
mod search;
mod value;
mod pointer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<_> = env::args().collect();
    let file_name = args
        .get(1)
        .ok_or_else(|| anyhow!("Missing filename in argument list."))?;

    let doc = fetch_document(file_name)?;
    let initial_state = State::new(doc, file_name.clone());

    // 
    //  !!!PANICS beyond this point will ruin the terminal state!!!
    //
    let terminal = configure_terminal()?;
    let mut lifecycle = Application::new(terminal);
    lifecycle.refresh(&initial_state)?;

    let lifecycle = Arc::new(Mutex::new(lifecycle));
    let store = Store::new_with_state(reducer::reducer, initial_state);

    let ui_lifecycle = Arc::clone(&lifecycle);
    store
        .subscribe(move |state: &State| {
            if let Ok(mut ui_lifecycle) = ui_lifecycle.lock() {
                ui_lifecycle.refresh(state).ok();
            }
        })
        .await;

    let result = tokio::spawn(event_listener(store, Arc::clone(&lifecycle))).await?;

    // At this point we just really want to fix the terminal if we can 
    let mut lifecycle = match lifecycle.lock() {
        Ok(lock) => lock,
        Err(err) => err.into_inner()
    };
    lifecycle.suspend()?;

    if let Err(err) = result {
        println!("{err}");
    }

    Ok(())
}

