use crate::{
    orientation::Orientation,
    token::{Token, TokenType},
};
mod active_laser;
use active_laser::ActiveLaser;
use lazy_static::lazy_static;

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
            for i in SPIRAL_ORDER.iter() {
                if self.cells[*i].is_none() {
                    let mut new_node = self.clone();
                    new_node.cells[*i] = Some(token.clone());
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
                    for x in token.orientation_range() {
                        let mut new_node = self.clone();
                        new_node.cells[i]
                            .as_mut()
                            .expect("We just validated there is a token in this cell")
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
                                cell_index: next_laser_position,
                                orientation: new_laser_direction,
                            });
                            new_laser_index += 1;
                        }
                    } else {
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
                        cell_index: i,
                    };
                    self.active_lasers[0] = Some(initial_active_laser);
                    return;
                }
            }
        }
    }

    // returns an array representing the out-of-board orientations
    fn is_edge_cell(&self, cell_index: usize) -> [Option<Orientation>; 2] {
        // the center cannot be considered an edge piece, regardless of the cell blocker's location
        if cell_index == 12 {
            return [None, None];
        }

        // we need to check the cell blocker first because edge pieces can have a different result from this
        // function if the cell blocker is on a corner
        if let Some((cell_blocker_index, _)) =
            self.cells
                .as_ref()
                .into_iter()
                .enumerate()
                .find(|(_, token)| {
                    if let Some(token) = token {
                        token.type_() == &TokenType::CellBlocker
                    } else {
                        false
                    }
                })
        {
            // neighboring_cell_indices are the cell(s) neighboring the blocker we need to check
            let neighboring_cell_indices = match cell_blocker_index {
                // corners
                0 => [Some(1), Some(5)],
                4 => [Some(3), Some(9)],
                20 => [Some(15), Some(21)],
                24 => [Some(23), Some(19)],
                // edges, but not a corner
                1 => [Some(6), None],
                2 => [Some(7), None],
                3 => [Some(8), None],
                9 => [Some(8), None],
                14 => [Some(13), None],
                19 => [Some(18), None],
                23 => [Some(18), None],
                22 => [Some(17), None],
                21 => [Some(16), None],
                15 => [Some(16), None],
                10 => [Some(11), None],
                5 => [Some(6), None],
                // cell blocker is not on an edge
                _ => [None, None],
            };
            if neighboring_cell_indices
                .into_iter()
                .flatten()
                .collect::<Vec<usize>>()
                .contains(&cell_index)
            {
                // now, we know that the token is impacted by the cell blocker.
                // if the cell blocker is on a non-corner edge, it's unambiguous which direction the laser cannot face
                if NORTH_EDGE_CELL_INDICES.contains(&cell_blocker_index) {
                    return [Some(Orientation::North), None];
                }
                if EAST_EDGE_CELL_INDICES.contains(&cell_blocker_index) {
                    return [Some(Orientation::East), None];
                }
                if SOUTH_EDGE_CELL_INDICES.contains(&cell_blocker_index) {
                    return [Some(Orientation::South), None];
                }
                if WEST_EDGE_CELL_INDICES.contains(&cell_blocker_index) {
                    return [Some(Orientation::West), None];
                }
                // if we reach this point, the cell blocker is on a corner, AND the piece is on an edge neighboring that corner
                match cell_index {
                    1 => return [Some(Orientation::South), Some(Orientation::West)],
                    3 => return [Some(Orientation::South), Some(Orientation::East)],
                    9 => return [Some(Orientation::South), Some(Orientation::East)],
                    19 => return [Some(Orientation::North), Some(Orientation::East)],
                    23 => return [Some(Orientation::North), Some(Orientation::East)],
                    21 => return [Some(Orientation::North), Some(Orientation::West)],
                    15 => return [Some(Orientation::North), Some(Orientation::West)],
                    5 => return [Some(Orientation::South), Some(Orientation::West)],
                    _ => panic!("Logical error in is_edge_cell()"),
                }
            }
        }

        // now we know the cell blocker is not on the edge

        // corners
        if cell_index == 0 {
            return [Some(Orientation::South), Some(Orientation::West)];
        }
        if cell_index == 4 {
            return [Some(Orientation::South), Some(Orientation::East)];
        }
        if cell_index == 20 {
            return [Some(Orientation::North), Some(Orientation::West)];
        }
        if cell_index == 24 {
            return [Some(Orientation::North), Some(Orientation::East)];
        }
        // edges, but not on corner
        if NORTH_EDGE_CELL_INDICES.contains(&cell_index) {
            return [Some(Orientation::North), None];
        }
        if EAST_EDGE_CELL_INDICES.contains(&cell_index) {
            return [Some(Orientation::East), None];
        }
        if SOUTH_EDGE_CELL_INDICES.contains(&cell_index) {
            return [Some(Orientation::South), None];
        }
        if WEST_EDGE_CELL_INDICES.contains(&cell_index) {
            return [Some(Orientation::West), None];
        }

        [None, None]
    }
}

lazy_static! {
    static ref SPIRAL_ORDER: [usize; 25] = [
        0, 1, 2, 3, 4, 9, 14, 19, 24, 23, 22, 21, 20, 15, 10, 5, 6, 7, 8, 13, 18, 17, 16, 11, 12,
    ];
}

lazy_static! {
    static ref EDGE_CELL_INDICES: [usize; 16] =
        [0, 1, 2, 3, 4, 9, 14, 19, 24, 23, 22, 21, 20, 15, 10, 5,];
}

lazy_static! {
    static ref NORTH_EDGE_CELL_INDICES: [usize; 3] = [21, 22, 23,];
}

lazy_static! {
    static ref EAST_EDGE_CELL_INDICES: [usize; 3] = [9, 14, 19,];
}

lazy_static! {
    static ref SOUTH_EDGE_CELL_INDICES: [usize; 3] = [1, 2, 3,];
}

lazy_static! {
    static ref WEST_EDGE_CELL_INDICES: [usize; 3] = [5, 10, 15,];
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

    #[test]
    fn test_edge_detect() {
        // test cell blocker on top right corner
        let mut cells: [Option<Token>; 25] = Default::default();
        cells[24] = Some(Token::new(TokenType::CellBlocker, None, false));
        let solver = SolverNode::new(cells, vec![], 1);
        assert_eq!(
            [Some(Orientation::North), Some(Orientation::East)],
            solver.is_edge_cell(19)
        );
        // test piece away from cell blocker or edge
        assert_eq!([None, None], solver.is_edge_cell(18));
        // test piece on edge
        assert_eq!([Some(Orientation::West), None], solver.is_edge_cell(10));
        // test piece on corner
        assert_eq!(
            [Some(Orientation::South), Some(Orientation::West)],
            solver.is_edge_cell(0)
        );
        // test center
        assert_eq!([None, None], solver.is_edge_cell(12));

        // test cell blocker on non-corner edge with piece neighboring
        let mut cells: [Option<Token>; 25] = Default::default();
        cells[3] = Some(Token::new(TokenType::CellBlocker, None, false));
        let solver = SolverNode::new(cells, vec![], 1);
        assert_eq!([Some(Orientation::South), None], solver.is_edge_cell(8));
    }
}
