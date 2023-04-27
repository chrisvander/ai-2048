use std::{io, thread, time::Duration};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Widget},
    Frame, Terminal,
};

struct Game {
    state: u64,
}

impl Game {
    fn new() -> Self {
        Game { state: 0 }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Min(0), Constraint::Length(40)].as_ref())
        .split(f.size());
    let block = Block::default().title("Game").borders(Borders::ALL);
    f.render_widget(block, chunks[0]);
    let block = Block::default().title("Menu").borders(Borders::ALL);
    f.render_widget(block, chunks[1]);
}

fn main() -> Result<(), io::Error> {
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.clear()?;
    terminal.draw(ui)?;
    thread::sleep(Duration::from_millis(5000));

    let game = Game::new();

    Ok(())
}
