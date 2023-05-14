use enum_map::EnumMap;

use crate::game::{Game, Move};

use super::Agent;

type MoveScores = EnumMap<Move, usize>;
pub struct Expectimax {
    depth: usize,
    scores: MoveScores,
}

impl Agent for Expectimax {
    fn new() -> Self
    where
        Self: Sized,
    {
        Expectimax {
            depth: 8,
            scores: MoveScores::default(),
        }
    }

    fn get_move(&mut self, game: &Game) -> Move {
        todo!()
    }

    fn tui_messages(&self) -> Vec<tui::text::Spans> {
        todo!()
    }
}
