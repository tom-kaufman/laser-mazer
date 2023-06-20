use lazy_static::lazy_static;

#[derive(PartialEq, Debug)]
pub enum PieceType {
    Laser,
    SingleMirror,
    Gate,
    DoubleMirror,
    Block,
    SplittingMirror,
}
use std::fmt;

pub trait GamePiece: fmt::Debug {
    fn new(orientation: Orientation) -> Self
    where
        Self: Sized;
    fn get_orientation(&self) -> Orientation;
    fn get_piece_type(&self) -> PieceType;
    /// this method should also update `lit` and `target_lit`
    fn reference_outbound_lasers_given_inbound_laser_direction(
        &mut self,
        laser_inbound_orientation: Orientation,
    ) -> [Option<Orientation>; 2];
    fn is_lit(&self) -> bool;
    fn is_target_lit(&self) -> bool;

    fn outbound_lasers_given_inbound_laser_direction(
        &mut self,
        laser_inbound_orientation: Orientation,
    ) -> [Option<Orientation>; 2] {
        let reoriented_direction = self
            .get_orientation()
            .reorientate_inbound_laser(laser_inbound_orientation.clone());
        let reoriented_outbound_lasers =
            self.reference_outbound_lasers_given_inbound_laser_direction(reoriented_direction);
        let mut outbound_lasers = [None, None];
        for i in 0..=1 {
            if let Some(outbound_laser) = reoriented_outbound_lasers[i] {
                outbound_lasers[i] = Some(
                    self.get_orientation()
                        .reorientate_outbound_laser(outbound_laser),
                );
            }
        }
        match outbound_lasers {
            [None, Some(_)] => panic!("reference_outbound_lasers_given_inbound_laser_direction() returned a [None, Some(_)]!"),
            _ => outbound_lasers,
        }
    }
}

/// the red piece
/// reference orientation is laser facing north
#[derive(Debug)]
pub struct LaserPiece {
    orientation: Orientation,
}

impl GamePiece for LaserPiece {
    fn new(orientation: Orientation) -> Self {
        Self { orientation }
    }

    fn get_orientation(&self) -> Orientation {
        self.orientation.clone()
    }

    fn get_piece_type(&self) -> PieceType {
        PieceType::Laser
    }

    /// this will be used to kick start the maze; we'll start the puzzle from the laser piece,
    /// and the next laser follows the laser piece's orientation. if the beam loops back and hits
    /// the laser piece, it will yield the same direction again, which the game board will know
    /// the laser is already over that slot in that direction, and stop tracing that laser's
    /// path
    fn reference_outbound_lasers_given_inbound_laser_direction(
        &mut self,
        laser_inbound_orientation: Orientation,
    ) -> [Option<Orientation>; 2] {
        match laser_inbound_orientation {
            _ => [Some(Orientation::North), None],
        }
    }

    fn is_target_lit(&self) -> bool {
        false
    }

    fn is_lit(&self) -> bool {
        true
    }
}

#[derive(Debug)]
pub struct GatePiece {
    orientation: Orientation,
    lit: bool,
}

/// the yellow piece
/// reference orientation is with the transmissive direction N/S
impl GamePiece for GatePiece {
    fn new(orientation: Orientation) -> Self {
        Self {
            orientation,
            lit: false,
        }
    }

    fn get_orientation(&self) -> Orientation {
        self.orientation.clone()
    }

    fn get_piece_type(&self) -> PieceType {
        PieceType::Gate
    }

    fn is_lit(&self) -> bool {
        self.lit
    }

    fn reference_outbound_lasers_given_inbound_laser_direction(
        &mut self,
        laser_inbound_orientation: Orientation,
    ) -> [Option<Orientation>; 2] {
        self.lit = true;
        match laser_inbound_orientation {
            Orientation::North | Orientation::South => [Some(laser_inbound_orientation), None],
            Orientation::West | Orientation::East => [None, None],
        }
    }

    fn is_target_lit(&self) -> bool {
        false
    }
}

/// The purple piece
/// the reference position is with the target facing north
/// this means the mirror has this orientation: \|
#[derive(Debug)]
pub struct SingleMirrorPiece {
    orientation: Orientation,
    lit: bool,
    target_lit: bool,
}

impl GamePiece for SingleMirrorPiece {
    fn new(orientation: Orientation) -> Self {
        Self {
            orientation,
            lit: false,
            target_lit: false,
        }
    }

    fn get_orientation(&self) -> Orientation {
        self.orientation.clone()
    }

    fn get_piece_type(&self) -> PieceType {
        PieceType::SingleMirror
    }

    fn reference_outbound_lasers_given_inbound_laser_direction(
        &mut self,
        laser_inbound_orientation: Orientation,
    ) -> [Option<Orientation>; 2] {
        self.lit = true;
        match laser_inbound_orientation {
            Orientation::North => [Some(Orientation::West), None],
            Orientation::West => [None, None],
            Orientation::South => {
                if !self.target_lit {
                    self.target_lit = true;
                    println!("Hit a target!");
                }
                [None, None]
            }
            Orientation::East => [Some(Orientation::South), None],
        }
    }

    fn is_lit(&self) -> bool {
        self.lit
    }

    fn is_target_lit(&self) -> bool {
        self.target_lit
    }
}

/// The blue piece
/// the reference position is the mirror oriented like \
#[derive(Debug)]
pub struct DoubleMirrorPiece {
    orientation: Orientation,
    lit: bool,
}

impl GamePiece for DoubleMirrorPiece {
    fn new(orientation: Orientation) -> Self {
        Self {
            orientation,
            lit: false,
        }
    }

    fn get_orientation(&self) -> Orientation {
        self.orientation.clone()
    }

    fn get_piece_type(&self) -> PieceType {
        PieceType::DoubleMirror
    }

    fn reference_outbound_lasers_given_inbound_laser_direction(
        &mut self,
        laser_inbound_orientation: Orientation,
    ) -> [Option<Orientation>; 2] {
        self.lit = true;
        match laser_inbound_orientation {
            Orientation::North => [Some(Orientation::West), None],
            Orientation::West => [Some(Orientation::North), None],
            Orientation::South => [Some(Orientation::East), None],
            Orientation::East => [Some(Orientation::South), None],
        }
    }

    fn is_lit(&self) -> bool {
        self.lit
    }

    fn is_target_lit(&self) -> bool {
        false
    }
}

/// The black piece
#[derive(Debug)]
pub struct BlockPiece {
    orientation: Orientation,
}

impl GamePiece for BlockPiece {
    fn new(orientation: Orientation) -> Self {
        Self {
            orientation,
        }
    }

    fn get_orientation(&self) -> Orientation {
        self.orientation.clone()
    }

    fn get_piece_type(&self) -> PieceType {
        PieceType::Block
    }

    fn reference_outbound_lasers_given_inbound_laser_direction(  // TODO
        &mut self,
        laser_inbound_orientation: Orientation,
    ) -> [Option<Orientation>; 2] {
        [Some(laser_inbound_orientation), None]
    }

    fn is_lit(&self) -> bool {  // TODO
        true
    }

    fn is_target_lit(&self) -> bool {  // TODO
        false
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
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

/// The green piece
/// the reference position is the mirror oriented like \
#[derive(Debug)]
pub struct SplittingMirrorPiece {
    orientation: Orientation,
    lit: bool,
}

impl GamePiece for SplittingMirrorPiece {
    fn new(orientation: Orientation) -> Self {
        Self {
            orientation,
            lit: false,
        }
    }

    fn get_orientation(&self) -> Orientation {
        self.orientation.clone()
    }

    fn get_piece_type(&self) -> PieceType {
        PieceType::SplittingMirror
    }

    fn reference_outbound_lasers_given_inbound_laser_direction(
        &mut self,
        laser_inbound_orientation: Orientation,
    ) -> [Option<Orientation>; 2] {
        self.lit = true;
        match laser_inbound_orientation {
            // this piece is the only one to return two beams
            // in this match statement, the item[0] acts just like the blue double mirror piece
            // while item[1] is the transmitted beam
            Orientation::North => [Some(Orientation::West), Some(laser_inbound_orientation)],
            Orientation::West => [Some(Orientation::North), Some(laser_inbound_orientation)],
            Orientation::South => [Some(Orientation::East), Some(laser_inbound_orientation)],
            Orientation::East => [Some(Orientation::South), Some(laser_inbound_orientation)],
        }
    }

    fn is_lit(&self) -> bool {
        self.lit
    }

    fn is_target_lit(&self) -> bool {
        false
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