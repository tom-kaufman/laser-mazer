use crate::solver_node::active_laser::ActiveLaser;
use crate::solver_node2::SolverNode2;
use crate::token::{Token, TokenType};

#[derive(Clone, Default)]
pub struct Checker {
    grid: SolverNode2,
    // there can be 4 active lasers if 2 perpindicular lasers hit the same beam splitter
    active_lasers: [Option<ActiveLaser>; 4],
    laser_visited: [[bool; 4]; 25],
}

impl Checker {
    pub fn check(self) -> Self {
        todo!()
    }

    pub fn from_solver_node(solver_node: SolverNode2) -> Self {
        Self {
            grid: solver_node,
            ..Default::default()
        }
    }

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
