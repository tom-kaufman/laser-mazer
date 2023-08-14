use std::collections::HashMap;

pub mod orientation;

pub mod token;
use token::{Token, TokenType};

mod solver_node;
use solver_node::SolverNode;

mod checker;

/// LaserMazeSolver: main struct. initialize this with the puzzle -> run .solve()
/// initial_grid_config: initially, where the tokens are placed on the grid and their rotation
/// tokens_to_be_added: the "add to grid" section of the card
/// dfs_stack: an Arc<Mutex<SolverNode>>> that holds the thread-safe stack used by DFS algorithm
pub struct LaserMazeSolver {
    initial_grid_config: [Option<Token>; 25],
    tokens_to_be_added: Vec<Token>,
    pub stack: Vec<SolverNode>,
    targets: u8,
}

impl LaserMazeSolver {
    #[allow(dead_code)]
    pub fn new(
        initial_grid_config: [Option<Token>; 25],
        tokens_to_be_added: Vec<Token>,
        targets: u8,
    ) -> Self {
        let initial_solver_node = SolverNode::new(
            initial_grid_config.clone(),
            tokens_to_be_added.clone(),
            targets,
        );
        Self {
            initial_grid_config,
            tokens_to_be_added,
            targets,
            stack: vec![initial_solver_node],
        }
    }

    /// validate that a good Challenge is provided
    fn validate(&self) -> bool {
        // 1 - 3 targets
        if (self.targets == 0) || (self.targets > 3) {
            println!("Invalid number of targets!");
            return false;
        }

        // make sure count of each type of Token is valid
        // count piece types on the grid
        let mut token_counts: HashMap<TokenType, u8> = HashMap::new();
        for token in self.initial_grid_config.iter().flatten() {
            token_counts
                .entry(*token.type_())
                .and_modify(|counter| *counter += 1)
                .or_insert(1);
        }
        // count pieces to be added
        for token in &self.tokens_to_be_added {
            token_counts
                .entry(*token.type_())
                .and_modify(|counter| *counter += 1)
                .or_insert(1);
        }
        // check the counts
        for (token_type, count) in token_counts {
            let (min_count, max_count) = match token_type {
                TokenType::Laser => (1, 1),
                TokenType::TargetMirror => (1, 5),
                TokenType::BeamSplitter => (0, 2),  // previously I thought `n_targets = 1 + n_beam_splitters`, but bonus challenge 98, 99 contracdict this (self.targets - 1, self.targets - 1),
                TokenType::DoubleMirror => (0, 1),
                TokenType::Checkpoint => (0, 1),
                TokenType::CellBlocker => (0, 1),
            };
            if (count < min_count) || (count > max_count) {
                println!("Invalid piece count for piece type {:?}!", token_type);
                return false;
            }
        }

        // check that there are enough TargetMirror tokens, given the number of targets for this challenge
        // and the number of pieces that must be lit
        let must_light_count: u8 = self
            .initial_grid_config
            .iter()
            .filter_map(|cell: &Option<Token>| cell.as_ref())
            .map(|token| token.must_light() as u8)
            .sum();
        if self.targets < must_light_count {
            println!("Invalid number of pieces which must be lit!");
            return false;
        }

        // no cell blocker in tokens to be added
        if self
            .tokens_to_be_added
            .iter()
            .any(|token| token.type_() == &TokenType::CellBlocker)
        {
            println!("Cell Blocker included in tokens_to_be_added!");
            return false;
        }

        true
    }

    // initialize some things (for now, just sort the pieces to be added heuristically)
    fn initialize(&mut self) {
        // sort the tokens to be added
        // we will get tokens out of the "to be added vec" with pop so we will sort with the
        // tokens we want to place first at the end of the vec
        let reference_order = [
            // CellBlocker is not allowed in to_be_added, but is here for completeness
            TokenType::CellBlocker,
            // the last 3 are ordered by gut feel (for now)
            TokenType::DoubleMirror,
            TokenType::BeamSplitter,
            // the first 3 have more heuristics
            TokenType::Checkpoint,
            TokenType::TargetMirror,
            TokenType::Laser,
        ];
        let mut new_tokens = vec![];

        for token_type in reference_order {
            for token in &self.tokens_to_be_added {
                if token.type_() == &token_type {
                    new_tokens.push(token.clone());
                }
            }
        }

        self.tokens_to_be_added = new_tokens;
    }

    #[allow(dead_code)]
    pub fn solve(&mut self) -> Option<[Option<Token>; 25]> {
        if !self.validate() {
            panic!("invalid challenge");
        }

        self.initialize();

        while !self.stack.is_empty() {
            let mut node = self.stack.pop().expect("loop criteria is non-empty vec");
            match node.generate_branches() {
                Ok(cells) => return Some(cells),
                Err(new_nodes) => self.stack.extend(new_nodes),
            }
        }

        None
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::solver::orientation::Orientation;
    use std::time;

    // /| -- /  -- X
    //       ||
    //       []
    // /| -- /
    //       \\ -- |/
    #[test]
    fn test_checker_all_tokens() {
        let mut cells: [Option<Token>; 25] = Default::default();

        // laser in top right
        cells[24] = Some(Token::new(TokenType::Laser, Some(Orientation::West), false));

        // splitting mirror piece on center col, top row cell
        cells[22] = Some(Token::new(
            TokenType::BeamSplitter,
            Some(Orientation::East),
            false,
        ));

        // target 1: top left cell, target facing east
        cells[20] = Some(Token::new(
            TokenType::TargetMirror,
            Some(Orientation::East),
            false,
        ));

        // gate piece, middle col  row[3]
        cells[17] = Some(Token::new(
            TokenType::Checkpoint,
            Some(Orientation::South),
            false,
        ));

        // block piece, true center
        cells[12] = Some(Token::new(
            TokenType::CellBlocker,
            Some(Orientation::West),
            false,
        ));

        // splitting mirror piece on center col, row[1] cell
        cells[7] = Some(Token::new(
            TokenType::BeamSplitter,
            Some(Orientation::East),
            false,
        ));

        // double mirror piece on bottom middle cell, facing south
        cells[2] = Some(Token::new(
            TokenType::DoubleMirror,
            Some(Orientation::South),
            false,
        ));

        // target 2: left col, row[1] cell, facing east
        cells[5] = Some(Token::new(
            TokenType::TargetMirror,
            Some(Orientation::East),
            false,
        ));

        // target 3: bottom right cell, facing west
        cells[4] = Some(Token::new(
            TokenType::TargetMirror,
            Some(Orientation::West),
            false,
        ));

        let mut solver = LaserMazeSolver::new(cells, vec![], 3);
        let result = solver
            .stack
            .pop()
            .expect("LaserMazeSolver initializes with a node")
            .check()
            .solved();
        assert!(result)
    }

    #[test]
    fn test_solver_simple() {
        let mut cells: [Option<Token>; 25] = Default::default();

        cells[0] = Some(Token::new(
            TokenType::Laser,
            Some(Orientation::North),
            false,
        ));
        cells[6] = Some(Token::new(
            TokenType::TargetMirror,
            Some(Orientation::West),
            true,
        ));
        cells[10] = Some(Token::new(
            TokenType::TargetMirror,
            Some(Orientation::South),
            false,
        ));

        let mut tokens_to_be_added = vec![];
        tokens_to_be_added.push(Token::new(TokenType::BeamSplitter, None, false));

        let mut solver = LaserMazeSolver::new(cells, tokens_to_be_added, 2);

        let t0 = time::Instant::now();
        let result = solver.solve();
        let t1 = time::Instant::now();

        println!("{:?}", result.unwrap());
        println!("Processed in {:?}", t1 - t0);
    }

    #[test]
    fn test_solver_puzzle_25() {
        let mut cells: [Option<Token>; 25] = Default::default();

        cells[3] = Some(Token::new(TokenType::TargetMirror, None, true));
        cells[7] = Some(Token::new(TokenType::Checkpoint, None, false));
        cells[8] = Some(Token::new(TokenType::BeamSplitter, None, false));
        cells[20] = Some(Token::new(TokenType::Laser, None, false));
        cells[23] = Some(Token::new(
            TokenType::CellBlocker,
            Some(Orientation::East),
            false,
        ));

        let mut tokens_to_be_added = vec![];
        tokens_to_be_added.push(Token::new(TokenType::TargetMirror, None, true));
        tokens_to_be_added.push(Token::new(TokenType::DoubleMirror, None, false));

        let mut solver = LaserMazeSolver::new(cells, tokens_to_be_added, 2);

        let t0 = time::Instant::now();
        let result = solver.solve();
        let t1 = time::Instant::now();

        println!("{:?}", result.unwrap());
        println!("Processed in {:?}", t1 - t0);
    }

    #[test]
    fn test_solver_puzzle_40() {
        let mut cells: [Option<Token>; 25] = Default::default();

        cells[3] = Some(Token::new(
            TokenType::TargetMirror,
            Some(Orientation::North),
            true,
        ));
        cells[9] = Some(Token::new(
            TokenType::TargetMirror,
            Some(Orientation::West),
            true,
        ));
        cells[11] = Some(Token::new(
            TokenType::DoubleMirror,
            Some(Orientation::North),
            false,
        ));
        cells[17] = Some(Token::new(
            TokenType::Checkpoint,
            Some(Orientation::North),
            false,
        ));
        cells[20] = Some(Token::new(TokenType::Laser, None, false));

        let mut tokens_to_be_added = vec![];
        tokens_to_be_added.push(Token::new(TokenType::TargetMirror, None, false));
        tokens_to_be_added.push(Token::new(TokenType::TargetMirror, None, false));
        tokens_to_be_added.push(Token::new(TokenType::TargetMirror, None, false));
        tokens_to_be_added.push(Token::new(TokenType::BeamSplitter, None, false));

        let mut solver = LaserMazeSolver::new(cells, tokens_to_be_added, 2);

        let t0 = time::Instant::now();
        let result = solver.solve();
        let t1 = time::Instant::now();

        println!("{:?}", result.unwrap());
        println!("Processed in {:?}", t1 - t0);
    }

    #[test]
    fn test_solver_puzzle_50() {
        let mut cells: [Option<Token>; 25] = Default::default();

        cells[3] = Some(Token::new(
            TokenType::CellBlocker,
            Some(Orientation::North),
            false,
        ));
        cells[4] = Some(Token::new(TokenType::TargetMirror, None, true));
        cells[6] = Some(Token::new(
            TokenType::BeamSplitter,
            Some(Orientation::North),
            false,
        ));
        cells[7] = Some(Token::new(TokenType::TargetMirror, None, true));
        cells[13] = Some(Token::new(
            TokenType::Checkpoint,
            Some(Orientation::East),
            false,
        ));
        cells[20] = Some(Token::new(TokenType::TargetMirror, None, true));

        let mut tokens_to_be_added = vec![];
        tokens_to_be_added.push(Token::new(TokenType::TargetMirror, None, false));
        tokens_to_be_added.push(Token::new(TokenType::TargetMirror, None, false));
        tokens_to_be_added.push(Token::new(TokenType::BeamSplitter, None, false));
        tokens_to_be_added.push(Token::new(TokenType::Laser, None, false));

        let mut solver = LaserMazeSolver::new(cells, tokens_to_be_added, 3);

        let t0 = time::Instant::now();
        let result = solver.solve();
        let t1 = time::Instant::now();

        println!("{:?}", result.unwrap());
        println!("Processed in {:?}", t1 - t0);
    }

    // 2nd to last puzzle with the laser's position not given
    #[test]
    fn test_solver_puzzle_54() {
        let mut cells: [Option<Token>; 25] = Default::default();

        cells[3] = Some(Token::new(TokenType::TargetMirror, None, false));
        cells[6] = Some(Token::new(
            TokenType::TargetMirror,
            Some(Orientation::North),
            true,
        ));
        cells[12] = Some(Token::new(
            TokenType::TargetMirror,
            Some(Orientation::South),
            true,
        ));
        cells[18] = Some(Token::new(TokenType::DoubleMirror, None, false));
        cells[21] = Some(Token::new(TokenType::BeamSplitter, None, false));
        cells[24] = Some(Token::new(TokenType::TargetMirror, None, false));

        let mut tokens_to_be_added = vec![];
        tokens_to_be_added.push(Token::new(TokenType::Laser, None, false));
        tokens_to_be_added.push(Token::new(TokenType::TargetMirror, None, false));
        tokens_to_be_added.push(Token::new(TokenType::BeamSplitter, None, false));

        let mut solver = LaserMazeSolver::new(cells, tokens_to_be_added, 3);

        let t0 = time::Instant::now();
        let result = solver.solve();
        let t1 = time::Instant::now();

        println!("{:?}", result.unwrap());
        println!("Processed in {:?}", t1 - t0);
    }

    #[test]
    fn test_solver_puzzle_55() {
        let mut cells: [Option<Token>; 25] = Default::default();

        cells[2] = Some(Token::new(TokenType::TargetMirror, None, false));
        cells[6] = Some(Token::new(TokenType::TargetMirror, None, false));
        cells[9] = Some(Token::new(TokenType::TargetMirror, None, false));
        cells[12] = Some(Token::new(TokenType::TargetMirror, None, false));
        cells[18] = Some(Token::new(TokenType::TargetMirror, None, false));
        cells[3] = Some(Token::new(TokenType::DoubleMirror, None, false));
        cells[16] = Some(Token::new(TokenType::Laser, None, false));

        let mut tokens_to_be_added = vec![];
        tokens_to_be_added.push(Token::new(TokenType::Checkpoint, None, false));
        tokens_to_be_added.push(Token::new(TokenType::BeamSplitter, None, false));

        let mut solver = LaserMazeSolver::new(cells, tokens_to_be_added, 2);

        let t0 = time::Instant::now();
        let result = solver.solve();
        let t1 = time::Instant::now();

        println!("{:?}", result.unwrap());
        println!("Processed in {:?}", t1 - t0);
    }

    #[test]
    fn test_solver_puzzle_59() {
        let mut cells: [Option<Token>; 25] = Default::default();

        cells[6] = Some(Token::new(
            TokenType::Laser,
            Some(Orientation::North),
            false,
        ));
        cells[8] = Some(Token::new(TokenType::Checkpoint, None, false));
        cells[10] = Some(Token::new(TokenType::TargetMirror, None, true));
        cells[12] = Some(Token::new(TokenType::DoubleMirror, None, false));
        cells[15] = Some(Token::new(TokenType::TargetMirror, None, false));
        cells[17] = Some(Token::new(
            TokenType::CellBlocker,
            Some(Orientation::East),
            false,
        ));
        cells[18] = Some(Token::new(TokenType::BeamSplitter, None, false));

        let mut tokens_to_be_added = vec![];
        tokens_to_be_added.push(Token::new(TokenType::TargetMirror, None, false));
        tokens_to_be_added.push(Token::new(TokenType::TargetMirror, None, false));
        tokens_to_be_added.push(Token::new(TokenType::TargetMirror, None, false));
        tokens_to_be_added.push(Token::new(TokenType::BeamSplitter, None, false));

        let mut solver = LaserMazeSolver::new(cells, tokens_to_be_added, 3);

        let t0 = time::Instant::now();
        let result = solver.solve();
        let t1 = time::Instant::now();

        println!("{:?}", result.unwrap());
        println!("Processed in {:?}", t1 - t0);
    }

    #[test]
    fn test_solver_puzzle_60() {
        let mut cells: [Option<Token>; 25] = Default::default();

        cells[9] = Some(Token::new(
            TokenType::TargetMirror,
            Some(Orientation::North),
            true,
        ));
        cells[23] = Some(Token::new(
            TokenType::TargetMirror,
            Some(Orientation::West),
            true,
        ));
        cells[15] = Some(Token::new(
            TokenType::TargetMirror,
            Some(Orientation::South),
            false,
        ));
        cells[1] = Some(Token::new(TokenType::DoubleMirror, None, false));
        cells[12] = Some(Token::new(TokenType::Checkpoint, None, false));
        cells[11] = Some(Token::new(
            TokenType::CellBlocker,
            Some(Orientation::South),
            false,
        ));

        let mut tokens_to_be_added = vec![];
        tokens_to_be_added.push(Token::new(TokenType::Laser, None, false));
        tokens_to_be_added.push(Token::new(TokenType::BeamSplitter, None, false));
        tokens_to_be_added.push(Token::new(TokenType::BeamSplitter, None, false));
        tokens_to_be_added.push(Token::new(TokenType::TargetMirror, None, false));
        tokens_to_be_added.push(Token::new(TokenType::TargetMirror, None, false));

        let mut solver = LaserMazeSolver::new(cells, tokens_to_be_added, 3);

        let t0 = time::Instant::now();
        let result = solver.solve();
        let t1 = time::Instant::now();

        println!("{:?}", result.unwrap());
        println!("Processed in {:?}", t1 - t0);
    }

    #[test]
    // bonus 99 - the last bonus puzzle with the laser position not given
    fn test_solver_puzzle_153() {
        let mut cells: [Option<Token>; 25] = Default::default();

        cells[9] = Some(Token::new(
            TokenType::Checkpoint,
            Some(Orientation::North),
            false,
        ));
        cells[11] = Some(Token::new(
            TokenType::BeamSplitter,
            Some(Orientation::North),
            false,
        ));
        cells[13] = Some(Token::new(
            TokenType::DoubleMirror,
            Some(Orientation::East),
            false,
        ));
        cells[16] = Some(Token::new(
            TokenType::TargetMirror,
            Some(Orientation::West),
            true,
        ));
        cells[18] = Some(Token::new(
            TokenType::CellBlocker,
            Some(Orientation::North),
            false,
        ));

        let mut tokens_to_be_added = vec![];
        tokens_to_be_added.push(Token::new(TokenType::TargetMirror, None, false));
        tokens_to_be_added.push(Token::new(TokenType::TargetMirror, None, false));
        tokens_to_be_added.push(Token::new(TokenType::TargetMirror, None, false));
        tokens_to_be_added.push(Token::new(TokenType::TargetMirror, None, false));
        tokens_to_be_added.push(Token::new(TokenType::BeamSplitter, None, false));
        tokens_to_be_added.push(Token::new(TokenType::Laser, None, false));

        let mut solver = LaserMazeSolver::new(cells, tokens_to_be_added, 3);

        let t0 = time::Instant::now();
        let result = solver.solve();
        let t1 = time::Instant::now();

        println!("{:?}", result.unwrap());
        println!("Processed in {:?}", t1 - t0);
    }

    // bonus 99
    #[test]
    fn test_solver_puzzle_159() {
        let mut cells: [Option<Token>; 25] = Default::default();

        cells[10] = Some(Token::new(
            TokenType::Checkpoint,
            Some(Orientation::North),
            false,
        ));
        cells[16] = Some(Token::new(
            TokenType::DoubleMirror,
            Some(Orientation::North),
            false,
        ));
        cells[20] = Some(Token::new(
            TokenType::CellBlocker,
            Some(Orientation::North),
            false,
        ));
        cells[23] = Some(Token::new(TokenType::Laser, None, false));

        let mut tokens_to_be_added = vec![];
        tokens_to_be_added.push(Token::new(TokenType::TargetMirror, None, false));
        tokens_to_be_added.push(Token::new(TokenType::TargetMirror, None, false));
        tokens_to_be_added.push(Token::new(TokenType::TargetMirror, None, false));
        tokens_to_be_added.push(Token::new(TokenType::TargetMirror, None, false));
        tokens_to_be_added.push(Token::new(TokenType::BeamSplitter, None, false));
        tokens_to_be_added.push(Token::new(TokenType::BeamSplitter, None, false));

        let mut solver = LaserMazeSolver::new(cells, tokens_to_be_added, 3);

        let t0 = time::Instant::now();
        let result = solver.solve();
        let t1 = time::Instant::now();

        println!("{:?}", result.unwrap());
        println!("Processed in {:?}", t1 - t0);
    }
}
