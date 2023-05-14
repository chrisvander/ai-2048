use crate::agent::Agent;
use crate::game::{Game, Move};

use enum_map::EnumMap;
use rayon::prelude::*;
use strum::IntoEnumIterator;
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};

// Basic random agent, randomly selects an action and takes the move.
pub struct RandomAgent;
impl Agent for RandomAgent {
    fn new() -> Self {
        RandomAgent {}
    }

    fn get_move(&mut self, _: &Game) -> Move {
        match fastrand::usize(0..4) {
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

// Random tree search, simulate many games per move and then select the move based on the highest
// average score.
type MoveScores = EnumMap<Move, usize>;
pub struct RandomTree {
    sim_count: usize,
    metric: RandomTreeMetric,
    last_scores: MoveScores,
}

pub enum RandomTreeMetric {
    AvgScore,
    AvgMoves,
}

impl RandomTree {
    pub fn new_with(metric: RandomTreeMetric) -> Self {
        let mut ag = RandomTree::new();
        ag.metric = metric;
        ag
    }
}

impl Agent for RandomTree {
    fn new() -> Self {
        RandomTree {
            sim_count: 1000,
            metric: RandomTreeMetric::AvgScore,
            last_scores: MoveScores::default(),
        }
    }

    fn get_move(&mut self, game: &Game) -> Move {
        let mut scores = MoveScores::default();
        for game_move in Move::iter() {
            let mut sim_game = game.clone();
            let test = sim_game.update(game_move);
            if !test {
                continue;
            }

            let score = vec![0; self.sim_count]
                .par_iter()
                .map(|_| {
                    let mut sim_game = game.clone();
                    sim_game.update(game_move);
                    let game = simulate_random_game(sim_game);
                    match self.metric {
                        RandomTreeMetric::AvgMoves => game.get_num_moves().clone(),
                        RandomTreeMetric::AvgScore => game.get_score().clone(),
                    }
                })
                .sum::<usize>();

            scores[game_move] = score;
        }

        self.last_scores = scores;
        scores.iter().max_by_key(|(_, score)| *score).unwrap().0
    }

    fn tui_messages(&self) -> Vec<Spans> {
        let highest_move = self
            .last_scores
            .iter()
            .max_by_key(|(_, score)| *score)
            .unwrap()
            .0;

        let mut score_spans = Move::iter()
            .map(|m| {
                if m == highest_move {
                    Spans::from(Span::styled(
                        format!("{}: {}", m, self.last_scores[m] / self.sim_count),
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .bg(Color::Green),
                    ))
                } else {
                    Spans::from(format!("{}: {}", m, self.last_scores[m] / self.sim_count))
                }
            })
            .collect::<Vec<_>>();

        let mut msgs = vec![
            Spans::from(Span::styled(
                "Random Tree Search",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Spans::from(format!(
                "Taking the average of {} simulations, per move, to determine the next best move. Comparing by {}.",
                self.sim_count,
                match self.metric {
                    RandomTreeMetric::AvgScore=>"highest score",
                    RandomTreeMetric::AvgMoves =>"number of moves"
                }
            )),
            Spans::from(""),
        ];
        msgs.append(&mut score_spans);
        msgs
    }
}
