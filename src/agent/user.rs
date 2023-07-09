use crate::{
    game::{Game, Move},
    tui::IntAction,
};

use super::{Agent, TuiAgent};

use crossterm::event::{Event, KeyCode};
use std::{thread, time::Duration};

pub struct UserAgent {
    game: Game,
}

impl UserAgent {
    pub fn new(game: Game) -> Self
    where
        Self: Sized,
    {
        UserAgent { game }
    }
}

impl Agent for UserAgent {
    fn next_move(&self) -> Move {
        unimplemented!();
    }

    fn make_move(&mut self) {
        thread::sleep(Duration::from_millis(10));
    }

    fn get_game(&self) -> &Game {
        &self.game
    }
}

impl TuiAgent for UserAgent {
    fn messages(&self) -> Vec<tui::text::Spans> {
        vec![tui::text::Spans::from("Use WASD or arrow keys to move.")]
    }

    fn get_input(&mut self, event: &Event) -> IntAction {
        let Ok(keyboard_move) = (match event {
            Event::Key(key) => match key.code {
                KeyCode::Char('q') => return IntAction::Exit,
                KeyCode::Char('w') => Ok(Move::Up),
                KeyCode::Char('a') => Ok(Move::Left),
                KeyCode::Char('s') => Ok(Move::Down),
                KeyCode::Char('d') => Ok(Move::Right),
                KeyCode::Up => Ok(Move::Up),
                KeyCode::Left => Ok(Move::Left),
                KeyCode::Down => Ok(Move::Down),
                KeyCode::Right => Ok(Move::Right),
                _ => Err("Invalid key"),
            },
            _ => Err("Event not a key"),
        }) else {
            return IntAction::Exit;
        };

        // synchronously update the game
        self.game.make_move(keyboard_move);
        IntAction::Continue
    }
}
