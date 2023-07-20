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
        true
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
                    if node.clone().check() {
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

        // get the stack out of the Arc<Mutex<>>
        let mut stack = self.dfs_stack.lock().unwrap().clone();

        while !stack.is_empty() {
            let mut node = stack.pop().expect("loop criteria is non-empty vec");
            let new_nodes = node.generate_branches();
            if new_nodes.is_empty() {
                if node.clone().check() {
                    return Some(node.clone_cells());
                }
            } else {
                stack.extend(new_nodes);
            }
        }

        None
    }
}

fn main() {
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

    let tokens_to_be_added = vec![
        Token::new(TokenType::TargetMirror, None, true),
        Token::new(TokenType::DoubleMirror, None, false),
    ];

    let mut solver = LaserMazeSolver::new(cells, tokens_to_be_added, 2);

    let t0 = time::Instant::now();
    let result = solver.solve_single_thread();
    let t1 = time::Instant::now();

    println!("{:?}", result.unwrap());
    println!("Processed in {:?}", t1 - t0);
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
            .check();
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
}
