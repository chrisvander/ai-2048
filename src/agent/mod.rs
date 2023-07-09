use crossterm::event::Event;
use tui::text::Spans;

use crate::{game::Game, tui::IntAction};

pub mod expectimax;
pub mod random;
pub mod user;

pub trait Agent {
    fn get_game(&self) -> &Game;
    fn next_move(&mut self);
}

pub trait TuiAgent: Agent {
    fn get_input(&mut self, _: &Event) -> IntAction {
        IntAction::Continue
    }
    fn messages(&self) -> Vec<Spans>;
}
