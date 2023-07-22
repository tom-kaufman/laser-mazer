use crate::checker::Checker;
use crate::orientation::Orientation;
use crate::token::{Token, TokenType};
use lazy_static::lazy_static;

#[derive(Clone, Default, Debug)]
pub struct SolverNode2 {
    pub cells: [Option<Token>; 25],
    pub tokens_to_be_added: Vec<Token>,
    pub tokens_to_be_added_shuffled: Vec<Token>,
    pub targets: u8,
}

impl SolverNode2 {
    // returns Ok() if we hit the solution, or Err(new_nodes) otherwise
    pub fn generate_branches(&mut self) -> Result<[Option<Token>; 25], Vec<Self>> {
        // place the laser if it's not been added to the grid and rotated
        if !self.laser_placed_and_rotated() {
            return Err(self.generate_laser_placement_branches());
        }

        // next, shuffle the remaining pieces to be added
        if !self.tokens_to_be_added.is_empty() {
            return Err(self.generate_shuffled_tokens_to_be_added_branches());
        }

        // now, make a checker. it will march the laser forward.
        // it will return Ok() if we hit the solution, or Err(new_nodes) otherwise
        self.clone_to_checker().check().generate_branches()
    }

    fn generate_laser_placement_branches(&mut self) -> Vec<Self> {
        if self.laser_placed_and_rotated() {
            // (we shouldn't enter this branch) the laser is already placed and rotated so no branches
            vec![]
        } else if self.laser_placed() {
            // the laser has been placed but not rotated, so we just need orientation branches for the laser
            self.generate_orientation_branches_at_cell(
                self.laser_position()
                    .expect("We just validated that the laser is placed"),
            )
        } else {
            // the laser hasn't been placed or rotated
            // get the laser out of tokens_to_be_added
            self.tokens_to_be_added
                .retain(|token| token.type_() != &TokenType::Laser);
            let laser = Token::new(TokenType::Laser, None, false);
            let mut result = vec![];
            for i in SPIRAL_ORDER.iter() {
                // find all unoccupied cells
                if self.cells[*i].is_none() {
                    // make a copy of this node, place the laser token in this unoccupied slot, and make new nodes for all the orientations of the laser
                    let mut new_node = self.clone();
                    new_node.cells[*i] = Some(laser.clone());
                    let new_nodes = new_node.generate_orientation_branches_at_cell(*i);
                    result.extend(new_nodes);
                }
            }
            result
        }
    }

    pub fn generate_orientation_branches_at_cell(&self, cell_index: usize) -> Vec<Self> {
        if let Some(token) = self.cells[cell_index].as_ref() {
            let mut result = vec![];
            for orientation_index in self.orientation_iter(token.type_(), cell_index) {
                let mut new_node = self.clone();
                new_node.cells[cell_index]
                    .as_mut()
                    .expect("We just validated there is a token in this cell")
                    .orientation = Some(Orientation::from_index(orientation_index));
                result.push(new_node);
            }
            result
        } else {
            vec![]
        }
    }

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

    pub fn reset_tokens(&mut self) {
        self.cells
            .as_mut()
            .into_iter()
            .flatten()
            .for_each(|token| token.reset())
    }

    fn clone_to_checker(&self) -> Checker {
        Checker::from_solver_node(self.clone())
    }

    fn laser_position(&self) -> Option<usize> {
        self.cells.as_ref().iter().position(|token| {
            if let Some(token) = token {
                token.type_() == &TokenType::Laser
            } else {
                false
            }
        })
    }

    fn laser_placed(&self) -> bool {
        self.cells
            .as_ref()
            .iter()
            .flatten()
            .any(|token| token.type_() == &TokenType::Laser)
    }

    fn laser_placed_and_rotated(&self) -> bool {
        self.cells
            .as_ref()
            .iter()
            .flatten()
            .any(|token| token.type_() == &TokenType::Laser && token.orientation().is_some())
    }

    pub fn all_placed_tokens_have_orientation_set(&self) -> bool {
        self.cells
            .as_ref()
            .iter()
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
                let mut new_ordering = current_ordering;
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

        let mut result = vec![];

        for unique_ordering in unique_orderings {
            let mut new_node = self.clone();
            new_node.tokens_to_be_added = vec![];
            new_node.tokens_to_be_added_shuffled = unique_ordering;
            result.push(new_node);
        }

        result
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

    fn target_mirror_orientation_iter(
        &self,
        forbidden_directions: Vec<usize>,
        cell_index: usize,
    ) -> Vec<usize> {
        let mut result = vec![0, 1, 2, 3];
        // if this token must be lit, it cannot be inaccessible
        if let Some(target_mirror_token) = &self.cells[cell_index] {
            if target_mirror_token.type_() != &TokenType::TargetMirror {
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

        result
    }

    // returns an array representing the out-of-board orientations
    fn forbidden_orientations(&self, cell_index: usize) -> [Option<Orientation>; 2] {
        // the center cannot be considered an edge piece, regardless of the cell blocker's location
        if cell_index == 12 {
            return [None, None];
        }

        // we need to check the cell blocker first because edge pieces can have a different result from this
        // function if the cell blocker is on a corner
        if let Some((cell_blocker_index, _)) =
            self.cells.as_ref().iter().enumerate().find(|(_, token)| {
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

    pub fn check(self) -> Checker {
        let mut checker = self.clone_to_checker();
        checker.check()
    }
}

lazy_static! {
    pub static ref SPIRAL_ORDER: [usize; 25] = [
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
