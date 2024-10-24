use crate::solver::orientation::Orientation;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Token {
    type_: TokenType,
    pub orientation: Option<Orientation>,
    pub lit: bool,
    target_lit: Option<bool>,
    must_light: bool,
}

#[derive(Debug, Clone)]
pub enum LaserTokenInteractionResult {
    // The laser interacts and is re-emitted
    OutboundLaser(Orientation),
    // The laser interacts and is not re-emitted.
    // `valid` indicates if it's a valid result (with regards to the puzzle),
    // i.e., a target mirror absorbs the laser, or a token only emits one laser,
    // or if it's an invalid result
    // i.e. the laser is hitting the wrong side of a Checkpoint token
    NoOutboundLaser { valid: bool },
}

impl Token {
    pub fn new(type_: TokenType, orientation: Option<Orientation>, must_light: bool) -> Self {
        let must_light = if type_ == TokenType::TargetMirror {
            must_light
        } else {
            false
        };
        let target_lit = if type_ == TokenType::TargetMirror {
            Some(false)
        } else {
            None
        };
        let orientation = if type_ == TokenType::CellBlocker {
            Some(Orientation::North)
        } else {
            orientation
        };
        let lit = (type_ == TokenType::CellBlocker) || (type_ == TokenType::Laser);
        Self {
            type_,
            orientation,
            lit,
            target_lit,
            must_light,
        }
    }

    pub fn reset(&mut self) {
        self.lit = (self.type_ == TokenType::CellBlocker) || (self.type_ == TokenType::Laser);
        if self.target_lit.is_some() {
            self.target_lit = Some(false);
        }
    }

    // getter for private field
    pub fn type_(&self) -> &TokenType {
        &self.type_
    }

    // getter for private field
    pub fn must_light(&self) -> bool {
        self.must_light
    }

    pub fn target_lit(&self) -> Option<bool> {
        self.target_lit
    }

    // getter for private field
    pub fn orientation(&self) -> Option<&Orientation> {
        self.orientation.as_ref()
    }

    pub fn outbound_lasers_given_inbound_laser_direction(
        &mut self,
        laser_inbound_orientation: &Orientation,
    ) -> [LaserTokenInteractionResult; 2] {
        let reoriented_direction = self
            .orientation
            .as_mut()
            .expect("Called check() with tokens still not having orientation set")
            .reorient_inbound_laser(laser_inbound_orientation);
        let reoriented_outbound_lasers =
            self.reference_outbound_lasers_given_inbound_laser_direction(reoriented_direction);
        // initialize array with some defaults that we'll overwrite
        let mut outbound_lasers = [
            LaserTokenInteractionResult::NoOutboundLaser { valid: false },
            LaserTokenInteractionResult::NoOutboundLaser { valid: false },
        ];
        for i in 0..2 {
            outbound_lasers[i] = match &reoriented_outbound_lasers[i] {
                LaserTokenInteractionResult::OutboundLaser(laser) => {
                    LaserTokenInteractionResult::OutboundLaser(
                        self.orientation
                            .as_ref()
                            .expect("Called check() with tokens still not having orientation set")
                            .reorient_outbound_laser(laser),
                    )
                }
                x => x.clone(),
            }
        }

        outbound_lasers
    }

    // uses reference orientation for each piece to calculate its interaction with an inbound laser
    // also marks the pieces as lit
    fn reference_outbound_lasers_given_inbound_laser_direction(
        &mut self,
        laser_inbound_orientation: Orientation,
    ) -> [LaserTokenInteractionResult; 2] {
        type R = LaserTokenInteractionResult;
        const NONE_VALID: R = R::NoOutboundLaser { valid: true };
        const NONE_INVALID: R = R::NoOutboundLaser { valid: false };

        match self.type_ {
            TokenType::Laser => {
                match laser_inbound_orientation {
                    // The laser is shining back into the laser source
                    Orientation::South => [NONE_VALID, NONE_VALID],
                    // The laser is returning to the laser token on a wall-side of the laser
                    _ => [NONE_INVALID, NONE_INVALID],
                }
            }
            TokenType::Checkpoint => match laser_inbound_orientation {
                Orientation::North | Orientation::South => {
                    self.lit = true;
                    [R::OutboundLaser(laser_inbound_orientation), NONE_VALID]
                }
                Orientation::West | Orientation::East => [NONE_INVALID, NONE_INVALID],
            },

            TokenType::TargetMirror => {
                self.lit = true;
                match laser_inbound_orientation {
                    Orientation::North => [R::OutboundLaser(Orientation::West), NONE_VALID],
                    Orientation::West => [NONE_INVALID, NONE_INVALID],
                    Orientation::South => {
                        self.target_lit = Some(true);
                        [NONE_VALID, NONE_VALID]
                    }
                    Orientation::East => [R::OutboundLaser(Orientation::South), NONE_VALID],
                }
            }

            TokenType::DoubleMirror => {
                self.lit = true;
                match laser_inbound_orientation {
                    Orientation::North => [R::OutboundLaser(Orientation::West), NONE_VALID],
                    Orientation::West => [R::OutboundLaser(Orientation::North), NONE_VALID],
                    Orientation::South => [R::OutboundLaser(Orientation::East), NONE_VALID],
                    Orientation::East => [R::OutboundLaser(Orientation::South), NONE_VALID],
                }
            }

            TokenType::CellBlocker => [R::OutboundLaser(laser_inbound_orientation), NONE_VALID],

            TokenType::BeamSplitter => {
                self.lit = true;
                match laser_inbound_orientation {
                    // this piece is the only one to return two beams
                    // in this match statement, the item[0] acts just like the blue double mirror piece
                    // while item[1] is the transmitted beam
                    Orientation::North => [
                        R::OutboundLaser(Orientation::West),
                        R::OutboundLaser(laser_inbound_orientation),
                    ],
                    Orientation::West => [
                        R::OutboundLaser(Orientation::North),
                        R::OutboundLaser(laser_inbound_orientation),
                    ],
                    Orientation::South => [
                        R::OutboundLaser(Orientation::East),
                        R::OutboundLaser(laser_inbound_orientation),
                    ],
                    Orientation::East => [
                        R::OutboundLaser(Orientation::South),
                        R::OutboundLaser(laser_inbound_orientation),
                    ],
                }
            }
        }
    }

    pub fn toggle_must_light(&mut self) {
        if self.type_ == TokenType::TargetMirror {
            self.must_light = !self.must_light;
        }
    }
}

#[derive(PartialEq, Copy, Clone, Eq, Hash, Debug, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TokenType {
    Laser,
    TargetMirror,
    BeamSplitter,
    DoubleMirror,
    Checkpoint,
    CellBlocker,
}

impl TokenType {
    // considers the symmetry of the pieces
    pub fn orientation_range(&self) -> Vec<usize> {
        match self {
            TokenType::BeamSplitter => vec![0, 1],
            TokenType::DoubleMirror => vec![0, 1],
            TokenType::Checkpoint => vec![0, 1],
            TokenType::CellBlocker => vec![0],
            _ => vec![0, 1, 2, 3],
        }
    }
}

lazy_static! {
    pub static ref TOKEN_TYPES: [TokenType; 6] = [
        TokenType::Laser,
        TokenType::TargetMirror,
        TokenType::BeamSplitter,
        TokenType::DoubleMirror,
        TokenType::Checkpoint,
        TokenType::CellBlocker,
    ];
}
