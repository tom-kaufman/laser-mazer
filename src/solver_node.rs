use crate::{
    orientation::Orientation,
    token::{Token, TokenType},
};
mod active_laser;
use active_laser::ActiveLaser;
use lazy_static::lazy_static;

#[derive(Clone, Default, Debug)]
pub struct SolverNode {
    // TODO revert away from pub
    pub cells: [Option<Token>; 25],
    tokens_to_be_added: Vec<Token>,
    tokens_to_be_added_shuffled: Vec<Token>,
    laser_visited: [[bool; 4]; 25],
    // there can be 4 active lasers if 2 perpindicular lasers hit the same beam splitter
    active_lasers: [Option<ActiveLaser>; 4],
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

    pub fn generate_branches_laser_aware(&mut self) -> Vec<Self> {
        if !self.laser_placed() {
            // the laser will be sorted to be at the top of the vec, so this will place the laser
            println!("Placing laser");
            return self.generate_token_placement_branches();
        }
        if !self.all_placed_tokens_have_orientation_set() {
            // if not all pieces on the grid have their rotation set, we want to set them first
            println!("Setting orientations of tokens already on grid");
            return self.generate_rotation_setting_branches();
        }
        // now, we have the laser placed and all orientations of pieces on the board set
        // next, we need to shuffle the pieces to be added
        if !self.tokens_to_be_added.is_empty() {
            println!("Shuffling tokens to be added");
            return self.generate_shuffled_tokens_to_be_added_branches();
        }
        if !self.tokens_to_be_added_shuffled.is_empty() {
            // we now need to place pieces on the grid such that they interact with the laser
            println!("Placing a token somewhere in the path of the laser");
            return self.generate_token_placement_branches_laser_aware();
        }

        // if we reach this point, we are at a leaf
        println!("At a leaf!");
        vec![]
    }

    fn laser_placed(&self) -> bool {
        self.cells
            .as_ref()
            .into_iter()
            .flatten()
            .any(|token| token.type_() == &TokenType::Laser)
    }

    fn all_placed_tokens_have_orientation_set(&self) -> bool {
        self.cells
            .as_ref()
            .into_iter()
            .flatten()
            .all(|token| token.orientation().is_some())
    }

    fn count_tokens_to_be_added_by_type(&self, type_: TokenType) -> usize {
        self.tokens_to_be_added
            .iter()
            .filter(|token| token.type_() == &type_)
            .count()
    }

    fn count_must_light_tokens_to_be_added(&self) -> usize {
        self.tokens_to_be_added
            .iter()
            .filter(|token| token.must_light())
            .count()
    }

    fn generate_shuffled_tokens_to_be_added_branches(&mut self) -> Vec<Self> {
        // because cell blockers may not be in the vec of tokens to be added, and we onyl call this function once the laser has been placed,
        // we may only have TargetMirrors, Checkpoints, DoubleMirrors, and BeamSplitters. Given that we have t, c, d, and b of each,
        // we will generate this many shufflings: (t+c+d+b)!/(t!c!d!b!). In the worst case, we'll have t=5, c=1, d=1, b=2, generating
        // (5+1+1+2)!/(5!2!) = 1512 shufflings.

        // recursively build a list of the unique permutations
        fn backtrack(
            n_target_mirrors_must_light: usize,
            n_target_mirrors_may_not_light: usize,
            n_checkpoints: usize,
            n_double_mirrors: usize,
            n_beam_splitters: usize,
            current_ordering: Vec<Token>,
            unique_orderings: &mut Vec<Vec<Token>>,
        ) {
            if n_target_mirrors_must_light == 0
                && n_target_mirrors_may_not_light == 0
                && n_checkpoints == 0
                && n_double_mirrors == 0
                && n_beam_splitters == 0
            {
                unique_orderings.push(current_ordering);
                return;
            }

            if n_target_mirrors_must_light > 0 {
                let mut new_ordering = current_ordering.clone();
                new_ordering.push(Token::new(TokenType::TargetMirror, None, true));
                backtrack(
                    n_target_mirrors_must_light - 1,
                    n_target_mirrors_may_not_light,
                    n_checkpoints,
                    n_double_mirrors,
                    n_beam_splitters,
                    new_ordering,
                    unique_orderings,
                );
            }

            if n_target_mirrors_may_not_light > 0 {
                let mut new_ordering = current_ordering.clone();
                new_ordering.push(Token::new(TokenType::TargetMirror, None, false));
                backtrack(
                    n_target_mirrors_must_light,
                    n_target_mirrors_may_not_light - 1,
                    n_checkpoints,
                    n_double_mirrors,
                    n_beam_splitters,
                    new_ordering,
                    unique_orderings,
                );
            }

            if n_checkpoints > 0 {
                let mut new_ordering = current_ordering.clone();
                new_ordering.push(Token::new(TokenType::Checkpoint, None, false));
                backtrack(
                    n_target_mirrors_must_light,
                    n_target_mirrors_may_not_light,
                    n_checkpoints - 1,
                    n_double_mirrors,
                    n_beam_splitters,
                    new_ordering,
                    unique_orderings,
                );
            }

            if n_double_mirrors > 0 {
                let mut new_ordering = current_ordering.clone();
                new_ordering.push(Token::new(TokenType::DoubleMirror, None, false));
                backtrack(
                    n_target_mirrors_must_light,
                    n_target_mirrors_may_not_light,
                    n_checkpoints,
                    n_double_mirrors - 1,
                    n_beam_splitters,
                    new_ordering,
                    unique_orderings,
                );
            }

            if n_beam_splitters > 0 {
                let mut new_ordering = current_ordering.clone();
                new_ordering.push(Token::new(TokenType::BeamSplitter, None, false));
                backtrack(
                    n_target_mirrors_must_light,
                    n_target_mirrors_may_not_light,
                    n_checkpoints,
                    n_double_mirrors,
                    n_beam_splitters - 1,
                    new_ordering,
                    unique_orderings,
                );
            }
        }

        let n_target_mirrors_must_light = self.count_must_light_tokens_to_be_added();
        let n_target_mirrors_may_not_light = self
            .count_tokens_to_be_added_by_type(TokenType::TargetMirror)
            - n_target_mirrors_must_light;
        let n_checkpoints = self.count_tokens_to_be_added_by_type(TokenType::Checkpoint);
        let n_double_mirrors = self.count_tokens_to_be_added_by_type(TokenType::DoubleMirror);
        let n_beam_splitters = self.count_tokens_to_be_added_by_type(TokenType::BeamSplitter);

        let mut unique_orderings: Vec<Vec<Token>> = vec![];
        let current_ordering: Vec<Token> = vec![];

        backtrack(
            n_target_mirrors_must_light,
            n_target_mirrors_may_not_light,
            n_checkpoints,
            n_double_mirrors,
            n_beam_splitters,
            current_ordering,
            &mut unique_orderings,
        );

        // println!("Found {} unique orderings of the tokens to be added", unique_orderings.len());

        let mut result = vec![];

        for unique_ordering in unique_orderings {
            let mut new_node = self.clone();
            new_node.tokens_to_be_added = vec![];
            new_node.tokens_to_be_added_shuffled = unique_ordering;
            result.push(new_node);
        }

        result
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
            if self.cells[idx].is_none() && (cell[0] || cell[1] || cell[2] || cell[3]) {
                result.push(idx);
            }
        }
        result
    }

    fn generate_token_placement_branches_laser_aware(&mut self) -> Vec<Self> {
        // this function will make a copy of the node and march the laser forward, get a list of indices with the laser over it but no token present,
        // and create nodes with the next token to be placed in each of those locations
        // orientation will get set next time we go back to generate_branches_laser_aware()!
        let mut result = vec![];

        if let Some(token) = self.tokens_to_be_added_shuffled.pop() {
            // println!("Got a token of type {:?} of the shuffled vec of tokens to be added", token.type_());
            let empty_cells_with_active_laser =
                self.clone().check().empty_cells_with_active_laser();
            // println!("These cells are empty and have been visited by the laser: {:?}", empty_cells_with_active_laser);
            for i in SPIRAL_ORDER.iter() {
                if !empty_cells_with_active_laser.contains(i) {
                    continue;
                }
                // println!("Generating a new node with the token placed at cell {i}");
                let mut new_node = self.clone();
                new_node.cells[*i] = Some(token.clone());
                result.push(new_node)
            }
        }

        result
    }

    fn generate_token_placement_branches(&mut self) -> Vec<Self> {
        if let Some(token) = self.tokens_to_be_added.pop() {
            let mut result = vec![];
            for i in SPIRAL_ORDER.iter() {
                if self.cells[*i].is_none() {
                    let mut new_node = self.clone();
                    new_node.cells[*i] = Some(token.clone());
                    if *i == 14 && token.type_() == &TokenType::Laser {
                        // TODO delete me
                        println!(
                            "Placed laser correctly for puzzle 50, orientation = {:?}",
                            token.orientation()
                        );
                    }
                    result.push(new_node)
                }
            }
            result
        } else {
            vec![]
        }
    }

    // for generating rotation branches, which rotations are valid?
    fn orientation_iter(&self, token_type: &TokenType, cell_index: usize) -> Vec<usize> {
        let mut result = token_type.orientation_range();

        // if the token can point out of the board, directly return this token type's orientation range
        if [
            TokenType::BeamSplitter,
            TokenType::DoubleMirror,
            TokenType::CellBlocker,
        ]
        .contains(token_type)
        {
            return result;
        }
        // otherwise, we need to know if this piece is on an edge
        let mut forbidden_directions = self
            .forbidden_orientations(cell_index)
            .into_iter()
            .flatten()
            .map(|o| o.to_index())
            .collect::<Vec<usize>>();

        match token_type {
            // the laser has no symmetry so we can directly use forbidden_directions to prune the result
            TokenType::Laser => {
                result.retain(|orientation_idx| !forbidden_directions.contains(orientation_idx));
                result
            }
            // the checkpoint has 180 degree symmetry
            TokenType::Checkpoint => {
                for idx in forbidden_directions.iter_mut() {
                    if *idx > 1 {
                        *idx -= 2;
                    }
                }
                result.retain(|orientation_idx| !forbidden_directions.contains(orientation_idx));
                result
            }
            // the target mirror is more complicated. we must consider if this target must be lit,
            // how many target mirrors are lightable,
            TokenType::TargetMirror => {
                self.target_mirror_orientation_iter(forbidden_directions, cell_index)
            }
            _ => {
                // this should be unreachable
                result
            }
        }
    }

    fn n_targets_which_must_be_lit(&self) -> u8 {
        self.cells
            .as_ref()
            .into_iter()
            .flatten()
            .filter(|token| {
                // only TargetMirrors can be constructed with must_light = true, so no need to check token type
                token.must_light()
            })
            .count() as u8
    }

    fn n_targets_which_may_not_be_lit_and_accessible_or_not_oriented(&self) -> u8 {
        self.cells.as_ref().into_iter().enumerate().filter(|(idx, token)| {
            if let Some(token) = token {
                let forbidden_directions: Vec<usize> = self.forbidden_orientations(*idx).into_iter().flatten().map(|o| {o.to_index()}).collect::<Vec<usize>>();
                (token.type_() == &TokenType::TargetMirror) && !token.must_light() && (token.orientation().is_none() || !forbidden_directions.contains(&token.orientation().expect("won't enter this branch of or statement if orientation is None").to_index()))
            } else {
                false
            }
        }).count() as u8
    }

    fn target_mirror_orientation_iter(
        &self,
        forbidden_directions: Vec<usize>,
        cell_index: usize,
    ) -> Vec<usize> {
        let mut result = vec![0, 1, 2, 3];
        // if this token must be lit, it cannot be inaccessible
        if let Some(target_mirror_token) = &self.cells[cell_index] {
            if !(target_mirror_token.type_() == &TokenType::TargetMirror) {
                panic!(
                    "Tried checking target mirror rotations on a cell not holding a target mirror"
                )
            }
            if target_mirror_token.must_light() {
                result.retain(|orientation_idx| !forbidden_directions.contains(orientation_idx));
                return result;
            }
        } else {
            panic!("Tried checking target mirror rotations on a cell not holding a target mirror")
        }

        // first, subtract the number of targets which must be lit from total number of targets
        // if we have more than that difference of targets which may not be lit,
        // then this piece may or may not point out of the board. but if we have less than or equal to that
        // difference, we must make this target accessible.
        // TODO debug why this causes puzzle 40 to not solve (TargetMirror at slot 1 can't face south)
        // if self.targets - self.n_targets_which_must_be_lit()
        //     <= self.n_targets_which_may_not_be_lit_and_accessible_or_not_oriented()
        // {
        //     result.retain(|orientation_idx| !forbidden_directions.contains(orientation_idx));
        // }

        result
    }

    fn generate_rotation_setting_branches(&mut self) -> Vec<Self> {
        for i in SPIRAL_ORDER.iter() {
            if let Some(token) = &self.cells[*i] {
                if token.orientation().is_none() {
                    let mut result = vec![];
                    if *i == 14 && token.type_() == &TokenType::Laser {
                        // TODO delete me
                        let x = self.orientation_iter(token.type_(), *i);
                        println!(
                            "Configuring rotation of laser in slot 14, orientation indices = {:?}",
                            x
                        );
                    }
                    for x in self.orientation_iter(token.type_(), *i) {
                        let mut new_node = self.clone();
                        new_node.cells[*i]
                            .as_mut()
                            .expect("We just validated there is a token in this cell")
                            .orientation = Some(Orientation::from_index(x));
                        // if *i == 14 && new_node.cells[*i].as_ref().unwrap().type_() == &TokenType::Laser && new_node.cells[*i].as_ref().unwrap().orientation == Some(Orientation::West) {
                        //     // TODO delete me
                        //     println!("Laser oriented correctly for puzzle 50");
                        // }
                        // if *i == 11 && new_node.cells[*i].as_ref().unwrap().type_() == &TokenType::BeamSplitter && new_node.cells[*i].as_ref().unwrap().orientation == Some(Orientation::East) {
                        //     // TODO delete me
                        //     println!("Beam splitter on slot 11 oriented correctly for puzzle 50");
                        // }
                        // if *i == 10 && new_node.cells[*i].as_ref().unwrap().type_() == &TokenType::TargetMirror && new_node.cells[*i].as_ref().unwrap().orientation == Some(Orientation::South) {
                        //     // TODO delete me
                        //     println!("Target on slot 10 oriented correctly for puzzle 50");
                        // }
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

    pub fn check(mut self) -> Self {
        self.initialize();

        // outer loop: keep cranking the laser states until there are no more lasers
        while self.has_active_lasers() {
            // println!("active lasers: {:?}", self.active_lasers);
            // println!("visited lasers: {:?}", self.laser_visited);
            // inner loop: iterate on lasers and do some work on Some()s until no more active lasers
            let mut new_laser_index = 0;
            let mut new_lasers = [None, None, None, None];
            for laser in self.active_lasers.iter_mut().flatten() {
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
                            if self.laser_visited[next_laser_position]
                                [new_laser_direction.to_index()]
                            {
                                // println!("Laser is going in a loop!");
                                continue;
                            }
                            // println!("setting indices to true (hit piece): self.laser_visited[{}][{}]", next_laser_position, new_laser_direction.to_index());
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
                                .collect::<Vec<ActiveLaser>>()
                                .contains(&new_active_laser)
                            {
                                new_lasers[new_laser_index] = Some(new_active_laser);
                                new_laser_index += 1;
                            }
                        }
                    } else {
                        // println!("setting indices to true (empty cell): self.laser_visited[{}][{}]", next_laser_position, laser.orientation.to_index());
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

    pub fn solved(&self) -> bool {
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
    // TODO this should also check for neighboring pieces which block the laser path (i.e. checkpoint feeding into the wall of a target)
    fn forbidden_orientations(&self, cell_index: usize) -> [Option<Orientation>; 2] {
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
            solver.forbidden_orientations(19)
        );
        // test piece away from cell blocker or edge
        assert_eq!([None, None], solver.forbidden_orientations(18));
        // test piece on edge
        assert_eq!(
            [Some(Orientation::West), None],
            solver.forbidden_orientations(10)
        );
        // test piece on corner
        assert_eq!(
            [Some(Orientation::South), Some(Orientation::West)],
            solver.forbidden_orientations(0)
        );
        // test center
        assert_eq!([None, None], solver.forbidden_orientations(12));

        // test cell blocker on non-corner edge with piece neighboring
        let mut cells: [Option<Token>; 25] = Default::default();
        cells[3] = Some(Token::new(TokenType::CellBlocker, None, false));
        let solver = SolverNode::new(cells, vec![], 1);
        assert_eq!(
            [Some(Orientation::South), None],
            solver.forbidden_orientations(8)
        );
    }
}
