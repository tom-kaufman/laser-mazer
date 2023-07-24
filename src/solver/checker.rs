use crate::solver::solver_node::active_laser::ActiveLaser;
use crate::solver::solver_node::{SolverNode, SPIRAL_ORDER};
use crate::solver::token::{Token, TokenType};

#[derive(Clone, Default, Debug)]
pub struct Checker {
    grid: SolverNode,
    // there can be 4 active lasers if 2 perpindicular lasers hit the same beam splitter
    active_lasers: [Option<ActiveLaser>; 4],
    laser_visited: [[bool; 4]; 25],
    unoriented_occupied_cells: Vec<usize>,
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
                            .flatten()
                        {
                            if self.laser_visited[next_laser_position]
                                [new_laser_direction.to_index()]
                            {
                                continue;
                            }
                            self.laser_visited[next_laser_position]
                                [new_laser_direction.to_index()] = true;
                            if new_laser_index > 3 {
                                println!("panic config: {:?}", self);
                                panic!("laser index > 3!");
                            }
                            let new_active_laser = ActiveLaser {
                                cell_index: next_laser_position,
                                orientation: new_laser_direction,
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
                }
            }
            self.active_lasers = new_lasers;
        }

        self
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
            for i in SPIRAL_ORDER.iter() {
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
        self.grid.targets == self.count_lit_targets() && self.all_required_targets_lit()
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
