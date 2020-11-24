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

impl From<GameTree> for History {
    //this is costly, playing out the whole game to create the history tree
    fn from(_t: GameTree) -> Self {
        let _goban = crate::goban::Goban::default();
        Self::default()
    }
}

impl History {
    pub fn into_game_tree(&self) -> sgf_parser::GameTree {
        self.build_game_tree(Player::Black).unwrap()
    }

    fn build_game_tree(&self, player: crate::Player) -> Option<GameTree> {
        let mut player = player;
        let mut sgf = GameTree::default();
        if !self.moves.is_empty() {
            for mov in self.moves.iter() {
                let crate::goban::Point { x, y } = crate::goban::Goban::idx_to_coord(mov.index);
                sgf.nodes.push(GameNode {
                    tokens: vec![sgf_parser::SgfToken::Move {
                        color: player.into(),
                        action: sgf_parser::Action::Move(x as u8 + 1, y as u8 + 1)}]
                });
                player.next();
            }
        }
        if !self.variations.is_empty() {
            for v in self.variations.iter() {
                if let Some(g) = v.build_game_tree(player) {
                    sgf.variations.push(g);
                }
            }
        }

        Some(sgf)
    }

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
