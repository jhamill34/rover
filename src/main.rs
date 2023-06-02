use std::{
    env,
    sync::{Arc, Mutex},
};

use anyhow::anyhow;
use events::event_listener;
use lifecycle::ApplicationLifecycle;
use redux_rs::Store;
use state::{index::DocIndex, State};
use ui::configure_terminal;
use util::fetch_document;

mod action;
mod events;
mod lifecycle;
mod reducer;
mod state;
mod ui;
mod util;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let terminal = configure_terminal()?;
    let args: Vec<_> = env::args().collect();
    let file_name = args
        .get(1)
        .ok_or_else(|| anyhow!("Missing filename in argument list."))?;

    let doc = fetch_document(file_name)?;
    let index = DocIndex::build_from(&doc);

    let initial_state = State::new(doc, index, file_name.clone());
    let mut lifecycle = ApplicationLifecycle::new(terminal);
    lifecycle.refresh(&initial_state)?;

    let lifecycle = Arc::new(Mutex::new(lifecycle));
    let store = Store::new_with_state(reducer::reducer, initial_state);

    let ui_lifecycle = Arc::clone(&lifecycle);
    store
        .subscribe(move |state: &State| {
            let mut ui_lifecycle = ui_lifecycle.lock().unwrap();
            ui_lifecycle.refresh(state).unwrap();
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
