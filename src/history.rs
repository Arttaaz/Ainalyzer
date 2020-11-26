use std::collections::HashMap;

use sgf_parser::{GameTree, GameNode};
use petgraph::prelude::*;

use crate::{Player, goban::{Point, Stone}};
use crate::goban::{Move, Group};

pub enum HistoryError {
    PushCurrentNodeNotFound,
}


//TODO: from sgf_parser::GameTree
//      into sgf_parser::GameTree
#[derive(Debug, Clone, Default)]
pub struct History {
    pub moves: Graph<Move, (), Directed>,
    pub current_index: NodeIndex<u32>,
    pub variation_picker: HashMap<NodeIndex<u32>, NodeIndex<u32>>,
}

impl From<GameTree> for History {
    // This is costly, playing out the whole game to create the history tree
    // The sgf loading from must only contain valid moves
    fn from(t: GameTree) -> Self {
        let mut goban = crate::goban::Goban::default();
        let mut history = Self::default();
        for n in t.nodes {
            for t in n.tokens {
                match t {
                    sgf_parser::SgfToken::Move { color, action } => {
                        if let sgf_parser::Action::Move(x, y) = action {
                            let mut turn = match color {
                                sgf_parser::Color::Black => Player::Black,
                                sgf_parser::Color::White => Player::White,
                            };
                            let stone = match turn.clone() { Player::White => Stone::white(), Player::Black => Stone::black() };
                            goban.play(&mut history, &mut turn, Point::new(x as u32 -1, y as u32 - 1), stone);
                        }
                    },
                    _ => (),
                }
            }
        }
        history
    }
}

impl History {
    pub fn into_game_tree(&self) -> sgf_parser::GameTree {
        self.build_game_tree(Player::Black).unwrap()
    }

    fn build_game_tree(&self, player: crate::Player) -> Option<GameTree> {
        let mut player = player;
        let mut sgf = GameTree::default();
        Some(sgf)
    }

    pub fn push(&mut self, elem: (Player, usize, Vec<Group>)) -> Result<(), HistoryError> {
        // If the graph is empty we need a fictional first node
        if self.moves.node_count() == 0 {
            self.moves.add_node(Move {
                player: Player::White,
                index: 0,
                groups: Vec::new(),
            });
        }

        let new_node = self.moves.add_node(elem.into());
        // This check is there to prevent add_edge from panicking
        if self.moves.node_weight(self.current_index).is_some() {
            // if our current node already have outgoing neighbors
            // we just created a variation and need to add it to variation_picker
            if self.moves.neighbors(self.current_index).next().is_some() {
                self.variation_picker.insert(self.current_index, new_node);
            }
            self.moves.add_edge(self.current_index, new_node, ());
        } else {
            return Err(HistoryError::PushCurrentNodeNotFound)
        }
        self.current_index = new_node;
        Ok(())
    }

    pub fn pop(&mut self) -> Option<(Option<usize>, (Player, usize, Vec<Group>))> {
        if self.current_index == 0.into() {
            return None
        }
        // we check if the index exists
        if let Some(_) = self.moves.node_weight(self.current_index) {
            let move_to_pop = self.moves[self.current_index].clone();
            // We should ever have one incoming edge
            let previous_move = self.moves.neighbors_directed(self.current_index, Direction::Incoming).next();
            let previous_move = if let Some(i) = previous_move {
                self.current_index = i;
                if i == 0.into() {
                    None
                } else {
                    Some(self.moves[i].index)
                }
            } else {
                None
            };
            Some((previous_move, move_to_pop.into()))
        } else {
            None
        }
    }

    pub fn next(&mut self) -> Option<(Player, usize, Vec<Group>)> {
        // If variations exist we just pick the current one from our variation_picker
        if let Some(next_move) =  self.variation_picker.get(&self.current_index) {
            let mov = self.moves[next_move.clone()].clone();
            self.current_index = *next_move;
            Some(mov.into())
        } else if let Some(next) = self.moves.neighbors(self.current_index).next() {
            let mov = self.moves[next].clone();
            self.current_index = next;
            Some(mov.into())
        } else {
            None
        }
    }

    // returns true if move is a variation choice
    pub fn set_variation_to_move(&mut self, mov: usize) -> bool {
        for e in self.moves.neighbors(self.current_index) {
            if self.moves[e].index == mov {
                self.variation_picker.insert(self.current_index, e);
                return true
            } 
        }
        false
    }

    pub fn get_possible_moves(&self) -> Vec<usize> {
        self.moves.neighbors(self.current_index)
            .map(|e| self.moves[e].index)
            .collect()
    }
}
