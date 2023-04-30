use crate::agent::Agent;
use crate::game::{Game, Move};

use enum_map::EnumMap;
use strum::IntoEnumIterator;
use tui::style::{Modifier, Style};
use tui::text::{Span, Spans};

// Basic random agent, randomly selects an action and takes the move.
pub struct RandomAgent;
impl Agent for RandomAgent {
    fn new() -> Self {
        RandomAgent {}
    }

    fn get_move(&mut self, _: &Game) -> Move {
        match rand::random::<usize>() % 4 {
            0 => Move::Up,
            1 => Move::Down,
            2 => Move::Left,
            3 => Move::Right,
            _ => unreachable!(),
        }
    }

    fn tui_messages(&self) -> Vec<Spans> {
        vec![Spans::from("Performing random actions.")]
    }
}

// Use a RandomAgent to simulate a full game from a starting point
fn simulate_random_game(mut game: Game) -> Game {
    let mut agent = RandomAgent::new();
    while !game.game_over() {
        agent.make_move(&mut game);
    }
    game
}

// Random tree search, simulate many games per move and then select the move based on some metric.
type MoveScores = EnumMap<Move, usize>;
pub struct RandomTree {
    sim_count: usize,
    last_scores: MoveScores,
}

impl Agent for RandomTree {
    fn new() -> Self {
        RandomTree {
            sim_count: 500,
            last_scores: MoveScores::default(),
        }
    }

    fn get_move(&mut self, game: &Game) -> Move {
        let mut scores = MoveScores::default();
        for game_move in Move::iter() {
            let mut sim_game = game.clone();
            sim_game.update(game_move);
            let score_sum = vec![0; self.sim_count]
                .iter()
                .map(|_| simulate_random_game(sim_game.clone()).get_score().clone())
                .fold(0, |a, b| a + b);

            scores[game_move] = score_sum / self.sim_count;
        }

        self.last_scores = scores;
        scores.iter().max_by_key(|(_, score)| *score).unwrap().0
    }

    fn tui_messages(&self) -> Vec<Spans> {
        vec![
            Spans::from(Span::styled(
                "Random Tree Search",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Spans::from(Span::styled(
                format!(
                    "Taking the average of {} simulations, per move, to determine the next best move.",
                    self.sim_count
                ),
                Style::default().add_modifier(Modifier::ITALIC),
            )),
            Spans::from(""),
            Spans::from(format!("Up: {}", self.last_scores[Move::Up])),
            Spans::from(format!("Down: {}", self.last_scores[Move::Down])),
            Spans::from(format!("Left: {}", self.last_scores[Move::Left])),
            Spans::from(format!("Right: {}", self.last_scores[Move::Right])),
        ]
    }
}
