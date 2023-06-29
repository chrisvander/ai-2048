use enum_map::EnumMap;

use crate::game::{Game, Move};

use super::{Agent, TuiAgent};

type MoveScores = EnumMap<Move, usize>;
pub struct Expectimax {
    game: Game,
    depth: usize,
    scores: MoveScores,
}

impl Expectimax {
    pub fn new(game: Game) -> Self
    where
        Self: Sized,
    {
        Expectimax {
            game,
            depth: 8,
            scores: MoveScores::default(),
        }
    }
}

impl Agent for Expectimax {
    fn next_move(&mut self) {
        todo!()
    }

    fn get_game(&self) -> &Game {
        &self.game
    }
}

impl TuiAgent for Expectimax {
    fn messages(&self) -> Vec<tui::text::Spans> {
        todo!()
    }
}
