use crossterm::event::Event;
use tui::text::Spans;

use crate::{game::Game, IntAction};

pub mod expectimax;
pub mod random;
pub mod user;

pub trait Agent {
    fn new(game: Game) -> Self
    where
        Self: Sized;

    fn get_game(&self) -> &Game;
    fn get_input(&mut self, _: &Event) -> IntAction {
        IntAction::Continue
    }
    fn next_move(&mut self);
    fn messages(&self) -> Vec<Spans>;
}
