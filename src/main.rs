use std::collections::HashMap;

mod pieces;
use pieces::{GamePiece, Orientation, PieceType, ORIENTATION_ORDER};

#[derive(Debug)]
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
            println!(
                "Invalid coordinates: {}, {}",
                position_coordinates.0, position_coordinates.1
            );
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

#[derive(Debug)]
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

        // make sure one piece is a laser
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

        // make sure one piece is a purple target piece
        if !(self.slots.iter().any(|slot| {
            if let Some(piece) = &slot.occupying_game_piece {
                piece.get_piece_type() == PieceType::SingleMirror
            } else {
                false
            }
        })) {
            false
        } else {
            true
        }

        // TODO
        // count purple pieces <= 5
        // count yellow pieces <= 1
        // count green pieces <= 2
        // count black piece <= 1
        // check blue piece <= 1
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
                    println!("Marching forward laser {:?}", laser_position);
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
                        println!("Marching the laser forward, it's now at slot index {}", neighboring_slot_index);
                        // the next slot in the laser's path is on the board
                        let neighboring_slot_active_direction = self.slots.get_mut(neighboring_slot_index as usize).expect("we just validated that we are on the board").active_laser_directions.get_mut(&laser_position.orientation).expect("this hashmap is populated with all the keys from the Orientation enum");
                        if *neighboring_slot_active_direction {
                            return [None, None];
                        }
                        // the laser hasn't traveled over this slot in this direction yet
                        *neighboring_slot_active_direction = true;

                        if let Some(neighboring_piece) = self.slots.get_mut(neighboring_slot_index as usize).expect("we just validated that we are on the board").occupying_game_piece.as_mut() {
                            // the laser has hit a piece, we need to calculate the result
                            println!("The laser has hit a piece of type {:?}", neighboring_piece.get_piece_type());
                            let returned_orientations = neighboring_piece.outbound_lasers_given_inbound_laser_direction(laser_position.orientation);
                            println!("After hitting the piece, the laser became these orientations: {:?}", returned_orientations);
                            let mut result = [None, None];
                            for i in 0..2 {
                                if let Some(orientation) = returned_orientations[i] {
                                    result[i] = Some(LaserPosition::new(neighboring_slot_index, orientation));
                                }
                            }
                            println!("Reconstructed those orientations into these laser positions: {:?}", result);
                            return result
                        } else {
                            // the laser hasn't hit a piece
                            println!("The laser hasn't hit a piece, it's now over index {neighboring_slot_index} with orientation {:?}", laser_position.orientation);
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

        println!("has active lasers? {}", self.has_active_lasers());
        while self.has_active_lasers() {
            //println!("TURN {}\n\n{:?}\n\n", self.turns, self);
            println!("TURN {}\n\n", self.turns);
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
        self.valid_solution = Some(self.count_lit_targets() == self.targets);
        self
    }
}

struct Puzzle {
    start_game_board: GameBoard,
    available_game_pieces: Vec<GamePiece>,
}

impl Puzzle {
    fn wipe_board(mut self) -> Self {
        // TODO
        // move all non-starting pieces off the board back to the available game pieces
        self
    }

    fn check_solution(&mut self) {
        // TODO
    }
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod test {
    use super::*;

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
        ));

        // splitting mirror piece on center col, top row slot
        game_board.slots.get_mut(22).unwrap().occupying_game_piece = Some(GamePiece::new(
            PieceType::SplittingMirror,
            Some(Orientation::East),
            true,
        ));

        // target 1: top left slot, target facing east
        game_board.slots.get_mut(20).unwrap().occupying_game_piece = Some(GamePiece::new(
            PieceType::SingleMirror,
            Some(Orientation::East),
            true,
        ));

        // gate piece, middle col  row[3]
        game_board.slots.get_mut(17).unwrap().occupying_game_piece = Some(GamePiece::new(
            PieceType::Gate,
            Some(Orientation::South),
            true,
        ));

        // block piece, true center
        game_board.slots.get_mut(12).unwrap().occupying_game_piece = Some(GamePiece::new(
            PieceType::Block,
            Some(Orientation::West),
            true,
        ));

        // splitting mirror piece on center col, row[1] slot
        game_board.slots.get_mut(7).unwrap().occupying_game_piece = Some(GamePiece::new(
            PieceType::SplittingMirror,
            Some(Orientation::East),
            true,
        ));

        // double mirror piece on bottom middle slot, facing south
        game_board.slots.get_mut(2).unwrap().occupying_game_piece = Some(GamePiece::new(
            PieceType::DoubleMirror,
            Some(Orientation::South),
            true,
        ));

        // target 2: left col, row[1] slot, facing east
        game_board.slots.get_mut(5).unwrap().occupying_game_piece = Some(GamePiece::new(
            PieceType::SingleMirror,
            Some(Orientation::East),
            true,
        ));

        // target 3: bottom right slot, facing west
        game_board.slots.get_mut(4).unwrap().occupying_game_piece = Some(GamePiece::new(
            PieceType::SingleMirror,
            Some(Orientation::West),
            true,
        ));

        game_board = game_board.calculate_result();
        println!("lit targets: {}", game_board.count_lit_targets());
        assert!(game_board.valid_solution.unwrap())
    }
}
