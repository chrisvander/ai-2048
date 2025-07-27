use std::sync::{Arc, Mutex};

use fastrand::shuffle;
use rayon::prelude::*;
use strum::IntoEnumIterator;
use tui::{
    style::{Color, Modifier, Style},
    text::{Span, Spans},
};

use crate::game::{Game, Move};

use super::{random::simulate_random_game, Agent, MaxMove, MoveScores, TuiAgent};

pub struct Expectimax {
    game: Game,
    params: ExpectimaxParams,
    last_scores: MoveScores,
}

pub struct ExpectimaxParams {
    seed: Option<u64>,             // rand seed
    max_evals: usize,              // how many nodes will we evaluate before using a heuristic?
    curr_evals: Arc<Mutex<usize>>, // a mutexed counter for how many moves we've evaluated
    num_tiles: usize, // what is the maximum number of tiles will we evaluate at the expectation step?
    heuristic_sims: usize, // how many random simulations will we do as a "heuristic" for board evaluation
}

impl ExpectimaxParams {
    fn add_count(&self) {
        let mut e = self.curr_evals.lock().unwrap();
        *e += 1;
    }

    fn curr_count(&self) -> usize {
        let e = self.curr_evals.lock().unwrap();
        e.to_owned()
    }

    fn reset_count(&self) {
        let mut e = self.curr_evals.lock().unwrap();
        *e = 0;
    }
}

impl Default for ExpectimaxParams {
    fn default() -> Self {
        ExpectimaxParams {
            seed: None,
            max_evals: 500,
            curr_evals: Arc::new(Mutex::new(0)),
            num_tiles: 16,
            heuristic_sims: 10,
        }
    }
}

trait GameHeuristic {
    fn score(game: &Game, params: &ExpectimaxParams) -> f32;
}

struct RandHeuristic;
impl GameHeuristic for RandHeuristic {
    fn score(game: &Game, params: &ExpectimaxParams) -> f32 {
        if let Some(seed) = params.seed {
            fastrand::seed(seed);
        }
        (0..params.heuristic_sims)
            .into_iter()
            .map(|_| *simulate_random_game(game.clone()).get_score() as f32)
            .sum::<f32>()
            / params.heuristic_sims as f32
    }
}

struct GameOverHeuristic;
impl GameHeuristic for GameOverHeuristic {
    fn score(game: &Game, params: &ExpectimaxParams) -> f32 {
        let empty_tiles = empty_idxs(game).len();
        if let Some(seed) = params.seed {
            fastrand::seed(seed);
        }

        ((0..params.heuristic_sims)
            .into_par_iter()
            .map(|_| {
                let rand_game = simulate_random_game(game.clone());
                (empty_tiles * *rand_game.get_score()) as f32
            })
            .sum::<f32>())
            / params.heuristic_sims as f32
    }
}

fn empty_idxs(game: &Game) -> Vec<u8> {
    game.get_state()
        .iter()
        .enumerate()
        .filter(|(_, n)| **n == 0)
        .map(|(i, _)| i as u8)
        .collect::<Vec<_>>()
}

/// starting from the expectation layer of the tree
fn expectimax_recurse(game: &Game, params: &ExpectimaxParams) -> usize {
    if params.curr_count() >= params.max_evals {
        return GameOverHeuristic::score(&game, &params) as usize;
    }

    // expecti
    // aka get a board state with each empty slot filled with a 2 or 4
    let mut empty_idxs = empty_idxs(&game);
    if game.game_over() {
        return 0;
    }
    shuffle(&mut empty_idxs);

    // for speed, let's limit the number of tiles we recurse on in the expectation portion.
    // We expect that the expectation portion can be "fuzzier".
    let idx_to_tile: Vec<(u8, u8, f32)> =
        empty_idxs
            .iter()
            .take(params.num_tiles)
            .fold(vec![], |mut acc, i| {
                acc.push((*i, 1, 0.9));
                acc.push((*i, 2, 0.1));
                acc
            });

    let num_tiles = idx_to_tile.len();
    let games_to_avg: Vec<(Game, f32)> = if num_tiles == 0 {
        vec![(game.clone(), 1.0)]
    } else {
        idx_to_tile
            .iter()
            .map(|(idx, s, p)| {
                // clone the game and set each tile to each of their expected values
                let mut game = game.clone();
                game.set_tile(idx % 4, idx / 4, *s);
                (game, *p)
            })
            .collect::<Vec<_>>()
    };

    // map each possible expectation to that expectation's score
    let list_of_expect_scores = games_to_avg
        .par_iter()
        .map(|(game, p)| {
            let mut scores = MoveScores::default();
            for m in game.available_moves() {
                params.add_count();

                let mut game = game.clone();
                game.shift(m);
                scores[m] = expectimax_recurse(&game, params);
            }
            let max_score = *scores.values().max().unwrap() as f32;
            max_score * p
        })
        .collect::<Vec<_>>();

    // average the expectation scores and return it
    let sum_scores: f32 = list_of_expect_scores.iter().sum();
    (sum_scores / list_of_expect_scores.len() as f32) as usize
}

impl Expectimax {
    pub fn new(game: Game) -> Self
    where
        Self: Sized,
    {
        Expectimax {
            game,
            params: ExpectimaxParams::default(),
            last_scores: MoveScores::default(),
        }
    }

    pub fn new_seeded(seed: u64, game: Game) -> Self
    where
        Self: Sized,
    {
        Expectimax {
            game,
            params: ExpectimaxParams {
                seed: Some(seed),
                ..ExpectimaxParams::default()
            },
            last_scores: MoveScores::default(),
        }
    }

    fn expectimax(&self) -> MoveScores {
        let mut scores = MoveScores::default();
        let avail_moves = self.game.available_moves();
        self.params.reset_count();
        for m in avail_moves {
            let mut game = self.game.clone();
            game.shift(m);
            let score = expectimax_recurse(&game, &self.params);
            scores[m] = score;
        }
        scores
    }
}

impl Agent for Expectimax {
    fn next_move(&self) -> Move {
        let m = self.expectimax();
        m.max_move()
    }

    fn make_move(&mut self) {
        self.params = ExpectimaxParams {
            seed: self.params.seed,
            ..ExpectimaxParams::default()
        };
        let m = self.expectimax();
        self.last_scores = m;
        self.game.make_move(m.max_move());
    }

    fn get_game(&self) -> &Game {
        &self.game
    }
}

impl TuiAgent for Expectimax {
    fn messages(&self) -> Vec<tui::text::Spans> {
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
                        format!("{}: {}", m, self.last_scores[m]),
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .bg(Color::Green),
                    ))
                } else {
                    Spans::from(format!("{}: {}", m, self.last_scores[m]))
                }
            })
            .collect::<Vec<_>>();

        let mut msgs = vec![
            Spans::from(Span::styled(
                "Expectimax",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Spans::from(format!(
                "Expectimax to determine the next best move. Approximately analyzing {} {} {} states per turn.",
                self.params.curr_evals.try_lock().unwrap(), self.params.heuristic_sims, self.params.max_evals
            )),
            Spans::from(""),
        ];
        msgs.append(&mut score_spans);
        msgs
    }
}
