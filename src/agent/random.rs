use crate::game::{Agent, Game, Move};

pub struct RandomAgent {
    last_move: Move,
}

impl Default for RandomAgent {
    fn default() -> Self {
        RandomAgent {
            last_move: Move::Up,
        }
    }
}


impl Agent for RandomAgent {
    fn get_move(&mut self, _: &Game) -> Move {
        self.last_move = match rand::random::<usize>() % 4 {
            0 => Move::Up,
            1 => Move::Down,
            2 => Move::Left,
            3 => Move::Right,
            _ => unreachable!(),
        };
        self.last_move
    }

    fn log_messages(&self) -> String {
        format!(
            "Performing random actions. Last move: {:#?}",
            self.last_move
        )
    }
}
