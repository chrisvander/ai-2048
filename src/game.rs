use enum_map::Enum;
use strum_macros::{Display, EnumIter};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub struct Game {
    state: [u8; 16],
    score: usize,
    num_moves: usize,
}

impl Default for Game {
    fn default() -> Self {
        let mut g = Game {
            state: [0; 16],
            score: 0,
            num_moves: 0,
        };
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

/// This function generates a new tile in a random empty spot. The new tile
/// will be a 2 with 90% probability and a 4 with 10% probability.
///
/// * `v`: the current state of the game
/// * `f`: a function to call when a new tile is generated
fn merge_duplicates<F>(v: &Vec<u8>, mut f: F) -> Vec<u8>
where
    F: FnMut(usize) -> (),
{
    let mut r = Vec::with_capacity(4);
    let mut skip = false;
    for i in 0..v.len() {
        if skip {
            skip = false;
            continue;
        }
        if i < v.len() - 1 && v[i] == v[i + 1] {
            r.push(v[i] + 1);
            // score
            f(2_usize.pow((v[i] + 1).into()));
            skip = true;
        } else {
            r.push(v[i]);
        }
    }
    r
}

impl Game {
    pub fn new() -> Self {
        Game::default()
    }

    pub fn new_from(state: [u8; 16]) -> Self {
        let mut game = Game::default();
        game.state = state;
        game
    }

    pub fn update(&mut self, input: Move) -> bool {
        let state_before = self.state.clone();
        self.shift(input);
        // element wise compare
        if state_before != self.state {
            self.generate_tile();
            self.num_moves += 1;
            return true;
        }
        false
    }

    fn xy_to_index(x: u8, y: u8) -> usize {
        (x + y * 4) as usize
    }

    pub fn is_available_move(&self, input: Move) -> bool {
        let mut g = self.clone();
        g.shift(input);
        g.state != self.state
    }

    pub fn get_tile(&self, x: u8, y: u8) -> u8 {
        self.state[Game::xy_to_index(x, y)]
    }

    pub fn set_tile(&mut self, x: u8, y: u8, value: u8) {
        self.state[Game::xy_to_index(x, y)] = value;
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
        return true;
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

    fn get_condensed_rows(&self) -> Vec<Vec<u8>> {
        // get vec of rows with empty removed
        self.state
            .chunks(4)
            .map(|row| row.iter().map(|n| *n).filter(|n| *n != 0).collect())
            .collect()
    }

    fn get_condensed_cols(&self) -> Vec<Vec<u8>> {
        // get vec of cols with empty removed
        self.state
            .iter()
            .enumerate()
            .fold(vec![Vec::with_capacity(4); 4], |mut acc, (i, n)| {
                if *n != 0 {
                    acc[i % 4].push(*n);
                }
                acc
            })
    }

    fn shift(&mut self, input: Move) {
        let condensed = match input {
            Move::Up | Move::Down => self.get_condensed_cols(),
            Move::Left | Move::Right => self.get_condensed_rows(),
        };

        let new_state: [u8; 16] = condensed
            .iter()
            .map(|v| merge_duplicates(v, |s| self.score += s))
            .map(|mut v| {
                let mut r = vec![0; 4 - v.len()];
                if input == Move::Up || input == Move::Left {
                    v.append(&mut r);
                    v
                } else {
                    r.append(&mut v);
                    r
                }
            })
            .flatten()
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        match input {
            Move::Up | Move::Down => {
                let mut transposed = vec![0; 16];
                transpose::transpose(&mut Vec::from(new_state), &mut transposed, 4, 4);
                self.set_state(transposed.try_into().unwrap());
            }
            Move::Left | Move::Right => self.set_state(new_state),
        }
    }

    fn generate_tile(&mut self) {
        let n = fastrand::u8(1..=2);

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
        let mut game = Game::default();
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
        let mut game = Game::default();

        game.set_state([0; 16]);
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
        let v = vec![1, 1, 2, 2];
        assert_eq!(merge_duplicates(&v, |_| {}), vec![2, 3]);
        let v = vec![1, 2, 2, 2];
        assert_eq!(merge_duplicates(&v, |_| {}), vec![1, 3, 2]);
        let v = vec![1, 1, 1, 1];
        assert_eq!(merge_duplicates(&v, |_| {}), vec![2, 2]);
        let v = vec![1, 2, 2, 6];
        assert_eq!(merge_duplicates(&v, |_| {}), vec![1, 3, 6]);
    }

    #[test]
    fn test_condensed_getters() {
        let mut game = Game::default();

        game.set_state([0; 16]);
        game.set_tile(0, 0, 1);
        game.set_tile(1, 0, 1);
        game.set_tile(0, 3, 2);

        let condensed_rows = game.get_condensed_rows();
        let condensed_cols = game.get_condensed_cols();

        assert_eq!(condensed_rows[0], vec![1, 1]);
        assert_eq!(condensed_rows[3], vec![2]);
        assert_eq!(condensed_cols[0], vec![1, 2]);
        assert_eq!(condensed_cols[1], vec![1]);
    }
}
