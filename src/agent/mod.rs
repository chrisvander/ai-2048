use tui::text::Spans;

use crate::game::{Game, Move};

pub mod random;

pub trait Agent {
    fn new() -> Self
    where
        Self: Sized;

    fn get_move(&mut self, game: &Game) -> Move;
    fn make_move(&mut self, game: &mut Game) {
        game.update(self.get_move(game));
    }

    fn tui_messages(&self) -> Vec<Spans>;
}
