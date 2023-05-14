use etcetera::{choose_base_strategy, BaseStrategy};
use rurel::{
    mdp::{Agent, State},
    AgentTrainer,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs};
use strum::IntoEnumIterator;

use crate::game::{Game, Move};

use super::Agent as GameAgent;

pub const STRATEGY: BaseStrategy = choose_base_strategy().unwrap();
pub const STORE_PATH: &str = STRATEGY.data_dir();

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct RLAgent {
    game: Game,
}

impl RLAgent {
    pub fn new(game: Game) -> Self
    where
        Self: Sized,
    {
        RLAgent { game }
    }
}

impl State for Game {
    type A = Move;

    fn actions(&self) -> Vec<Move> {
        Move::iter().collect()
    }

    fn reward(&self) -> f64 {
        if self.game_over() {
            0.0
        } else {
            *self.get_score() as f64
        }
    }
}

impl Agent<Game> for RLAgent {
    fn current_state(&self) -> &Game {
        &self.game
    }

    fn take_action(&mut self, action: &Move) {
        self.game.update(*action);
    }
}

pub struct RLAgentTrained {
    game: Game,
    trainer: AgentTrainer<Game>,
}

type TrainerExport = HashMap<Game, HashMap<Move, f64>>;

impl RLAgentTrained {
    pub fn new(game: Game) -> Self
    where
        Self: Sized,
    {
        // read in trainer
        let mut trainer = AgentTrainer::new();
        let data = fs::read_to_string(STORE_PATH).expect("Unable to read file");
        let imported_state: TrainerExport = serde_json::from_str(&data).unwrap();
        trainer.import_state(imported_state);
        RLAgentTrained { game, trainer }
    }
}

impl GameAgent for RLAgentTrained {
    fn next_move(&mut self) {
        let Some(action) = self.trainer.best_action(&self.game) else { return; };
        self.game.update(action);
    }

    fn get_game(&self) -> &Game {
        &self.game
    }

    fn messages(&self) -> Vec<tui::text::Spans> {
        vec![tui::text::Spans::from("Performing RL actions.")]
    }
}
