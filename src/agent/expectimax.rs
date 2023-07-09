use rayon::prelude::*;
use strum::IntoEnumIterator;
use tui::{
    style::{Color, Modifier, Style},
    text::{Span, Spans},
};

use crate::game::{Game, Move};

use super::{
    random::{RandomTree, RandomTreeMetric},
    Agent, MaxMove, MoveScores, TuiAgent,
};

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
            depth: 0,
            scores: MoveScores::default(),
        }
    }

    /// starting from the expectation layer of the tree
    fn expectimax(&self, current_depth: usize) -> usize {
        // expecti
        // aka get a board state with each empty slot filled with a 2 or 4
        let empty_idxs: Vec<u8> = self
            .game
            .get_state()
            .iter()
            .enumerate()
            .filter(|(_, n)| **n != 0)
            .map(|(i, _)| i as u8)
            .collect::<Vec<_>>();

        let idx_to_tile: Vec<(u8, u8)> = empty_idxs.iter().fold(vec![], |mut acc, i| {
            acc.push((*i, 1));
            acc.push((*i, 2));
            acc
        });

        let games_to_avg: Vec<Game> = idx_to_tile
            .iter()
            .map(|(idx, s)| {
                // clone the game and set each tile to each of their expected values
                let mut game = self.game.clone();
                game.set_tile(idx % 4, idx / 4, *s);
                game
            })
            .collect::<Vec<_>>();

        // map each possible expectation to that expectation's score
        let list_of_expect_scores = games_to_avg
            .iter()
            .map(|game| {
                if current_depth >= self.depth {
                    // if we ran out of search depth, let's run RandomTree to get scores
                    dbg!("Loop");
                    let scores =
                        RandomTree::new_with(self.game.clone(), 10, RandomTreeMetric::AvgScore)
                            .score_moves();
                    return *scores.values().max().unwrap();
                }

                // maximization
                let mut scores = MoveScores::default();
                for m in Move::iter() {
                    let mut game = self.game.clone();
                    game.shift(m);
                    // and recurse
                    scores[m] = self.expectimax(current_depth + 1);
                }
                *scores.values().max().unwrap()
            })
            .collect::<Vec<_>>();

        // average the expectation scores and return it
        let sum_scores: usize = list_of_expect_scores.iter().sum();
        sum_scores / list_of_expect_scores.len()
    }
}

impl Agent for Expectimax {
    fn next_move(&self) -> Move {
        let mut scores = MoveScores::default();
        for m in Move::iter() {
            let mut game = self.game.clone();
            game.shift(m);
            scores[m] = self.expectimax(0);
        }
        scores.max_move()
    }

    fn make_move(&mut self) {
        let mut scores = MoveScores::default();
        for m in Move::iter() {
            let mut game = self.game.clone();
            game.shift(m);
            scores[m] = self.expectimax(0);
        }
        self.scores = scores;
        self.game.make_move(scores.max_move());
    }

    fn get_game(&mut self) -> &mut Game {
        &mut self.game
    }
}

impl TuiAgent for Expectimax {
    fn messages(&self) -> Vec<tui::text::Spans> {
        let highest_move = self
            .scores
            .iter()
            .max_by_key(|(_, score)| *score)
            .unwrap()
            .0;

        let mut score_spans = Move::iter()
            .map(|m| {
                if m == highest_move {
                    Spans::from(Span::styled(
                        format!("{}: {}", m, self.scores[m]),
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .bg(Color::Green),
                    ))
                } else {
                    Spans::from(format!("{}: {}", m, self.scores[m]))
                }
            })
            .collect::<Vec<_>>();

        let mut msgs = vec![
            Spans::from(Span::styled(
                "Expectimax",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Spans::from(format!(
                "Expectimax with search tree depth of {}, per move, to determine the next best move.",
                self.depth
            )),
            Spans::from(""),
        ];
        msgs.append(&mut score_spans);
        msgs
    }
}
