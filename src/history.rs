use std::collections::HashMap;

use sgf_parser::{GameTree, GameNode, SgfToken};
use petgraph::prelude::*;

use crate::{Player, goban::{Point, Stone}};
use crate::goban::{Move, Group};

pub enum HistoryError {
    PushCurrentNodeNotFound,
}


//TODO: into sgf_parser::GameTree
#[derive(Debug, Clone)]
pub struct History {
    pub moves: Graph<Move, (), Directed>,
    pub current_index: NodeIndex<u32>,
    pub variation_picker: HashMap<NodeIndex<u32>, NodeIndex<u32>>,
    pub game_info: GameNode,
}

impl Default for History {
    fn default() -> Self {
        let mut graph = Graph::new();
        graph.add_node(Move {
            player: Player::White,
            index: 0,
            groups: Vec::new(),
        });

        let game_info = GameNode {
            tokens: vec![
                SgfToken::Game(sgf_parser::Game::Go),
                SgfToken::FileFormat(4),
                SgfToken::Charset(sgf_parser::Encoding::UTF8),
                SgfToken::Application {
                    name: "AInalyzer".to_string(),
                    version: "0.1.0".to_string(),
                },
                SgfToken::VariationDisplay {
                    nodes: sgf_parser::DisplayNodes::Children,
                    on_board_display: true,
                },
                SgfToken::Rule(sgf_parser::RuleSet::Japanese),
                SgfToken::Size(19, 19),
                SgfToken::Komi(6.5),
                SgfToken::PlayerName {
                    color: sgf_parser::Color::Black,
                    name: "Black".to_string(),
                },
                SgfToken::PlayerName {
                    color: sgf_parser::Color::White,
                    name: "White".to_string(),
                },
            ],
        };

        History {
            moves: graph,
            current_index: 0.into(),
            variation_picker: HashMap::new(),
            game_info,
        }
    }
}

impl From<GameTree> for History {
    // This is costly, playing out the whole game to create the history tree
    // The sgf loading from must only contain valid moves
    fn from(t: GameTree) -> Self {
        log::debug!("start loading sgf");
        let mut goban = crate::Goban::default();
        goban.history.game_info = t.nodes.first().unwrap().clone();
        History::add_tree_to_history(t, &mut goban);
        goban.history.current_index = 0.into();
        log::debug!("finished loading sgf");
        goban.history
    }
}

impl History {

    fn add_tree_to_history(tree: GameTree, goban: &mut crate::Goban) -> usize {
        let mut counter = 0;
        for n in tree.nodes {
            for t in n.tokens {
                match t {
                    sgf_parser::SgfToken::Move { color: _, action } => {
                        if let sgf_parser::Action::Move(x, y) = action {
                            let stone = match goban.turn.clone() { Player::White => Stone::white(), Player::Black => Stone::black() };
                            crate::Goban::play(goban, Point::new(x as u32 - 1, y as u32 - 1), stone);
                            counter += 1;
                        }
                    },
                    _ => (),
                }
            }
        }

        // it's used but clippy can't see it
        #[allow(unused_assignments)]
        let mut counter2 = 0;
        for v in tree.variations {
            counter2 = History::add_tree_to_history(v, goban);
            for _ in 0..counter2 {
                goban.previous_state();
            }
        }
        counter
    }

    pub fn into_game_tree(&self) -> sgf_parser::GameTree {
        // 0 is always the root node
        self.build_game_tree(0.into()).unwrap()
    }

    fn build_game_tree(&self, index: NodeIndex<u32>) -> Option<GameTree> {
        let mut sgf = GameTree::default();
        if index != 0.into() {
            let Point {mut x, mut y} = crate::Goban::idx_to_coord(self.moves[index].index);
            x += 1;
            y += 1;
            sgf.nodes.push(GameNode { tokens: vec![sgf_parser::SgfToken::Move {
                color: match self.moves[index].player {
                    Player::Black => sgf_parser::Color::Black,
                    Player::White => sgf_parser::Color::White,
                },
                action: sgf_parser::Action::Move(x as u8, y as u8),
            }]});
        } else {
            sgf.nodes.push(self.game_info.clone());
        }
        for n in self.moves.neighbors(index) {
            if let Some(tree) = self.build_game_tree(n) {
                sgf.variations.push(tree);
            }
        }
        Some(sgf)
    }

    pub fn push(&mut self, elem: (Player, usize, Vec<Group>)) -> Result<(), HistoryError> {

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
