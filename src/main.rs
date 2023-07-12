use std::collections::HashMap;

mod pieces;
use pieces::{GamePiece, Orientation, PieceType, ORIENTATION_ORDER};

#[derive(Debug, Clone)]
struct Slot {
    occupying_game_piece: Option<GamePiece>,
    active_laser_directions: HashMap<Orientation, bool>,
    position_index: u8,
    position: (u8, u8),
}

impl Slot {
    fn new(position_index: u8) -> Self {
        let mut active_laser_directions: HashMap<Orientation, bool> = HashMap::new();
        for orientation in ORIENTATION_ORDER.iter() {
            active_laser_directions.insert(orientation.clone(), false);
        }
        Self {
            occupying_game_piece: None,
            active_laser_directions,
            position_index,
            position: Slot::position_from_index(position_index),
        }
    }

    fn position_from_index(position_index: u8) -> (u8, u8) {
        (position_index % 5, position_index / 5)
    }

    /// returns None if it's out of the board
    fn index_from_position(position_coordinates: (u8, u8)) -> Option<u8> {
        if (position_coordinates.0 > 4) | (position_coordinates.1 > 4) {
            // println!(
            //     "Invalid coordinates: {}, {}",
            //     position_coordinates.0, position_coordinates.1
            // );
            None
        } else {
            Some(position_coordinates.0 + position_coordinates.1 * 5)
        }
    }

    /// from the slot, in a certain direction, what's the index of that slot?
    /// returns None if it's out of the board
    fn neighboring_slot_from_orientation(&self, neighboring_direction: Orientation) -> Option<u8> {
        let new_x: u8;
        let new_y: u8;

        match neighboring_direction {
            Orientation::North => {
                new_x = self.position.0;
                new_y = self.position.1 + 1;
            }
            Orientation::East => {
                new_x = self.position.0 + 1;
                new_y = self.position.1;
            }
            // For negative directions, we use overflow subtraction; the overflowed value will be out of [0,4]
            Orientation::South => {
                new_x = self.position.0;
                (new_y, _) = self.position.1.overflowing_sub(1);
            }
            Orientation::West => {
                (new_x, _) = self.position.0.overflowing_sub(1);
                new_y = self.position.1;
            }
        }

        Self::index_from_position((new_x, new_y))
    }
}

#[derive(Clone, Debug)]
struct LaserPosition {
    position: (u8, u8),
    orientation: Orientation,
}

impl LaserPosition {
    fn new(position_index: u8, orientation: Orientation) -> Self {
        let position = Slot::position_from_index(position_index);
        Self {
            position,
            orientation,
        }
    }
}

#[derive(Debug, Clone)]
struct GameBoard {
    slots: [Slot; 25],
    laser_positions: [Option<LaserPosition>; 3],
    targets: u8,
    turns: usize,
    valid_solution: Option<bool>,
}

impl GameBoard {
    fn new(targets: u8) -> Self {
        let slots: [Slot; 25] = (0u8..25u8)
            .map(|x| Slot::new(x))
            .collect::<Vec<Slot>>()
            .try_into()
            .unwrap();
        Self {
            slots,
            laser_positions: [None, None, None],
            targets,
            turns: 0,
            valid_solution: None,
        }
    }

    fn check_setup(&mut self) -> bool {
        // if the game board has already taken turns we can't trust several assumptions
        // used later in this method
        if self.turns > 0 {
            return false;
        };

        // check that all rotations are set
        if self
            .slots
            .iter()
            .filter_map(|slot| {
                if let Some(piece) = &slot.occupying_game_piece {
                    if piece.get_orientation().is_none() {
                        Some(true)
                    } else {
                        Some(false)
                    }
                } else {
                    None
                }
            })
            .any(|b| b)
        {
            return false;
        }

        // make sure one piece is a laser
        // also sets the initial laser
        if self
            .slots
            .iter_mut()
            .map(|slot| {
                if let Some(piece) = &slot.occupying_game_piece {
                    if piece.get_piece_type() == PieceType::Laser {
                        *slot
                            .active_laser_directions
                            .get_mut(&piece.get_orientation().expect(
                                "method shouldn't be called if all orientations are not set",
                            ))
                            .unwrap() = true;
                        self.laser_positions[0] = Some(LaserPosition::new(
                            slot.position_index,
                            piece.get_orientation().expect(
                                "method shouldn't be called if all orientations are not set",
                            ),
                        ));
                        return 1;
                    } else {
                        return 0;
                    }
                } else {
                    return 0;
                }
            })
            .sum::<u8>()
            != 1
        {
            return false;
        }

        true
    }

    fn has_active_lasers(&self) -> bool {
        self.laser_positions
            .iter()
            .any(|laser_position| laser_position.is_some())
    }

    fn count_lit_targets(&self) -> u8 {
        self.slots
            .iter()
            .filter(|slot| {
                if let Some(piece) = &slot.occupying_game_piece {
                    return piece.is_target_lit().unwrap_or(false);
                } else {
                    false
                }
            })
            .count() as u8
    }

    fn calculate_new_laser_positions(&mut self) -> [[Option<LaserPosition>; 2]; 3] {
        self.laser_positions
            .iter()
            .map(|laser_position| {
                if let Some(laser_position) = laser_position {
                    // println!("Marching forward laser {:?}", laser_position);
                    // iterating on the 3 potential active lasers, this laser is active
                    let position = Slot::index_from_position(laser_position.position)
                        .expect("no slot should be outside of the gameboard");
                    let slot = self
                        .slots
                        .get_mut(position as usize)
                        .expect("no slot should be outside of the gameboard");
                    if let Some(neighboring_slot_index) =
                        slot.neighboring_slot_from_orientation(laser_position.orientation)
                    {
                        // println!("Marching the laser forward, it's now at slot index {}", neighboring_slot_index);
                        // the next slot in the laser's path is on the board
                        let neighboring_slot_active_direction = self.slots.get_mut(neighboring_slot_index as usize).expect("we just validated that we are on the board").active_laser_directions.get_mut(&laser_position.orientation).expect("this hashmap is populated with all the keys from the Orientation enum");
                        if *neighboring_slot_active_direction {
                            return [None, None];
                        }
                        // the laser hasn't traveled over this slot in this direction yet
                        *neighboring_slot_active_direction = true;

                        if let Some(neighboring_piece) = self.slots.get_mut(neighboring_slot_index as usize).expect("we just validated that we are on the board").occupying_game_piece.as_mut() {
                            // the laser has hit a piece, we need to calculate the result
                            // println!("The laser has hit a piece of type {:?}", neighboring_piece.get_piece_type());
                            let returned_orientations = neighboring_piece.outbound_lasers_given_inbound_laser_direction(laser_position.orientation);
                            // println!("After hitting the piece, the laser became these orientations: {:?}", returned_orientations);
                            let mut result = [None, None];
                            for i in 0..2 {
                                if let Some(orientation) = returned_orientations[i] {
                                    result[i] = Some(LaserPosition::new(neighboring_slot_index, orientation));
                                }
                            }
                            // println!("Reconstructed those orientations into these laser positions: {:?}", result);
                            return result
                        } else {
                            // the laser hasn't hit a piece
                            // println!("The laser hasn't hit a piece, it's now over index {neighboring_slot_index} with orientation {:?}", laser_position.orientation);
                            return [Some(LaserPosition::new(neighboring_slot_index, laser_position.orientation)), None]
                        }
                    }
                }
                [None, None]
            })
            .collect::<Vec<[Option<LaserPosition>; 2]>>()
            .try_into()
            .unwrap()
    }

    fn calculate_result(mut self) -> Self {
        if !self.check_setup() {
            self.valid_solution = None;
            return self;
        }

        // println!("has active lasers? {}", self.has_active_lasers());
        while self.has_active_lasers() {
            // println!("TURN {}\n\n", self.turns);
            let new_lasers: [[Option<LaserPosition>; 2]; 3] = self.calculate_new_laser_positions();
            let n_new_lasers = new_lasers
                .iter()
                .map(|new_laser_pair| {
                    new_laser_pair
                        .iter()
                        .map(|laser| if laser.is_some() { 1 } else { 0 })
                        .sum::<usize>()
                })
                .sum::<usize>();
            if n_new_lasers > 3 {
                panic!()
            }
            let vec_new_laser_positions = new_lasers[0]
                .iter()
                .chain(new_lasers[1].iter().chain(new_lasers[2].iter()))
                .filter_map(|laser| match laser {
                    Some(l) => Some(l.clone()),
                    None => None,
                })
                .collect::<Vec<LaserPosition>>();
            assert!(
                vec_new_laser_positions.len() <= 3,
                "we just checked that we only had 3 new laser positions"
            );
            let mut new_laser_positions: [Option<LaserPosition>; 3] = [None, None, None];
            for (index, new_position) in new_laser_positions.iter_mut().enumerate() {
                if let Some(p) = vec_new_laser_positions.get(index) {
                    *new_position = Some(p.clone());
                }
            }
            self.laser_positions = new_laser_positions;
            self.turns += 1;
        }
        self.valid_solution = Some(
            (self.count_lit_targets() == self.targets)
                && (self
                    .slots
                    .iter()
                    .filter_map(|slot| {
                        if let Some(piece) = &slot.occupying_game_piece {
                            if piece.must_light() {
                                piece.is_lit()
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .all(|b| b)),
        );
        self
    }
}

#[derive(Clone, Debug)]
struct Puzzle {
    start_game_board: GameBoard,
    available_game_pieces: Vec<GamePiece>,
}

impl Puzzle {
    fn check_solution(self) -> bool {
        // assumes we've already checked the setup

        if !self.available_game_pieces.is_empty() {
            return false;
        }

        self.start_game_board
            .calculate_result()
            .valid_solution
            .unwrap_or_else(|| false)
    }

    // make sure the number of pieces is a valid puzzle
    // only run this one time!
    fn check_setup(&self) -> bool {
        let mut pieces: HashMap<PieceType, u8> = HashMap::new();
        for slot in &self.start_game_board.slots {
            if let Some(piece) = &slot.occupying_game_piece {
                pieces
                    .entry(piece.get_piece_type())
                    .and_modify(|counter| *counter += 1)
                    .or_insert(1);
            }
        }
        for piece in &self.available_game_pieces {
            pieces
                .entry(piece.get_piece_type())
                .and_modify(|counter| *counter += 1)
                .or_insert(1);
        }

        if !pieces.contains_key(&PieceType::Laser) {
            println!("No laser in the puzzle!");
            return false;
        }
        if !pieces.contains_key(&PieceType::SingleMirror) {
            println!("No single mirror in the puzzle!");
            return false;
        }

        let must_light_count = self
            .start_game_board
            .slots
            .iter()
            .map(|slot| {
                if let Some(piece) = &slot.occupying_game_piece {
                    if piece.must_light() {
                        1
                    } else {
                        0
                    }
                } else {
                    0
                }
            })
            .sum::<u8>(); // Slots is an array of length 25, cannot overflow

        if self.start_game_board.targets < must_light_count {
            // if there are more pieces that must be lit than there are targets to the puzzle, invalid config
            println!("Must light count exceeds puzzle's number of targets! targets: {}, must light count: {}", self.start_game_board.targets, must_light_count);
            return false;
        }

        for (piece_type, count) in pieces {
            let (min_count, max_count) = match piece_type {
                PieceType::Block => (0, 1),
                PieceType::Gate => (0, 1),
                PieceType::DoubleMirror => (0, 1),
                PieceType::Laser => (1, 1),
                PieceType::SingleMirror => (1, 5),
                PieceType::SplittingMirror => (
                    &self.start_game_board.targets - 1,
                    &self.start_game_board.targets - 1,
                ),
            };
            if (count < min_count) || (count > max_count) {
                println!("Invalid piece count for piece type {:?}!", piece_type);
                return false;
            }
        }

        true
    }

    fn dfs(self) -> Option<Self> {
        if !self.check_setup() {
            panic!("Invalid puzzle!");
        }
        let mut stack: Vec<Puzzle> = vec![self];
        let mut leafs_encountered = 0;
        while !stack.is_empty() {
            println!(
                "Stack len: {}, encountered {leafs_encountered} leafs",
                stack.len()
            );
            let mut node = stack
                .pop()
                .expect("Loop condition is that stack is not empty");
            // println!(
            //     "Got a node off the stack with {} available pieces to place",
            //     node.available_game_pieces.len()
            // );

            // check if there are pieces to place
            if let Some(piece) = node.available_game_pieces.pop() {
                for i in 0..25 {
                    if node.start_game_board.slots[i]
                        .occupying_game_piece
                        .is_none()
                    {
                        // println!(
                        //     "Creating node: Adding piece of type {:?} to board at slot {i}",
                        //     piece.get_piece_type()
                        // );
                        let mut new_node = node.clone();
                        new_node.start_game_board.slots[i].occupying_game_piece =
                            Some(piece.clone());
                        stack.push(new_node);
                    }
                }
                continue;
            }

            // check if there are pieces to rotate
            let mut position: Option<usize> = None;
            for i in 0..25 {
                if let Some(piece) = &node.start_game_board.slots[i].occupying_game_piece {
                    if piece.get_orientation().is_none() {
                        // println!("Found a rotationally free piece at slot {i}");
                        position = Some(i);
                        break;
                    } else {
                        // println!("Found a piece at slot {i} but it is not rotationally free");
                    }
                }
            }
            if let Some(position) = position {
                for x in 0..4 {
                    // this change isn't sticking
                    let mut new_node = node.clone();
                    // println!("Creating node: Setting rotation of piece at slot {position} to orientation index {x}");
                    // println!(
                    //     "Node slot {position} before setting rotation:{:?}",
                    //     new_node.start_game_board.slots[position]
                    // );
                    if let Some(piece) =
                        &mut new_node.start_game_board.slots[position].occupying_game_piece
                    {
                        (*piece).orientation = Some(ORIENTATION_ORDER[x].clone());
                    }
                    // println!(
                    //     "Node slot {position} after setting rotation:{:?}",
                    //     new_node.start_game_board.slots[position]
                    // );
                    stack.push(new_node);
                }
                continue;
            }

            // check the solution
            // println!("Checking leaf: \n{:?}\n\n", node);
            leafs_encountered += 1;
            if node.clone().check_solution() {
                return Some(node);
            }
        }
        // return none if we get through the entire stack
        None
    }
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod test {
    use super::*;
    use std::time;

    #[test]
    fn test_inbound_reorientation() {
        let piece_orientation = Orientation::North;
        let laser_orientation = Orientation::West;
        let reoriented_laser_direction =
            piece_orientation.reorientate_inbound_laser(laser_orientation);
        assert_eq!(reoriented_laser_direction, Orientation::West);

        let piece_orientation = Orientation::West;
        let laser_orientation = Orientation::West;
        let reoriented_laser_direction =
            piece_orientation.reorientate_inbound_laser(laser_orientation);
        assert_eq!(reoriented_laser_direction, Orientation::North);

        let piece_orientation = Orientation::East;
        let laser_orientation = Orientation::West;
        let reoriented_laser_direction =
            piece_orientation.reorientate_inbound_laser(laser_orientation);
        assert_eq!(reoriented_laser_direction, Orientation::South);
    }

    // /| -- /  -- X
    //       ||
    //       []
    // /| -- /
    //       \\ -- |/
    #[test]
    fn test_solver_with_all_pieces() {
        let mut game_board = GameBoard::new(3);

        // laser in top right
        game_board.slots.get_mut(24).unwrap().occupying_game_piece = Some(GamePiece::new(
            PieceType::Laser,
            Some(Orientation::West),
            true,
            false,
        ));

        // splitting mirror piece on center col, top row slot
        game_board.slots.get_mut(22).unwrap().occupying_game_piece = Some(GamePiece::new(
            PieceType::SplittingMirror,
            Some(Orientation::East),
            true,
            false,
        ));

        // target 1: top left slot, target facing east
        game_board.slots.get_mut(20).unwrap().occupying_game_piece = Some(GamePiece::new(
            PieceType::SingleMirror,
            Some(Orientation::East),
            true,
            false,
        ));

        // gate piece, middle col  row[3]
        game_board.slots.get_mut(17).unwrap().occupying_game_piece = Some(GamePiece::new(
            PieceType::Gate,
            Some(Orientation::South),
            true,
            false,
        ));

        // block piece, true center
        game_board.slots.get_mut(12).unwrap().occupying_game_piece = Some(GamePiece::new(
            PieceType::Block,
            Some(Orientation::West),
            true,
            false,
        ));

        // splitting mirror piece on center col, row[1] slot
        game_board.slots.get_mut(7).unwrap().occupying_game_piece = Some(GamePiece::new(
            PieceType::SplittingMirror,
            Some(Orientation::East),
            true,
            false,
        ));

        // double mirror piece on bottom middle slot, facing south
        game_board.slots.get_mut(2).unwrap().occupying_game_piece = Some(GamePiece::new(
            PieceType::DoubleMirror,
            Some(Orientation::South),
            true,
            false,
        ));

        // target 2: left col, row[1] slot, facing east
        game_board.slots.get_mut(5).unwrap().occupying_game_piece = Some(GamePiece::new(
            PieceType::SingleMirror,
            Some(Orientation::East),
            true,
            false,
        ));

        // target 3: bottom right slot, facing west
        game_board.slots.get_mut(4).unwrap().occupying_game_piece = Some(GamePiece::new(
            PieceType::SingleMirror,
            Some(Orientation::West),
            true,
            false,
        ));

        game_board = game_board.calculate_result();
        println!("lit targets: {}", game_board.count_lit_targets());
        assert!(game_board.valid_solution.unwrap())
    }

    #[test]
    fn test_puzzle_validation() {
        let mut start_game_board = GameBoard::new(2);
        start_game_board.slots[0].occupying_game_piece =
            Some(GamePiece::new(PieceType::Laser, None, true, false));
        start_game_board.slots[1].occupying_game_piece =
            Some(GamePiece::new(PieceType::SingleMirror, None, true, true));
        let mut available_game_pieces = vec![];
        available_game_pieces.push(GamePiece::new(PieceType::SingleMirror, None, false, false));
        available_game_pieces.push(GamePiece::new(PieceType::DoubleMirror, None, false, false));
        available_game_pieces.push(GamePiece::new(PieceType::Block, None, false, false));
        let puzzle = Puzzle {
            available_game_pieces,
            start_game_board,
        };
        assert_eq!(puzzle.check_setup(), true);
        assert_eq!(puzzle.check_solution(), false);
    }

    #[test]
    fn test_solver_simple() {
        let mut start_game_board = GameBoard::new(2);
        start_game_board.slots[0].occupying_game_piece = Some(GamePiece::new(
            PieceType::Laser,
            Some(Orientation::North),
            true,
            false,
        ));
        start_game_board.slots[6].occupying_game_piece = Some(GamePiece::new(
            PieceType::SingleMirror,
            Some(Orientation::West),
            true,
            true,
        ));
        start_game_board.slots[10].occupying_game_piece = Some(GamePiece::new(
            PieceType::SingleMirror,
            Some(Orientation::South),
            true,
            true,
        ));
        let mut available_game_pieces = vec![];
        available_game_pieces.push(GamePiece::new(
            PieceType::SplittingMirror,
            None,
            false,
            false,
        ));
        let puzzle = Puzzle {
            available_game_pieces,
            start_game_board,
        };
        let t0 = time::Instant::now();
        let result = puzzle.dfs();
        let t1 = time::Instant::now();
        println!("Result: {:?}; \n\nelapsed: {:?}", result, t1 - t0);
        assert!(result.is_some());
    }

    #[test]
    fn test_solver_puzzle_5() {
        let mut start_game_board = GameBoard::new(2);
        start_game_board.slots[1].occupying_game_piece = Some(GamePiece::new(
            PieceType::Block,
            Some(Orientation::North),
            true,
            false,
        ));
        start_game_board.slots[9].occupying_game_piece =
            Some(GamePiece::new(PieceType::SingleMirror, None, true, true));
        start_game_board.slots[21].occupying_game_piece =
            Some(GamePiece::new(PieceType::SingleMirror, None, true, true));
        let mut available_game_pieces = vec![];
        available_game_pieces.push(GamePiece::new(
            PieceType::SplittingMirror,
            None,
            false,
            false,
        ));
        available_game_pieces.push(GamePiece::new(PieceType::Laser, None, false, false));
        let puzzle = Puzzle {
            available_game_pieces,
            start_game_board,
        };
        let t0 = time::Instant::now();
        let result = puzzle.dfs();
        let t1 = time::Instant::now();
        println!("Result: {:?}; elapsed: {:?}", result, t1 - t0);
        assert!(result.is_some());
    }

    // the first puzzle to use every type of piece
    #[test]
    fn test_solver_puzzle_25() {
        let mut start_game_board = GameBoard::new(2);
        start_game_board.slots[3].occupying_game_piece = Some(GamePiece::new(
            PieceType::SingleMirror,
            None,
            true,
            true,
        ));
        start_game_board.slots[7].occupying_game_piece = Some(GamePiece::new(
            PieceType::Gate,
            None,
            true,
            false,
        ));
        start_game_board.slots[8].occupying_game_piece = Some(GamePiece::new(
            PieceType::SplittingMirror,
            None,
            true,
            false,
        ));
        start_game_board.slots[20].occupying_game_piece = Some(GamePiece::new(
            PieceType::Laser,
            None,
            true,
            false,
        ));
        start_game_board.slots[23].occupying_game_piece = Some(GamePiece::new(
            PieceType::Block,
            Some(Orientation::East),
            true,
            false,
        ));

        let mut available_game_pieces = vec![];
        available_game_pieces.push(GamePiece::new(
            PieceType::SingleMirror,
            None,
            false,
            true,
        ));
        available_game_pieces.push(GamePiece::new(
            PieceType::DoubleMirror,
            None,
            false,
            false,
        ));

        let puzzle = Puzzle {
            available_game_pieces,
            start_game_board,
        };

        let t0 = time::Instant::now();
        let result = puzzle.dfs();
        let t1 = time::Instant::now();
        println!("Result: {:?}; elapsed: {:?}", result, t1 - t0);
        assert!(result.is_some());
    }
}
