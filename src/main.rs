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
use state::{index::Doc, State};
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let terminal = configure_terminal()?;
    let args: Vec<_> = env::args().collect();
    let file_name = args
        .get(1)
        .ok_or_else(|| anyhow!("Missing filename in argument list."))?;

    let doc = fetch_document(file_name)?;
    let index = Doc::build_from(&doc);

    let initial_state = State::new(doc, index, file_name.clone());
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

    let mut lifecycle = lifecycle.lock().unwrap();
    lifecycle.suspend()?;

    if let Err(err) = result {
        println!("{err}");
    }

    Ok(())
}
