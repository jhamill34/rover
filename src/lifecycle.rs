//!

use std::io;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{backend::Backend, layout::Rect, Terminal};

use crate::{state::State, ui};

///
pub struct Application<B>
where
    B: Backend + io::Write,
{
    ///
    terminal: Terminal<B>,
}

impl<B> Application<B>
where
    B: Backend + io::Write,
{
    ///
    pub fn new(terminal: Terminal<B>) -> Self {
        Self { terminal }
    }

    ///
    pub fn suspend(&mut self) -> anyhow::Result<()> {
        disable_raw_mode()?;

        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;

        self.terminal.show_cursor()?;
        Ok(())
    }

    ///
    pub fn resume(&mut self) -> anyhow::Result<()> {
        enable_raw_mode()?;

        execute!(
            self.terminal.backend_mut(),
            EnterAlternateScreen,
            EnableMouseCapture
        )?;

        self.terminal.clear()?;
        Ok(())
    }

    ///
    pub fn refresh(&mut self, state: &State) -> anyhow::Result<()> {
        self.terminal.draw(|f| ui::ui(f, state))?;

        Ok(())
    }

    ///
    pub fn resize(&mut self, width: u16, height: u16) -> anyhow::Result<()> {
        self.terminal.resize(Rect::new(0, 0, width, height))?;

        Ok(())
    }
}
