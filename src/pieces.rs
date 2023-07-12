use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Hash, Eq, Copy)]
pub enum PieceType {
    Laser,
    SingleMirror,
    Gate,
    DoubleMirror,
    Block,
    SplittingMirror,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct GamePiece {
    piece_type: PieceType,
    pub orientation: Option<Orientation>,
    lit: Option<bool>,
    target_lit: Option<bool>,
    starting_piece: bool,
    must_light: bool,
}

impl GamePiece {
    pub fn new(
        piece_type: PieceType,
        orientation: Option<Orientation>,
        starting_piece: bool,
        mut must_light: bool,
    ) -> Self {
        let (lit, target_lit) = match piece_type {
            PieceType::Laser => (Some(true), None),
            PieceType::SingleMirror => (Some(false), Some(false)),
            PieceType::Gate => (None, None),
            _ => (Some(false), None),
        };
        if piece_type != PieceType::SingleMirror && must_light {
            println!("WARNING tried to make a non-target piece required to be lit");
            must_light = false;
        }
        Self {
            piece_type,
            lit,
            target_lit,
            orientation,
            starting_piece,
            must_light,
        }
    }

    pub fn get_orientation(&self) -> Option<Orientation> {
        self.orientation.clone()
    }

    pub fn set_orientation_by_index(&mut self, idx: usize) {
        self.orientation = Some(ORIENTATION_ORDER[idx].clone());
    }

    pub fn get_piece_type(&self) -> PieceType {
        self.piece_type.clone()
    }

    pub fn is_lit(&self) -> Option<bool> {
        self.lit
    }

    pub fn is_target_lit(&self) -> Option<bool> {
        self.target_lit
    }

    pub fn is_starting_piece(&self) -> bool {
        self.starting_piece
    }

    pub fn must_light(&self) -> bool {
        self.must_light
    }

    pub fn outbound_lasers_given_inbound_laser_direction(
        &mut self,
        laser_inbound_orientation: Orientation,
    ) -> [Option<Orientation>; 2] {
        let reoriented_direction = self
            .get_orientation()
            .expect("method shouldn't be called if all orientations are not set")
            .reorientate_inbound_laser(laser_inbound_orientation.clone());
        let reoriented_outbound_lasers =
            self.reference_outbound_lasers_given_inbound_laser_direction(reoriented_direction);
        let mut outbound_lasers = [None, None];
        for i in 0..=1 {
            if let Some(outbound_laser) = reoriented_outbound_lasers[i] {
                outbound_lasers[i] = Some(
                    self.get_orientation()
                        .expect("method shouldn't be called if all orientations are not set")
                        .reorientate_outbound_laser(outbound_laser),
                );
            }
        }
        match outbound_lasers {
            [None, Some(_)] => panic!("reference_outbound_lasers_given_inbound_laser_direction() returned a [None, Some(_)]!"),
            _ => outbound_lasers,
        }
    }

    fn reference_outbound_lasers_given_inbound_laser_direction(
        &mut self,
        laser_inbound_orientation: Orientation,
    ) -> [Option<Orientation>; 2] {
        match self.piece_type {
            PieceType::Laser => match laser_inbound_orientation {
                _ => [Some(Orientation::North), None],
            },

            PieceType::Gate => {
                self.lit = Some(true);
                match laser_inbound_orientation {
                    Orientation::North | Orientation::South => {
                        [Some(laser_inbound_orientation), None]
                    }
                    Orientation::West | Orientation::East => [None, None],
                }
            }

            PieceType::SingleMirror => {
                self.lit = Some(true);
                match laser_inbound_orientation {
                    Orientation::North => [Some(Orientation::West), None],
                    Orientation::West => [None, None],
                    Orientation::South => {
                        self.target_lit = Some(true);
                        [None, None]
                    }
                    Orientation::East => [Some(Orientation::South), None],
                }
            }

            PieceType::DoubleMirror => {
                self.lit = Some(true);
                match laser_inbound_orientation {
                    Orientation::North => [Some(Orientation::West), None],
                    Orientation::West => [Some(Orientation::North), None],
                    Orientation::South => [Some(Orientation::East), None],
                    Orientation::East => [Some(Orientation::South), None],
                }
            }

            PieceType::Block => [Some(laser_inbound_orientation), None],

            PieceType::SplittingMirror => {
                self.lit = Some(true);
                match laser_inbound_orientation {
                    // this piece is the only one to return two beams
                    // in this match statement, the item[0] acts just like the blue double mirror piece
                    // while item[1] is the transmitted beam
                    Orientation::North => {
                        [Some(Orientation::West), Some(laser_inbound_orientation)]
                    }
                    Orientation::West => {
                        [Some(Orientation::North), Some(laser_inbound_orientation)]
                    }
                    Orientation::South => {
                        [Some(Orientation::East), Some(laser_inbound_orientation)]
                    }
                    Orientation::East => {
                        [Some(Orientation::South), Some(laser_inbound_orientation)]
                    }
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Orientation {
    North,
    South,
    West,
    East,
}

// TODO benchmark inlining these methods
impl Orientation {
    /// This function prevents us from needing to nest matches to consider the relative orientation
    /// of the piece and inbound lasers.
    /// can't use reorientatate_by_offset because of the subtraction
    pub fn reorientate_inbound_laser(&self, inbound_orientation: Orientation) -> Self {
        let self_orientation_ordinal_value = self.ordinal_value();
        let laser_orientation_ordinal_value = inbound_orientation.ordinal_value();
        let idx = laser_orientation_ordinal_value.wrapping_sub(self_orientation_ordinal_value) % 4;
        ORIENTATION_ORDER[idx].clone()
    }

    /// This function prevents us from needing to nest matches to consider the relative orientation
    /// of the piece and outbound lasers.
    fn reorientate_outbound_laser(&self, outbound_orientation: Orientation) -> Self {
        self.reorientatate_by_offset(outbound_orientation.ordinal_value())
    }

    fn reorientatate_by_offset(&self, offset: usize) -> Self {
        let self_orientation_ordinal_value = self.ordinal_value();
        let idx = (self_orientation_ordinal_value + offset) % 4;
        ORIENTATION_ORDER[idx].clone()
    }

    /// + is CW, 0 is North
    fn ordinal_value(&self) -> usize {
        match self {
            Self::North => 0,
            Self::East => 1,
            Self::South => 2,
            Self::West => 3,
        }
    }
}

lazy_static! {
    pub static ref ORIENTATION_ORDER: [Orientation; 4] = [
        Orientation::North,
        Orientation::East,
        Orientation::South,
        Orientation::West
    ];
}
