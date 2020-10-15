use sgf_parser::{GameTree, GameNode};
use crate::Player;
use crate::goban::{Move, Group};

//TODO: from sgf_parser::GameTree
//      into sgf_parser::GameTree
#[derive(Debug, Clone, Default)]
pub struct History {
    pub moves: Vec<Move>,
    pub variations: Vec<History>,
    pub current_index: usize,
}

impl History {
    pub fn push(&mut self, elem: (usize, Vec<Group>)) {
        if self.current_index + 1 < self.moves.len() {
            let _ = self.moves.split_off(self.current_index);
        }
        self.moves.push(elem.into());
        self.current_index += 1;
    }

    pub fn pop(&mut self) -> Option<(usize, Vec<Group>)> {
        if self.current_index as isize - 1 >= 0 {
            self.current_index -= 1;
            Some(self.moves[self.current_index].clone().into())
        } else {
            None
        }
    }

    pub fn next(&mut self) -> Option<(Player, usize, Vec<Group>)> {
        if self.current_index == self.moves.len() {
            None
        } else {
            let player = match self.current_index % 2 {
                0 => Player::Black,
                1 => Player::White,
                _ => unreachable!(),
            };
            self.current_index += 1;
            let (idx, dead_stones) = self.moves[self.current_index-1].clone().into();
            Some((player, idx, dead_stones))
        }
    }
}
