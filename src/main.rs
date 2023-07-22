use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time;

mod orientation;
use orientation::Orientation;
mod token;
use token::{Token, TokenType};
mod solver_node;
use solver_node::SolverNode;

/// LaserMazeSolver: main struct. initialize this with the puzzle -> run .solve()
/// initial_grid_config: initially, where the tokens are placed on the grid and their rotation
/// tokens_to_be_added: the "add to grid" section of the card
/// dfs_stack: an Arc<Mutex<SolverNode>>> that holds the thread-safe stack used by DFS algorithm
struct LaserMazeSolver {
    initial_grid_config: [Option<Token>; 25],
    tokens_to_be_added: Vec<Token>,
    dfs_stack: Arc<Mutex<Vec<SolverNode>>>,
    targets: u8,
}

impl LaserMazeSolver {
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
            dfs_stack: Arc::new(Mutex::new(vec![initial_solver_node])),
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
                TokenType::BeamSplitter => (self.targets - 1, self.targets - 1),
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
    fn solver_thread(
        &self,
        result_found: Arc<AtomicBool>,
    ) -> thread::JoinHandle<Option<[Option<Token>; 25]>> {
        let stack = Arc::clone(&self.dfs_stack);
        thread::spawn(move || {
            loop {
                // get the lock on the Mutex, then exit the loop if stack is empty or pop a node
                let mut vec = stack.lock().unwrap();
                if vec.is_empty() || result_found.load(Ordering::Acquire) {
                    break;
                }
                let mut node = vec
                    .pop()
                    .expect("We just checked that the stack isn't empty");

                // drop the lock on the vec while we do some work
                drop(vec);

                // build a vec of items to add if we aren't at a leaf
                // we don't need to clone here, because the nod only gets mutated if it's not a leaf
                let new_nodes = node.generate_branches();

                // if new_nodes is still empty, we're at a leaf; check the puzzle and return a Some() if we find solution
                // if new_nodes is not empty, we push the new items on the stack and don't check solution
                if new_nodes.is_empty() {
                    if node.clone().check().solved() {
                        result_found.store(true, Ordering::Release);
                        return Some(node.clone_cells());
                    }
                } else {
                    // get the lock back and push the new items
                    let mut vec = stack.lock().unwrap();
                    vec.extend(new_nodes);
                }
            }
            None
        })
    }

    /// returns None if no solution, or a grid of tokens if a solution is found
    #[allow(dead_code)]
    pub fn solve_multi_thread(&mut self, n_threads: usize) -> Option<[Option<Token>; 25]> {
        if !self.validate() {
            panic!("invalid challenge");
        }

        self.initialize();

        let result_found = Arc::new(AtomicBool::new(false));
        let mut threads = vec![];
        for _ in 0..n_threads {
            threads.push(self.solver_thread(result_found.clone()));
        }

        for thread in threads {
            if let Some(solution) = thread.join().unwrap() {
                return Some(solution);
            }
        }

        None
    }

    #[allow(dead_code)]
    pub fn solve_single_thread(&mut self) -> Option<[Option<Token>; 25]> {
        if !self.validate() {
            panic!("invalid challenge");
        }

        self.initialize();

        // get the stack out of the Arc<Mutex<>>
        let mut stack = self.dfs_stack.lock().unwrap().clone();

        while !stack.is_empty() {
            let mut node = stack.pop().expect("loop criteria is non-empty vec");
            let new_nodes = node.generate_branches();
            if new_nodes.is_empty() {
                if node.clone().check().solved() {
                    return Some(node.clone_cells());
                }
            } else {
                stack.extend(new_nodes);
            }
        }

        None
    }

    #[allow(dead_code)]
    pub fn solve_single_thread_laser_aware(&mut self) -> Option<[Option<Token>; 25]> {
        if !self.validate() {
            panic!("invalid challenge");
        }

        self.initialize();

        // get the stack out of the Arc<Mutex<>>
        let mut stack = self.dfs_stack.lock().unwrap().clone();

        while !stack.is_empty() {
            let mut node = stack.pop().expect("loop criteria is non-empty vec");
            let new_nodes = node.generate_branches_laser_aware();
            println!(
                "Stack has len {}, extending by {} nodes",
                stack.len(),
                new_nodes.len()
            );
            if new_nodes.is_empty() {
                println!("At a leaf!!");
                // println!("Leaf we're about to check: {:?}", &node);

                // TODO delete me
                // if node.cells[14].as_ref().is_some() && node.cells[14].as_ref().unwrap().type_() == &TokenType::Laser && node.cells[14].as_ref().unwrap().orientation == Some(Orientation::West) {
                //     // println!("Laser oriented correctly for puzzle 50");
                //     if node.cells[11].as_ref().is_some() && node.cells[11].as_ref().unwrap().type_() == &TokenType::BeamSplitter && node.cells[11].as_ref().unwrap().orientation == Some(Orientation::East) {
                //         // println!("Beam splitter on slot 11 oriented correctly for puzzle 50");
                //         if node.cells[10].as_ref().is_some() && node.cells[10].as_ref().unwrap().type_() == &TokenType::TargetMirror && node.cells[10].as_ref().unwrap().orientation == Some(Orientation::South) {
                //             // println!("Target on slot 10 oriented correctly for puzzle 50");
                //             if node.cells[20].as_ref().is_some() && node.cells[20].as_ref().unwrap().type_() == &TokenType::TargetMirror && node.cells[20].as_ref().unwrap().orientation == Some(Orientation::South) {
                //                 // println!("Target on slot 20 oriented correctly for puzzle 50");
                //                 if node.cells[6].as_ref().is_some() && node.cells[6].as_ref().unwrap().type_() == &TokenType::BeamSplitter && node.cells[6].as_ref().unwrap().orientation == Some(Orientation::North) {
                //                     // println!("Splitter on slot 6 oriented correctly for puzzle 50");
                //                     if node.cells[7].as_ref().is_some() && node.cells[7].as_ref().unwrap().type_() == &TokenType::TargetMirror && node.cells[7].as_ref().unwrap().orientation == Some(Orientation::West) {
                //                         println!("Target on slot 7 oriented correctly for puzzle 50");
                //                         if node.cells[1].as_ref().is_some() && node.cells[1].as_ref().unwrap().type_() == &TokenType::TargetMirror && node.cells[1].as_ref().unwrap().orientation == Some(Orientation::South) {
                //                             println!("Target on slot 1 oriented correctly for puzzle 50");
                //                             if node.cells[4].as_ref().is_some() && node.cells[4].as_ref().unwrap().type_() == &TokenType::TargetMirror && node.cells[4].as_ref().unwrap().orientation == Some(Orientation::West) {
                //                                 println!("Target on slot 4 oriented correctly for puzzle 50");
                //                             }
                //                         }  else if node.cells[1].as_ref().is_some() && node.cells[1].as_ref().unwrap().type_() == &TokenType::TargetMirror {
                //                             println!("Target is on slot 1 but wrong orientation; orientation = {:?}", node.cells[1].as_ref().unwrap().orientation);
                //                         }
                //                     }
                //                 }
                //             }
                //         }
                //     }
                // }

                if node.clone().check().solved() {
                    // println!("Solution!");
                    return Some(node.clone_cells());
                }
                // println!("Not a solution!");
            } else {
                stack.extend(new_nodes);
            }
        }

        None
    }
}

fn main() {
    for _ in 0..100 {
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
        let result = solver.solve_multi_thread(16);
        let t1 = time::Instant::now();

        // println!("{:?}", result.unwrap());
        println!("Processed in {:?}", t1 - t0);
    }
}

#[cfg(test)]
mod test {
    use super::*;
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

        let solver = LaserMazeSolver::new(cells, vec![], 3);
        let mut solver_node = solver.dfs_stack.lock().unwrap();
        let result = solver_node
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
        let result = solver.solve_multi_thread(16);
        let t1 = time::Instant::now();

        println!("{:?}", result.unwrap());
        println!("Processed in {:?}", t1 - t0);
    }

    #[test]
    fn test_solver_simple_laser_aware() {
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
        let result = solver.solve_single_thread_laser_aware();
        let t1 = time::Instant::now();

        println!("{:?}", result.unwrap());
        println!("Processed in {:?}", t1 - t0);
    }

    #[test]
    fn test_solver_puzzle_25_par() {
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
        let result = solver.solve_multi_thread(1);
        let t1 = time::Instant::now();

        println!("{:?}", result.unwrap());
        println!("Processed in {:?}", t1 - t0);
    }

    #[test]
    fn test_solver_puzzle_25_st() {
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
        let result = solver.solve_single_thread();
        let t1 = time::Instant::now();

        println!("{:?}", result.unwrap());
        println!("Processed in {:?}", t1 - t0);
    }

    #[test]
    fn test_solver_puzzle_40_st() {
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
        let result = solver.solve_single_thread();
        let t1 = time::Instant::now();

        println!("{:?}", result.unwrap());
        println!("Processed in {:?}", t1 - t0);
    }

    #[test]
    fn test_solver_puzzle_40_par() {
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
        let result = solver.solve_multi_thread(16);
        let t1 = time::Instant::now();

        println!("{:?}", result.unwrap());
        println!("Processed in {:?}", t1 - t0);
    }

    #[test]
    fn test_solver_puzzle_40_st_laser_aware() {
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
        let result = solver.solve_single_thread_laser_aware();
        let t1 = time::Instant::now();

        println!("{:?}", result.unwrap());
        println!("Processed in {:?}", t1 - t0);
    }

    #[test]
    fn test_solver_puzzle_50_st_laser_aware() {
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
        let result = solver.solve_single_thread_laser_aware();
        let t1 = time::Instant::now();

        println!("{:?}", result.unwrap());
        println!("Processed in {:?}", t1 - t0);
    }

    #[test]
    fn test_solver_puzzle_55_st_laser_aware() {
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
        let result = solver.solve_single_thread_laser_aware();
        let t1 = time::Instant::now();

        println!("{:?}", result.unwrap());
        println!("Processed in {:?}", t1 - t0);
    }
}

/*
bugged board
cells: [
        None,
        None,
        None,
        Some(Token {
            type_: TargetMirror,
            orientation: Some(North),
            lit: false,
            target_lit: Some(false),
            must_light: true
        }),
        None,
        Some(Token {
            type_: TargetMirror,
            orientation: Some(South),
            lit: true,
            target_lit: Some(false),
            must_light: false
        }),
        Some(Token {
            type_: TargetMirror,
            orientation: Some(East),
            lit: true,
            target_lit: Some(false),
            must_light: false
        }),
        None,
        None,
        Some(Token {
        type_: TargetMirror,
            orientation: Some(West),
            lit: false,
            target_lit: Some(false),
            must_light: true
        }),
        Some(Token {
            type_: BeamSplitter,
            orientation: Some(North),
            lit: true,
            target_lit: None,
            must_light: false
        }),
        Some(Token {
            type_: DoubleMirror,
            orientation: Some(North),
            lit: true,
            target_lit: None,
            must_light: false
        }),
        Some(Token {
            type_: TargetMirror,
            orientation: Some(West),
            lit: false,
            target_lit: Some(false),
            must_light: false
        }),
        None,
        None,
        None,
        None,
        Some(Token {
            type_: Checkpoint,
            orientation: Some(North),
            lit: false,
            target_lit: None,
            must_light: false
        }),
        None,
        None,
        Some(Token {
            type_: Laser,
            orientation: Some(South),
            lit: false,
            target_lit: None,
            must_light: false
        }),
        None,
        None,
        None,
        None

*/
