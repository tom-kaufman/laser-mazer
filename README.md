# TODO
## Docs
- [ ] Rewrite
## GUI
- [ ] load a "challenge" (starting Grid config + the pieces you can place to solve it)
- [ ] wipe board back to challenge
- [ ] save new challenges to file
- [ ] include the challenges from the official game
- [ ] show the laser's path
- [ ] improved layoutca
- [ ] better way to visualize results other than the message label at bottom left
- [ ] show controls (WASD, R, M are used but no visual indicator)
- [ ] put the token icon on mouse cursor when dragging it to move it between cells
- [ ] swap tokens places if you drag a token onto another cell already holding a token
- [ ] cursor changes on hovering tokens
- [ ] ctrl z to undo moves
- [ ] shortcut keys to move pieces back to bank or "to be added"
## Solver backend
- [ ] Eliminate panics
- [ ] Error enum

# How many possible GameBoard configurations are there?
- 12 game pieces; any # from 2-12 pieces may be included; actual # pieces in puzzle = `N`
- 25 cells, all `N` pieces must be placed -> `25! / (25 - N)!`
- each piece may have 4 rotations, for N pieces, there are `4^N` rotation configurations
- so, `summation on [2, 12] { 4^N * 25! / (25 - N)! }` yields 4.255014734672012e+22


# 
- for choosing rotations of pieces near the edge of the board:
    - gate pieces never point outside of the board
    - check single mirror pieces for if their targets are pointing outside of the board; need at least `N` targets available pointing into the board
    - laser doesn't point outside of the board
    - laser doesn't point along any row or column which is empty besides the laser
    - if black blocking piece is on the edge, these rules apply to the piece neighboring it 1 row/column inward
- when placing the moveable pieces:
    - **pieces never go in their own row and column**
        - this will easily narrow down the search space more than any rule
		- if there are L lonely pieces (pieces are on thier own row and column, which are not gate pieces or blocking pieces)) and N pieces left to place:
			- if L = N, the next piece must go on the same row/column as a lonely piece, and the same row/column as another piece (2nd piece not necessarily lonely), 
			- placing a piece must decrease the quantity L if L > 0
			
    - splitting mirror pieces never go on corners
    - gate pieces never go on corners


# using parts of the checker for the solver
DFS order:
1. Laser set + rotated
2. All set pieces rotated
3. Set + rotate one token at a time
4. When setting in (3), use the laser path to make assertions: the piece must go in the path of the laser
5. 