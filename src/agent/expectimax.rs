use std::cmp::max;

use fastrand::shuffle;
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

#[derive(Default)]
pub struct ExpectimaxMetrics {
    scores: MoveScores,
    num_tiles: usize,
}

pub struct Expectimax {
    game: Game,
    seed: Option<u64>,
    metrics: ExpectimaxMetrics,
}

pub struct ExpectimaxParams {
    tree_depth: usize,        // how deep will our search tree go?
    num_expecti_tiles: usize, // how many tiles will we evaluate at the expectation step?
    heuristic_sims: usize,    // how many simulations will we do as a "heuristic" for board
                              // evaluation
}

impl Expectimax {
    pub fn new(game: Game) -> Self
    where
        Self: Sized,
    {
        Expectimax {
            game,
            seed: None,
            metrics: ExpectimaxMetrics::default(),
        }
    }

    pub fn new_seeded(seed: u64, game: Game) -> Self
    where
        Self: Sized,
    {
        Expectimax {
            game,
            seed: Some(seed),
            metrics: ExpectimaxMetrics::default(),
        }
    }

    fn empty_idxs(&self) -> Vec<u8> {
        self.game
            .get_state()
            .iter()
            .enumerate()
            .filter(|(_, n)| **n == 0)
            .map(|(i, _)| i as u8)
            .collect::<Vec<_>>()
    }

    /// starting from the expectation layer of the tree
    fn expectimax_recurse(
        &self,
        current_depth: usize,
        params: &ExpectimaxParams,
    ) -> (usize, usize) {
        // expecti
        // aka get a board state with each empty slot filled with a 2 or 4
        let mut empty_idxs = self.empty_idxs();
        shuffle(&mut empty_idxs);

        // for speed, let's limit the number of tiles we recurse on in the expectation portion.
        // We expect that the expectation portion can be "fuzzier".
        let idx_to_tile: Vec<(u8, u8, f32)> = empty_idxs
            .iter()
            .take(params.num_expecti_tiles)
            .fold(vec![], |mut acc, i| {
                acc.push((*i, 1, 0.9));
                acc.push((*i, 2, 0.1));
                acc
            });

        let num_tiles = idx_to_tile.len();
        let games_to_avg: Vec<(Game, f32)> = if num_tiles == 0 {
            vec![(self.game.clone(), 1.0)]
        } else {
            idx_to_tile
                .iter()
                .map(|(idx, s, p)| {
                    // clone the game and set each tile to each of their expected values
                    let mut game = self.game.clone();
                    game.set_tile(idx % 4, idx / 4, *s);
                    (game, *p)
                })
                .collect::<Vec<_>>()
        };

        // map each possible expectation to that expectation's score
        let list_of_expect_scores = games_to_avg
            .par_iter()
            .map(|(game, p)| {
                if current_depth >= params.tree_depth {
                    // if we ran out of search depth, let's run RandomTree to get scores
                    if let Some(seed) = self.seed {
                        fastrand::seed(seed);
                    }
                    let scores = RandomTree::new_with(
                        self.game.clone(),
                        params.heuristic_sims,
                        RandomTreeMetric::AvgMoves,
                        true,
                    )
                    .score_moves();
                    return *scores.values().max().unwrap() as f32;
                }

                // maximization
                let mut scores = MoveScores::default();
                for m in game.available_moves() {
                    let mut game = self.game.clone();
                    game.shift(m);
                    // and recurse
                    let (score, _) = self.expectimax_recurse(current_depth + 1, params);
                    scores[m] = score;
                }
                let max_score = *scores.values().max().unwrap() as f32;
                max_score * p
            })
            .collect::<Vec<_>>();

        // average the expectation scores and return it
        let sum_scores: f32 = list_of_expect_scores.iter().sum();
        (
            (sum_scores / list_of_expect_scores.len() as f32) as usize,
            max(1, num_tiles),
        )
    }

    fn expectimax(&self) -> (Move, ExpectimaxMetrics) {
        let params = ExpectimaxParams {
            tree_depth: 3,
            heuristic_sims: 100,
            num_expecti_tiles: 3,
        };
        let mut scores = MoveScores::default();
        let mut num_tiles = 0;
        for m in Move::iter() {
            let mut game = self.game.clone();
            game.shift(m);
            let (score, nt) = self.expectimax_recurse(0, &params);
            num_tiles = nt;
            scores[m] = score;
        }
        (scores.max_move(), ExpectimaxMetrics { scores, num_tiles })
    }
}

impl Agent for Expectimax {
    fn next_move(&self) -> Move {
        let (m, _) = self.expectimax();
        m
    }

    fn make_move(&mut self) {
        let (m, mt) = self.expectimax();
        self.metrics = mt;
        self.game.make_move(self.metrics.scores.max_move());
    }

    fn get_game(&self) -> &Game {
        &self.game
    }
}

impl TuiAgent for Expectimax {
    fn messages(&self) -> Vec<tui::text::Spans> {
        let highest_move = self
            .metrics
            .scores
            .iter()
            .max_by_key(|(_, score)| *score)
            .unwrap()
            .0;

        let mut score_spans = Move::iter()
            .map(|m| {
                if m == highest_move {
                    Spans::from(Span::styled(
                        format!("{}: {}", m, self.metrics.scores[m]),
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .bg(Color::Green),
                    ))
                } else {
                    Spans::from(format!("{}: {}", m, self.metrics.scores[m]))
                }
            })
            .collect::<Vec<_>>();

        let mut msgs = vec![
            Spans::from(Span::styled(
                "Expectimax",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Spans::from(format!("Expectimax to determine the next best move.",)),
            Spans::from(format!("Analyzing {} tiles", self.metrics.num_tiles)),
            Spans::from(""),
        ];
        msgs.append(&mut score_spans);
        msgs
    }
}
