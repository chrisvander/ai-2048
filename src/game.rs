use enum_map::Enum;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub struct Game {
    state: [u8; 16],
    score: usize,
    num_moves: usize,
}

impl Default for Game {
    fn default() -> Self {
        let mut g = Game::empty();
        g.generate_tile();
        g.generate_tile();
        g
    }
}

#[derive(
    Enum, EnumIter, Debug, PartialEq, Eq, Hash, Clone, Copy, Display, Serialize, Deserialize,
)]
pub enum Move {
    Up,
    Down,
    Left,
    Right,
}

impl Game {
    pub fn new() -> Self {
        Game::default()
    }

    pub fn empty() -> Self {
        Game {
            state: [0; 16],
            score: 0,
            num_moves: 0,
        }
    }

    pub fn new_seeded(seed: u64) -> Self {
        fastrand::seed(seed);
        Game::default()
    }

    pub fn new_from(state: [u8; 16]) -> Self {
        let mut game = Game::default();
        game.state = state;
        game
    }

    pub fn available_moves(&self) -> Vec<Move> {
        Move::iter()
            .filter(|m| {
                let mut game = self.clone();
                let state_before = game.state.clone();
                game.shift(*m);
                state_before != game.state
            })
            .collect::<Vec<_>>()
    }

    pub fn make_move(&mut self, input: Move) -> bool {
        let state_before = self.state.clone();
        self.shift(input);
        if state_before == self.state {
            return false;
        }
        self.generate_tile();
        self.num_moves += 1;
        true
    }

    pub fn get_tile(&self, x: u8, y: u8) -> u8 {
        self.state[(x + y * 4) as usize]
    }

    pub fn set_tile(&mut self, x: u8, y: u8, value: u8) {
        self.state[(x + y * 4) as usize] = value;
    }

    pub fn get_table(&self) -> Vec<Vec<u32>> {
        let pows = (&self.state)
            .iter()
            .map(|n| 2_u32.pow(*n as u32))
            .collect::<Vec<_>>();
        pows.chunks(4).map(|s| s.into()).collect()
    }

    pub fn game_over(&self) -> bool {
        // game is not over if any tile is empty
        if self.state.iter().any(|n| *n == 0) {
            return false;
        }

        // if any tile has a neighbor with the same value, game is not over
        // check rows
        for row in self.get_condensed_rows() {
            for i in 0..3 {
                if row[i] == row[i + 1] {
                    return false;
                }
            }
        }

        // check cols
        for col in self.get_condensed_cols() {
            for i in 0..3 {
                if col[i] == col[i + 1] {
                    return false;
                }
            }
        }

        true
    }

    pub fn get_num_moves(&self) -> &usize {
        &self.num_moves
    }

    pub fn get_score(&self) -> &usize {
        &self.score
    }

    pub fn get_state(&self) -> &[u8; 16] {
        &self.state
    }

    pub fn set_state(&mut self, s: [u8; 16]) {
        self.state = s;
    }

    /// This function takes duplicates in a vector of u8 and merges them, while scoring
    /// the result.
    /// * `v`: the current state that row/column
    /// * `score`: whether or not to score the result
    fn merge_duplicates(&mut self, v: &[u8; 4], score: bool) -> [u8; 4] {
        let mut i = 0;
        let mut j = 0;
        let mut r = [0; 4];
        while i < 4 {
            if i < 3 && v[i] != 0 && v[i] == v[i + 1] {
                r[j] = v[i] + 1;
                if score {
                    self.score += 2_usize.pow(r[j] as u32);
                }
                j += 1;
                i += 1;
            } else if v[i] != 0 {
                r[j] = v[i];
                j += 1;
            }
            i += 1;
        }
        r
    }

    fn first_empty(&self, v: &[u8; 4]) -> usize {
        v.iter().position(|n| *n == 0).unwrap_or(4)
    }

    fn get_cols(&self) -> [[u8; 4]; 4] {
        [
            [self.state[0], self.state[4], self.state[8], self.state[12]],
            [self.state[1], self.state[5], self.state[9], self.state[13]],
            [self.state[2], self.state[6], self.state[10], self.state[14]],
            [self.state[3], self.state[7], self.state[11], self.state[15]],
        ]
    }

    fn get_condensed_rows(&self) -> [[u8; 4]; 4] {
        // get vec of rows with empty removed
        self.state
            .chunks(4)
            .enumerate()
            .fold([[0; 4]; 4], |mut acc, (i, row)| {
                row.iter()
                    .filter(|n| **n != 0)
                    .for_each(|n| acc[i][self.first_empty(&acc[i])] = *n);
                acc
            })
    }

    fn get_condensed_cols(&self) -> [[u8; 4]; 4] {
        // get vec of cols with empty removed
        self.get_cols()
            .iter()
            .enumerate()
            .fold([[0; 4]; 4], |mut acc, (i, col)| {
                col.iter()
                    .filter(|n| **n != 0)
                    .for_each(|n| acc[i][self.first_empty(&acc[i])] = *n);
                acc
            })
    }

    pub fn shift(&mut self, input: Move) {
        let condensed = match input {
            Move::Up | Move::Down => self.get_condensed_cols(),
            Move::Left | Move::Right => self.get_condensed_rows(),
        };

        let mut new_state: [u8; 16] = [0; 16];
        condensed
            .iter()
            .map(|v| self.merge_duplicates(v, true))
            .map(|v| {
                if input == Move::Up || input == Move::Left || !v.contains(&0) {
                    v
                } else {
                    // swap where 0's are
                    let sp = v.iter().position(|n| *n == 0).unwrap_or(3);
                    [&v[sp..], &v[0..sp]].concat().try_into().unwrap()
                }
            })
            .flatten()
            .enumerate()
            .for_each(|(i, n)| new_state[i] = n);

        match input {
            Move::Up | Move::Down => {
                let mut transposed = [0; 16];
                transpose::transpose(&mut new_state, &mut transposed, 4, 4);
                self.set_state(transposed);
            }
            Move::Left | Move::Right => self.set_state(new_state),
        }
    }

    fn generate_tile(&mut self) {
        let p = fastrand::f32();
        let n = if p < 0.9 { 1 } else { 2 };

        // get indexes of empty tiles
        let empty_indexes = self
            .state
            .iter()
            .enumerate()
            .filter(|(_, &v)| v == 0)
            .map(|(i, _)| i)
            .collect::<Vec<_>>();

        if empty_indexes.len() == 0 {
            return;
        }

        let c_idx = fastrand::usize(0..empty_indexes.len());
        let chosen_index = empty_indexes[c_idx];
        self.state[chosen_index] = n;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn set_state_and_tile() {
        let mut game = Game::empty();
        assert_eq!(game.get_tile(0, 0), 0);

        game.set_state([
            0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc,
            0xde, 0xf0,
        ]);

        assert_eq!(game.get_tile(0, 0), 0x12);
        assert_eq!(game.get_tile(1, 0), 0x34);
        assert_eq!(game.get_tile(2, 0), 0x56);
        assert_eq!(game.get_tile(3, 0), 0x78);
        assert_eq!(game.get_tile(0, 1), 0x9a);

        game.set_tile(0, 1, 0x00);
        assert_eq!(game.get_tile(0, 1), 0x00);
    }

    #[test]
    fn test_shift() {
        let mut game = Game::empty();

        game.set_tile(0, 0, 1);
        game.set_tile(1, 0, 1);

        game.shift(Move::Right);

        assert_eq!(game.get_tile(0, 0), 0);
        assert_eq!(game.get_tile(1, 0), 0);
        assert_eq!(game.get_tile(2, 0), 0);
        assert_eq!(game.get_tile(3, 0), 2);

        game.shift(Move::Left);

        assert_eq!(game.get_tile(0, 0), 2);

        game.shift(Move::Down);

        assert_eq!(game.get_tile(0, 3), 2);
    }

    #[test]
    fn test_merge_duplicates() {
        let mut game = Game::default();
        let v = [1, 1, 2, 2];
        assert_eq!(game.merge_duplicates(&v, true), [2, 3, 0, 0]);
        assert_eq!(game.get_score(), &12);
        let v = [1, 2, 2, 2];
        assert_eq!(game.merge_duplicates(&v, true), [1, 3, 2, 0]);
        assert_eq!(game.get_score(), &(12 + 8));
        let v = [1, 1, 1, 1];
        assert_eq!(game.merge_duplicates(&v, true), [2, 2, 0, 0]);
        assert_eq!(game.get_score(), &(12 + 8 + 8));
        let v = [1, 2, 2, 6];
        assert_eq!(game.merge_duplicates(&v, true), [1, 3, 6, 0]);
    }

    #[test]
    fn test_condensed_getters() {
        let mut game = Game::empty();

        game.set_tile(0, 0, 1);
        game.set_tile(1, 0, 1);
        game.set_tile(0, 3, 2);

        let condensed_rows = game.get_condensed_rows();
        let condensed_cols = game.get_condensed_cols();

        assert_eq!(condensed_rows[0], [1, 1, 0, 0]);
        assert_eq!(condensed_rows[3], [2, 0, 0, 0]);
        assert_eq!(condensed_cols[0], [1, 2, 0, 0]);
        assert_eq!(condensed_cols[1], [1, 0, 0, 0]);
    }
}
