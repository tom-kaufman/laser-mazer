# laser-mazer

A Solver + GUI for the pguzzle board game [Laser Maze](https://www.thinkfun.com/wp-content/uploads/2013/09/Laser-1014-Instructions.pdf), implemented in Rust.

## Running
- Use [`rustup`](https://rustup.rs/) to install Rust on your machine
- `git clone` this repo
- `cd` into the cloned repo
- `cargo run --release`

## Demo
[simplescreenrecorder.webm](https://github.com/tom-kaufman/laser-mazer/assets/102370231/ff689c76-3815-4c21-8669-2a459107b09c)

## About the solver
The solver uses an iterative depth-first search (DFS) algorithm to solve the puzzle.

You'll probably want to read the [game instructions](https://www.thinkfun.com/wp-content/uploads/2013/09/Laser-1014-Instructions.pdf) before proceeding.

To initialize a `LaserMazeSolver`, we need to know:
- The state of the game board (25 `Cell`s, which may be occupied by `Token`s; each `Token`'s orientation may or may not be set)
- The `Token`s to be added; these are the tokens the player must place to solve the puzzle
- The number of targets which must be lit

Once the `LaserMazeSolver` is created, we can solve it by calling `LaserMazeSolver::solve`. `solve` first calls `LaserMazeSolver::validate` to check that a valid puzzle was provided (valid number of tokens, targets etc.).

The solver begins in earnest when we call `LaserMazeSolver::initialize`, which sorts the `tokens_to_be_added` to minimize the board permutations we have to check. Here is the order:
1. `Laser` (red): As described later in this section, we use the path of the laser to rule out where tokens might be placed, so we should always place the laser first.
2. `TargetMirror` (purple): We can check if a `TargetMirror`'s target side is inaccessible, if it is at the edge of the board or blocked by another token, ruling out many branches.
3. `Checkpoint` (yellow): We can check if the open direction of the checkpoint is obscured by other tokens or points outside the game board.
4. `BeamSplitter` (green): These last two tokens are ordered based on my gut feeling, having played the game myself. There is no strong difference between the two as far as ruling out DFS branches.
5. `DoubleMirror` (blue): same as `BeamSplitter`.

Our DFS algorithm will crawl the tree of possible ways to place and orient tokens, until all tokens have been placed and all orientations are set, then check the puzzle by tracing the laser's path and checking if all tokens are interacting with the laser and the appropriate `TargetMirror`s are lit. As described later in this document, the upper bound of valid board configurations is ~10^22. Therefore, we cannot construct and hold the entire tree in memory, so we will iteratively construct the tree, holding interim states in `LaserMazeSolver::stack`. The stack is a `std::collections::Vec<SolverNode>`.

To crawl the tree, we follow this procedure:
1. Pop a `SolverNode` off the stack. If the stack is empty, we have checked all game board states for the given puzzle, and there is no solution.
2. Call `SolverNode::generate_branches`, which returns a `Result<[Option<Token>; 25], Vec<Self>>`. The `Ok` variant means we found the solution; if we find it, we stop the search and return the result early. the `Err` variant means we did not find the solution, and holds a vec of children `SolverNode`s to check. This vec may be empty, such as if the previous node was a valid (but wrong) solution attempt, or if our heuristics tell us no children of this incomplete solution are worth checking.

Let's dive in to how `SolverNode::generate_branches` works. 

The first few calls to `SolverNode::generate_branches` have two special behaviors, made necessary because we use the laser's path to rule out child branches.
1. If the `Laser` token is yet to be placed, we first generate child branches for each placement and orientation of the laser. This means that puzzles where the position of the laser is not set already take much longer to solve!
2. Next, we need to create child branches for each unique order of token placement. The reason will be more clear after we finally talk about how we use the laser's path to heuristically rule out branches, but for now, trust me that it's necessary.

Let's consider the limiting case where only 1 of the 25 cells initially have a token, and we need to place 6 tokens (including 1 laser, 1 checkpoint, 1 double mirror, 1 beam splitter, and 2 target mirrors). The laser may be placed in any of the 24 open cells, with 4 possible orientations in each cell. Then, for each of the laser placements, we have `(1 + 1 + 1 + 2)!/(1! * 1! * 1! * 2!) = 60` unique shufflings of the 5 remaining tokens. So, we could have up to `4*60*24 = 5760` nodes before even placing any non-laser tokens.

Finally, we need to generate child nodes by either orienting the unoriented tokens, or by placing unplaced tokens. We run `SolverNode::check`, to iteratively march the laser forward, initiating it from the laser token, and stopping either when we hit an unoriented token, or once all lasers are inactive. Lasers may be inactive if they go off the board, hit a piece which doesn't transmit the laser, or overlaps a path already tread by a laser (loops are possible because of the beam splitter pieces). 

If the laser hits an unoriented token, we stop, and generate child branches for each valid, unique orientation of that token. 

If all lasers terminate without hitting an unoriented token, and we still have tokens that need to be placed, we use the path of the laser to determine which cells we might place the next token in. We do this because if we were to place a token in a cell the laser does *not* traverese, that token would have no impact on the path of the laser! This is why we must shuffle the order of the tokens to be placed; each type of token will have a totally different impact on the path of the laser. 

Here are a few more random tricks and heuristics not covered in the text above.
- If the `CellBlocker` is on an edge or corner, we treat it as an edge from the appropriate sides, when checking valid placements of other tokens.
- We place tokens in a spiral order, so that we can check edges greedily.
- `SplittingMirrors` never go on corners, as this would cause one of the emitted lasers to go off the board
- `Checkpoint`'s valid direction never points off the edge of the board

## About the GUI
This application provides a GUI, implemented with Rust + the `egui` crate, to set up board configurations and run the solver.

### Controls
- Mouse drag and drop: Move token
- W/A/S/D: Reorient hovered token
- R: Set hovered token's orientation to unknown
- M: Toggle whether hovered token must be lit (purple tokens only)

## About the game itself
The [game instructions](https://www.thinkfun.com/wp-content/uploads/2013/09/Laser-1014-Instructions.pdf) are available on ThinkFun's website. The primary purpose of this app is the solver; the fact that this app can be used to play the game is secondary. Please buy a copy to support the original creators. 

### How many possible GameBoard configurations are there?
- 12 game pieces; any # from 2-12 pieces may be included; actual # pieces in puzzle = `N`
- 25 cells, all `N` pieces must be placed -> `25! / (25 - N)!`
- each piece may have 4 rotations, for N pieces, there are `4^N` rotation configurations
- so, `summation on [2, 12] { 4^N * 25! / (25 - N)! }` yields 4.255014734672012e+22
