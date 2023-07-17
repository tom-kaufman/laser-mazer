use crate::orientation::Orientation;

#[derive(Clone, Debug)]
pub struct ActiveLaser {
    pub slot_index: usize,
    pub orientation: Orientation,
}

impl ActiveLaser {
    pub fn next_position(&self) -> Option<usize> {
        match self.orientation {
            // if we're not on the top row, increment index by 5
            Orientation::North => {
                if self.slot_index >= 20 {
                    None
                } else {
                    Some(self.slot_index + 5)
                }
            }
            // if we're not on the right column, increment by 1
            Orientation::East => {
                if self.slot_index % 5 == 4 {
                    None
                } else {
                    Some(self.slot_index + 1)
                }
            }
            // if we're not on the bottom row, decrement index by 5
            Orientation::South => {
                if self.slot_index <= 4 {
                    return None;
                } else {
                    Some(self.slot_index - 5)
                }
            }
            // if we're not on the left column, decrement by 1
            Orientation::West => {
                if self.slot_index % 5 == 0 {
                    None
                } else {
                    Some(self.slot_index - 1)
                }
            }
        }
    }
}