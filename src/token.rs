use crate::orientation::{Orientation, };

#[derive(Clone)]
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
    ) -> Vec<Orientation> {
        let reoriented_direction = self
            .orientation
            .as_mut()
            .expect("Called check() with tokens still not having orientation set")
            .reorient_inbound_laser(laser_inbound_orientation);
        let mut reoriented_outbound_lasers =
            self.reference_outbound_lasers_given_inbound_laser_direction(reoriented_direction);
        for outbound_laser in &mut reoriented_outbound_lasers {
            *outbound_laser = self.orientation.as_mut().expect("Called check() with tokens still not having orientation set").reorient_outbound_laser(&outbound_laser);
        }

        reoriented_outbound_lasers
    }

    // uses reference orientation for each piece to calculate its interaction with an inbound laser
    // also marks the pieces as lit
    fn reference_outbound_lasers_given_inbound_laser_direction(
        &mut self,
        laser_inbound_orientation: Orientation,
    ) -> Vec<Orientation> {
        match self.type_ {
            TokenType::Laser => match laser_inbound_orientation {
                _ => vec![Orientation::North],
            },

            TokenType::Checkpoint => {
                self.lit = true;
                match laser_inbound_orientation {
                    Orientation::North | Orientation::South => {
                        vec![laser_inbound_orientation]
                    }
                    Orientation::West | Orientation::East => vec![],
                }
            }

            TokenType::TargetMirror => {
                self.lit = true;
                match laser_inbound_orientation {
                    Orientation::North => vec![Orientation::West],
                    Orientation::West => vec![],
                    Orientation::South => {
                        self.target_lit = Some(true);
                        vec![]
                    }
                    Orientation::East => vec![Orientation::South],
                }
            }

            TokenType::DoubleMirror => {
                self.lit = true;
                match laser_inbound_orientation {
                    Orientation::North => vec![Orientation::West],
                    Orientation::West => vec![Orientation::North],
                    Orientation::South => vec![Orientation::East],
                    Orientation::East => vec![Orientation::South],
                }
            }

            TokenType::CellBlocker => vec![laser_inbound_orientation],

            TokenType::BeamSplitter => {
                self.lit = true;
                match laser_inbound_orientation {
                    // this piece is the only one to return two beams
                    // in this match statement, the item[0] acts just like the blue double mirror piece
                    // while item[1] is the transmitted beam
                    Orientation::North => {
                        vec![Orientation::West, laser_inbound_orientation]
                    }
                    Orientation::West => {
                        vec![Orientation::North, laser_inbound_orientation]
                    }
                    Orientation::South => {
                        vec![Orientation::East, laser_inbound_orientation]
                    }
                    Orientation::East => {
                        vec![Orientation::South, laser_inbound_orientation]
                    }
                }
            }
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
