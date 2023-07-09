use crossterm::event::Event;
use enum_map::EnumMap;
use tui::text::Spans;

use crate::{
    game::{Game, Move},
    tui::IntAction,
};

pub mod expectimax;
pub mod random;
pub mod user;

pub trait Agent {
    fn get_game(&self) -> &Game;
    fn next_move(&self) -> Move;
    fn make_move(&mut self); 
}

pub trait TuiAgent: Agent {
    fn get_input(&mut self, _: &Event) -> IntAction {
        IntAction::Continue
    }
    fn messages(&self) -> Vec<Spans>;
}

pub type MoveScores = EnumMap<Move, usize>;
pub trait MaxMove {
    fn max_move(&self) -> Move;
}

impl MaxMove for MoveScores {
    fn max_move(&self) -> Move {
        self.iter().max_by_key(|(_, score)| *score).unwrap().0
    }
}
