use crate::solver::solver_node::active_laser::ActiveLaser;
use crate::solver::solver_node::{SolverNode, SPIRAL_ORDER_REVERSE};
use crate::solver::token::{LaserTokenInteractionResult, Token, TokenType};

#[derive(Clone, Debug)]
pub struct Checker {
    grid: SolverNode,
    // there can be 4 active lasers if 2 perpindicular lasers hit the same beam splitter
    active_lasers: [Option<ActiveLaser>; 4],
    laser_visited: [[bool; 4]; 25],
    unoriented_occupied_cells: Vec<usize>,
    all_lasers_remain_on_board: bool,
}

impl Default for Checker {
    fn default() -> Self {
        let grid: SolverNode = Default::default();
        let active_lasers: [Option<ActiveLaser>; 4] = Default::default();
        let laser_visited: [[bool; 4]; 25] = Default::default();
        let unoriented_occupied_cells: Vec<usize> = Default::default();
        let all_lasers_remain_on_board = true;

        Self {
            grid,
            active_lasers,
            laser_visited,
            unoriented_occupied_cells,
            all_lasers_remain_on_board,
        }
    }
}

impl Checker {
    pub fn check(mut self) -> Self {
        self.initialize();

        while self.has_active_lasers() {
            // inner loop: iterate on lasers and do some work on Some()s until no more active lasers
            let mut new_laser_index = 0;
            let mut new_lasers = [None, None, None, None];
            for laser in self.active_lasers.iter_mut().flatten() {
                // if the laser is still on the board after going to the next position, check for
                // a token. if there's a token, do the interactions.
                // panics if more than 3 active lasers. if this happens it's either an invalid puzzle or programming error..
                if let Some(next_laser_position) = laser.next_position() {
                    if let Some(token) = &mut self.grid.cells[next_laser_position] {
                        // check for unoriented token; if we hit an unoriented token, terminate this laser and save the index
                        if token.orientation().is_none() {
                            new_laser_index += 1;
                            self.unoriented_occupied_cells.push(next_laser_position);
                            continue;
                        }

                        // if the piece is oriented, continue marching the laser
                        for new_laser_direction in token
                            .outbound_lasers_given_inbound_laser_direction(&laser.orientation)
                            .into_iter()
                        {
                            match new_laser_direction {
                                LaserTokenInteractionResult::OutboundLaser(orientation) => {
                                    if self.laser_visited[next_laser_position]
                                        [orientation.to_index()]
                                    {
                                        continue;
                                    }
                                    self.laser_visited[next_laser_position]
                                        [orientation.to_index()] = true;
                                    if new_laser_index > 3 {
                                        println!("panic config: {:?}", self);
                                        panic!("laser index > 3!");
                                    }
                                    let new_active_laser = ActiveLaser {
                                        cell_index: next_laser_position,
                                        orientation,
                                    };
                                    if !new_lasers
                                        .clone()
                                        .into_iter()
                                        .flatten()
                                        .any(|laser| laser == new_active_laser)
                                    {
                                        new_lasers[new_laser_index] = Some(new_active_laser);
                                        new_laser_index += 1;
                                    }
                                }
                                LaserTokenInteractionResult::NoOutboundLaser { valid } => {
                                    match valid {
                                        true => continue,
                                        false => self.all_lasers_remain_on_board = false, // TODO this variable name is now misleading
                                    }
                                }
                            }
                        }
                    } else {
                        self.laser_visited[next_laser_position][laser.orientation.to_index()] =
                            true;
                        if new_laser_index > 3 {
                            println!("panic config: {:?}", self);
                            panic!("laser index > 3!!");
                        }
                        new_lasers[new_laser_index] = Some(ActiveLaser {
                            cell_index: next_laser_position,
                            orientation: laser.orientation.clone(),
                        });
                        new_laser_index += 1;
                    }
                } else {
                    self.all_lasers_remain_on_board = false;
                }
            }
            self.active_lasers = new_lasers;
        }

        self
    }

    fn remaining_tokens_to_be_added(&self) -> bool {
        // Does the associated SolverNode have any tokens that still need to be placed on the grid?
        (!self.grid.tokens_to_be_added.is_empty())
            || (!self.grid.tokens_to_be_added_shuffled.is_empty())
    }

    pub fn generate_branches(mut self) -> Result<[Option<Token>; 25], Vec<SolverNode>> {
        // - march the laser forward until no active lasers
        // - if a laser visits an unoriented token: record the index and terminate that active laser
        // - if the laser visted unoriented tokens: generate new branches for orienting those pieces
        // - if the lasers didn't visit unoriented tokens, and not all tokens are placed,
        //     new branches will be made for placing the next token in any cell the laser visited

        self = self.check();
        if self.solved() {
            self.grid.reset_tokens();
            Ok(self.grid.cells.clone())
        } else {
            self.grid.reset_tokens();
            Err(self.generate_branches_after_check())
        }
    }

    fn generate_branches_after_check(&mut self) -> Vec<SolverNode> {
        if !self.unoriented_occupied_cells.is_empty() {
            // if the laser hit an unoriented token, populate the next branches by setting the orientation of that token
            self.unoriented_occupied_cells
                .iter()
                .flat_map(|cell_index| self.grid.generate_orientation_branches_at_cell(*cell_index))
                .collect::<Vec<SolverNode>>()
        } else if let Some(token) = self.grid.tokens_to_be_added_shuffled.pop() {
            // if the laser only hit oriented tokens, try placing the next token in any of the cells the laser visited but are not occupied by a token
            let empty_cells_with_active_laser = self.empty_cells_with_active_laser();
            let mut result = vec![];
            for i in SPIRAL_ORDER_REVERSE.iter() {
                if !empty_cells_with_active_laser.contains(i) {
                    continue;
                }
                let mut new_node = self.grid.clone();
                new_node.cells[*i] = Some(token.clone());
                result.push(new_node);
            }
            result
        } else {
            // this board isn't solved, and doesn't have any new children
            vec![]
        }
    }

    pub fn from_solver_node(solver_node: SolverNode) -> Self {
        Self {
            grid: solver_node,
            ..Default::default()
        }
    }

    #[allow(dead_code)]
    fn cells_with_active_laser(&self) -> Vec<usize> {
        let mut result = vec![];
        for (idx, cell) in self.laser_visited.into_iter().enumerate() {
            if cell[0] || cell[1] || cell[2] || cell[3] {
                result.push(idx);
            }
        }
        result
    }

    // return the indices of cells where the laser has visited but there is no token
    fn empty_cells_with_active_laser(&self) -> Vec<usize> {
        let mut result = vec![];
        for (idx, cell) in self.laser_visited.into_iter().enumerate() {
            if self.grid.cells[idx].is_none() && (cell[0] || cell[1] || cell[2] || cell[3]) {
                result.push(idx);
            }
        }
        result
    }

    fn has_active_lasers(&self) -> bool {
        self.active_lasers.iter().any(|laser| laser.is_some())
    }

    pub fn solved(&self) -> bool {
        self.grid.targets == self.count_lit_targets()
            && self.all_required_targets_lit()
            && self.all_tokens_lit()
            && self.all_lasers_remain_on_board
            && !self.remaining_tokens_to_be_added()
    }

    fn count_lit_targets(&self) -> u8 {
        self.grid
            .cells
            .iter()
            .filter(|cell| {
                if let Some(token) = cell {
                    token.target_lit().unwrap_or(false)
                } else {
                    false
                }
            })
            .count() as u8
    }

    fn all_required_targets_lit(&self) -> bool {
        self.grid
            .cells
            .iter()
            .filter_map(|cell| {
                if let Some(token) = cell {
                    if token.must_light() {
                        Some(
                            token
                                .target_lit()
                                .expect("Only a target should have must_light = true"),
                        )
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .all(|b| b)
    }

    fn all_tokens_lit(&self) -> bool {
        self.grid.cells.iter().flatten().all(|token| token.lit)
    }

    // Find the laser piece and set initialize the active laser there
    fn initialize(&mut self) {
        for i in 0..25 {
            if let Some(token) = &self.grid.cells[i] {
                if token.type_() == &TokenType::Laser {
                    self.laser_visited[i][token
                        .orientation()
                        .expect("Tried running checker on piece without orientation set")
                        .to_index()] = true;
                    let initial_active_laser = ActiveLaser {
                        orientation: token
                            .orientation()
                            .expect("Tried running checker on piece without orientation set")
                            .clone(),
                        cell_index: i,
                    };
                    self.active_lasers[0] = Some(initial_active_laser);
                    return;
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::solver::orientation::{self, Orientation};

    #[test]
    fn test_solver_puzzle_62_debug() {
        // The solver is struggling on Bonus Challenge 2, because the puzzle is "Completed"
        // before placing the last BeamSplitter

        // This is the last node before the solver claims it's "done". The puzzle is "solved" (2 targets
        // light, no lasers go off board), but there is still a remaining token to be added!
        let node = SolverNode {
            cells: [
                Some(Token::new(
                    TokenType::TargetMirror,
                    Some(Orientation::North),
                    false,
                )),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(Token::new(TokenType::Laser, Some(Orientation::East), false)),
                None,
                Some(Token::new(
                    TokenType::BeamSplitter,
                    Some(Orientation::East),
                    false,
                )),
                Some(Token::new(
                    TokenType::DoubleMirror,
                    Some(Orientation::East),
                    false,
                )),
                Some(Token::new(
                    TokenType::TargetMirror,
                    Some(Orientation::West),
                    false,
                )),
                None,
                Some(Token::new(
                    TokenType::Checkpoint,
                    Some(Orientation::East),
                    false,
                )),
                None,
                Some(Token::new(
                    TokenType::TargetMirror,
                    Some(Orientation::North),
                    false,
                )),
                None,
                None,
                Some(Token::new(
                    TokenType::TargetMirror,
                    Some(Orientation::East),
                    false,
                )),
                Some(Token::new(
                    TokenType::TargetMirror,
                    Some(Orientation::North),
                    false,
                )),
                None,
            ],
            tokens_to_be_added: vec![],
            tokens_to_be_added_shuffled: vec![Token::new(TokenType::BeamSplitter, None, false)],
            targets: 2,
        };
        let checker = node.check();
        println!("Checker after running node.check():\n{:?}\n---", checker);
        assert!(checker.remaining_tokens_to_be_added());
        assert!(!checker.solved());
    }

    #[test]
    fn test_checker_simple() {
        let node = SolverNode {
            cells: [
                Some(Token::new(TokenType::Laser, Some(Orientation::East), false)),
                Some(Token::new(
                    TokenType::BeamSplitter,
                    Some(Orientation::West),
                    false,
                )),
                Some(Token::new(
                    TokenType::TargetMirror,
                    Some(Orientation::West),
                    false,
                )),
                None,
                None,
                None,
                Some(Token::new(
                    TokenType::TargetMirror,
                    Some(Orientation::South),
                    false,
                )),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
            tokens_to_be_added: vec![],
            tokens_to_be_added_shuffled: vec![],
            targets: 2,
        };
        let checker = node.check();
        assert!(checker.solved());
    }
}
