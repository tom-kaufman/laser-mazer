use crate::orientation::Orientation;
use std::ops;

#[derive(Clone, Debug)]
pub struct Token {
    type_: TokenType,
    pub orientation: Option<Orientation>,
    pub lit: bool,
    target_lit: Option<bool>,
    must_light: bool,
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
        let lit = type_ == TokenType::CellBlocker;
        Self {
            type_,
            orientation,
            lit,
            target_lit,
            must_light,
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
    ) -> [Option<Orientation>; 2] {
        let reoriented_direction = self
            .orientation
            .as_mut()
            .expect("Called check() with tokens still not having orientation set")
            .reorient_inbound_laser(laser_inbound_orientation);
        let reoriented_outbound_lasers =
            self.reference_outbound_lasers_given_inbound_laser_direction(reoriented_direction);
        let mut outbound_lasers = [None, None];
        for i in 0..2 {
            if let Some(laser) = &reoriented_outbound_lasers[i] {
                outbound_lasers[i] = Some(
                    self.orientation
                        .as_ref()
                        .expect("Called check() with tokens still not having orientation set")
                        .reorient_outbound_laser(laser),
                );
            }
        }

        outbound_lasers
    }

    // uses reference orientation for each piece to calculate its interaction with an inbound laser
    // also marks the pieces as lit
    fn reference_outbound_lasers_given_inbound_laser_direction(
        &mut self,
        laser_inbound_orientation: Orientation,
    ) -> [Option<Orientation>; 2] {
        match self.type_ {
            TokenType::Laser => [Some(Orientation::North), None],

            TokenType::Checkpoint => {
                self.lit = true;
                match laser_inbound_orientation {
                    Orientation::North | Orientation::South => {
                        [Some(laser_inbound_orientation), None]
                    }
                    Orientation::West | Orientation::East => [None, None],
                }
            }

            TokenType::TargetMirror => {
                self.lit = true;
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

            TokenType::DoubleMirror => {
                self.lit = true;
                match laser_inbound_orientation {
                    Orientation::North => [Some(Orientation::West), None],
                    Orientation::West => [Some(Orientation::North), None],
                    Orientation::South => [Some(Orientation::East), None],
                    Orientation::East => [Some(Orientation::South), None],
                }
            }

            TokenType::CellBlocker => [Some(laser_inbound_orientation), None],

            TokenType::BeamSplitter => {
                self.lit = true;
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

    pub fn orientation_range(&self) -> ops::Range<usize> {
        match self.type_ {
            TokenType::BeamSplitter => 0..2,
            TokenType::DoubleMirror => 0..2,
            TokenType::Checkpoint => 0..2,
            TokenType::CellBlocker => 0..1,
            _ => 0..4,
        }
    }
}

#[derive(PartialEq, Copy, Clone, Eq, Hash, Debug)]
pub enum TokenType {
    Laser,
    TargetMirror,
    BeamSplitter,
    DoubleMirror,
    Checkpoint,
    CellBlocker,
}
