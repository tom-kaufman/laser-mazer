use lazy_static::lazy_static;

#[derive(Clone, Debug)]
pub enum Orientation {
    North,
    East,
    South,
    West,
}

impl Orientation {
    pub fn to_index(&self) -> usize {
        match self {
            Self::North => 0,
            Self::East => 1,
            Self::South => 2,
            Self::West => 3,
        }
    }

    pub fn from_index(idx: usize) -> Self {
        ORIENTATION_ORDER[idx].clone()
    }

    /// This function prevents us from needing to nest matches to consider the relative orientation
    /// of the piece and inbound lasers, by first rotating the orientation to the reference orientation.
    /// can't use reorientatate_by_offset because of the subtraction
    /// `self` should be the orientation of the piece, `inbound_orientation` is the laser's direction in the original reference frame.
    /// returns the laser's orientation in the reference orientation (North)
    pub fn reorient_inbound_laser(&self, inbound_orientation: &Orientation) -> Self {
        let self_orientation_ordinal_value = self.to_index();
        let laser_orientation_ordinal_value = inbound_orientation.to_index();
        let idx = laser_orientation_ordinal_value.wrapping_sub(self_orientation_ordinal_value) % 4;
        Self::from_index(idx)
    }

    /// This function prevents us from needing to nest matches to consider the relative orientation
    /// of the piece and outbound lasers, by rotating back to the original reference frame.
    /// `self` should be the orientation of the Token, and `outbound_orientation` is the outbound orientation in the outbound laser's orientation
    /// after calculating the interaction at the reference orientation. returns the orientation of the outbound laser in the original reference frame.
    pub fn reorient_outbound_laser(&self, outbound_orientation: &Orientation) -> Self {
        self.reorient_by_offset(outbound_orientation.to_index())
    }

    fn reorient_by_offset(&self, offset: usize) -> Self {
        let self_orientation_ordinal_value = self.to_index();
        let idx = (self_orientation_ordinal_value + offset) % 4;
        ORIENTATION_ORDER[idx].clone()
    }

}

lazy_static! {
    static ref ORIENTATION_ORDER: [Orientation; 4] = [
        Orientation::North,
        Orientation::East,
        Orientation::South,
        Orientation::West
    ];
}