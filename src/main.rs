use std::error::Error;

pub mod agent;
pub mod game;
mod tui;

fn main() -> Result<(), Box<dyn Error>> {
    tui::start()?;
    Ok(())
}
