use crate::{
    orientation::Orientation,
    token::{Token, TokenType},
};
mod active_laser;
use active_laser::ActiveLaser;

#[derive(Clone, Default, Debug)]
pub struct SolverNode {
    cells: [Option<Token>; 25],
    tokens_to_be_added: Vec<Token>,
    laser_visited: [[bool; 4]; 25],
    active_lasers: [Option<ActiveLaser>; 3],
    targets: u8,
}

impl SolverNode {
    pub fn new(
        initial_grid_config: [Option<Token>; 25],
        tokens_to_be_added: Vec<Token>,
        targets: u8,
    ) -> Self {
        Self {
            cells: initial_grid_config,
            tokens_to_be_added,
            targets,
            ..Default::default()
        }
    }
    pub fn generate_branches(&mut self) -> Vec<Self> {
        let placement_branches = self.generate_token_placement_branches();
        if placement_branches.is_empty() {
            self.generate_rotation_setting_branches()
        } else {
            placement_branches
        }
    }

    fn generate_token_placement_branches(&mut self) -> Vec<Self> {
        if let Some(token) = self.tokens_to_be_added.pop() {
            let mut result = vec![];
            // TODO HEURISTIC use spiral order
            for i in 0..25 {
                if self.cells[i].is_none() {
                    let mut new_node = self.clone();
                    new_node.cells[i] = Some(token.clone());
                    result.push(new_node)
                }
            }
            result
        } else {
            vec![]
        }
    }

    fn generate_rotation_setting_branches(&mut self) -> Vec<Self> {
        for i in 0..25 {
            if let Some(token) = &self.cells[i] {
                if token.orientation().is_none() {
                    let mut result = vec![];
                    for x in 0..4 {
                        let mut new_node = self.clone();
                        new_node.cells[i]
                            .as_mut()
                            .expect("We just validated there is a token in this slot")
                            .orientation = Some(Orientation::from_index(x));
                        result.push(new_node);
                    }
                    return result;
                }
            }
        }

        vec![]
    }

    pub fn clone_cells(&self) -> [Option<Token>; 25] {
        self.cells.clone()
    }

    fn has_active_lasers(&self) -> bool {
        self.active_lasers.iter().any(|laser| laser.is_some())
    }

    pub fn check(mut self) -> bool {
        self.initialize();
        // outer loop: keep cranking the laser states until there are no more lasers
        while self.has_active_lasers() {
            // inner loop: iterate on lasers and do some work on Some()s until no more active lasers
            let mut new_laser_index = 0;
            let mut new_lasers = [None, None, None];
            for laser in self.active_lasers.into_iter().flatten() {
                // if the laser is still on the board after going to the next position, check for
                // a token. if there's a token, do the interactions.
                // panics if more than 3 active lasers. if this happens it's either an invalid puzzle or programming error..
                if let Some(next_laser_position) = laser.next_position() {
                    if let Some(token) = &mut self.cells[next_laser_position] {
                        for new_laser_direction in token
                            .outbound_lasers_given_inbound_laser_direction(&laser.orientation)
                            .into_iter()
                            .flatten()
                        {
                            new_lasers[new_laser_index] = Some(ActiveLaser {
                                slot_index: next_laser_position,
                                orientation: new_laser_direction,
                            });
                            new_laser_index += 1;
                        }
                    } else {
                        new_lasers[new_laser_index] = Some(ActiveLaser {
                            slot_index: next_laser_position,
                            orientation: laser.orientation.clone(),
                        });
                        new_laser_index += 1;
                    }
                }
            }
            self.active_lasers = new_lasers;
        }

        self.targets == self.count_lit_targets() && self.all_required_targets_lit()
    }

    fn count_lit_targets(&self) -> u8 {
        self.cells
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
        self.cells
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
            if let Some(token) = &self.cells[i] {
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
                        slot_index: i,
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
    use std::mem;

    #[test]
    fn mem_sizes() {
        let solver_node = mem::size_of::<SolverNode>();
        println!("SolverNode has size {solver_node}");
    }
}
