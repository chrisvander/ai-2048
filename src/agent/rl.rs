use enum_map::EnumMap;
use etcetera::{choose_base_strategy, BaseStrategy};
use rurel::{
    mdp::{Agent, State},
    AgentTrainer,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs};
use strum::IntoEnumIterator;
use tui::{
    style::{Color, Modifier, Style},
    text::{Span, Spans},
};

use crate::game::{Game, Move};

use super::{Agent as GameAgent, TuiAgent};

pub fn data_file_path() -> String {
    let mut path = choose_base_strategy().unwrap().data_dir();
    path.push("2048");
    fs::create_dir_all(&path).unwrap();
    path.push("rl_agent.json");
    path.to_str().unwrap().to_string()
}

pub fn get_trainer() -> AgentTrainer<GameState> {
    let mut trainer = AgentTrainer::new();
    let Ok(data) = fs::read_to_string(data_file_path()) else {
        return trainer;
    };
    let imported_state: TrainerExport = ron::from_str(&data).unwrap();
    trainer.import_state(imported_state);
    trainer
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize)]
pub struct GameState {
    state: [u8; 16],
}
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct RLAgent {
    state: GameState,
}

impl RLAgent {
    pub fn new(game: Game) -> Self
    where
        Self: Sized,
    {
        RLAgent {
            state: GameState {
                state: *game.get_state(),
            },
        }
    }
}

impl State for GameState {
    type A = Move;

    fn actions(&self) -> Vec<Move> {
        Move::iter()
            .filter(|m| Game::new_from(self.state).is_available_move(*m))
            .collect()
    }

    fn reward(&self) -> f64 {
        let game = Game::new_from(self.state);
        if !game.game_over() {
            0.0
        } else {
            *game.get_score() as f64
        }
    }
}

impl Agent<GameState> for RLAgent {
    fn current_state(&self) -> &GameState {
        &self.state
    }

    fn take_action(&mut self, action: &Move) {
        let mut game = Game::new_from(self.state.state);
        game.update(*action);
        self.state.state = *game.get_state();
    }
}

type MoveScores = EnumMap<Move, usize>;
pub struct RLAgentTrained {
    game: Game,
    last_scores: MoveScores,
    trainer: AgentTrainer<GameState>,
}

type TrainerExport = HashMap<GameState, HashMap<Move, f64>>;
impl RLAgentTrained {
    pub fn new(game: Game) -> Self
    where
        Self: Sized,
    {
        RLAgentTrained {
            game,
            last_scores: Move::iter().map(|m| (m, 0)).collect(),
            trainer: get_trainer(),
        }
    }
}

impl GameAgent for RLAgentTrained {
    fn next_move(&mut self) {
        let state = GameState {
            state: *self.game.get_state(),
        };
        self.last_scores = Move::iter()
            .map(|m| {
                (
                    m,
                    self.trainer.expected_value(&state, &m).unwrap_or_default() as usize,
                )
            })
            .collect::<MoveScores>();

        let best_move = self
            .last_scores
            .iter()
            .filter(|(m, _)| self.game.is_available_move(*m))
            .max_by_key(|(_, score)| *score)
            .unwrap()
            .0;

        self.game.update(best_move);
    }

    fn get_game(&self) -> &Game {
        &self.game
    }
}

impl TuiAgent for RLAgentTrained {
    fn messages(&self) -> Vec<Spans<'_>> {
        let highest_move = self
            .last_scores
            .iter()
            .max_by_key(|(_, score)| *score)
            .unwrap()
            .0;

        let score_spans = Move::iter()
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

        score_spans
    }
}
