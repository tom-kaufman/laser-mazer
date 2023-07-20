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
    active_lasers: Vec<ActiveLaser>,
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

    pub fn check(mut self) -> bool {
        self.initialize();
        // outer loop: keep cranking the laser states until there are no more lasers
        while !self.active_lasers.is_empty() {
            // println!("{:?}\n\n", &self.active_lasers);
            // sanity check: only 3 lasers active at most
            assert!(self.active_lasers.len() <= 3);
            // inner loop: pop lasers and do some work on them until no more active lasers
            let mut new_active_lasers = vec![];
            while !self.active_lasers.is_empty() {
                let laser = self
                    .active_lasers
                    .pop()
                    .expect("We just checked the vec isn't empty");
                // if the laser is still on the board after going to the next position, check for
                // a token. if there's a token, do the interactions.
                if let Some(next_laser_position) = laser.next_position() {
                    // if the laser hits a new Token, calculate the results of the laser's interaction with the token
                    // otherwise, just make a new active laser at this spot
                    if let Some(token) = &mut self.cells[next_laser_position] {
                        let new_laser_directions =
                            token.outbound_lasers_given_inbound_laser_direction(&laser.orientation);
                        for new_laser_direction in new_laser_directions {
                            new_active_lasers.push(ActiveLaser {
                                slot_index: next_laser_position,
                                orientation: new_laser_direction,
                            });
                        }
                    } else {
                        new_active_lasers.push(ActiveLaser {
                            slot_index: next_laser_position,
                            orientation: laser.orientation.clone(),
                        })
                    }
                }
            }
            self.active_lasers.extend(new_active_lasers);
        }

        self.targets == self.count_lit_targets() && self.all_required_targets_lit()
    }

    fn count_lit_targets(&self) -> u8 {
        self.cells
            .iter()
            .filter(|cell| {
                if let Some(token) = cell {
                    token.target_lit().unwrap_or_else(|| false)
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
                    self.active_lasers.push(initial_active_laser);
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
