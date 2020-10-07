use crate::GeoNum;

/// Utility functions for working with quadrants of the cartesian plane,
/// which are labeled as follows:
/// ```ignore
///          (+)
///        NW ┃ NE
///    (-) ━━━╋━━━━ (+)
///        SW ┃ SE
///          (-)
/// ```
#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
pub enum Quadrant {
    NE,
    NW,
    SW,
    SE,
}

impl Quadrant {
    pub fn new<F: GeoNum>(dx: F, dy: F) -> Option<Quadrant> {
        if dx.is_zero() && dy.is_zero() {
            return None;
        }

        match (dy >= F::zero(), dx >= F::zero()) {
            (true, true) => Quadrant::NE,
            (true, false) => Quadrant::NW,
            (false, false) => Quadrant::SW,
            (false, true) => Quadrant::SE,
        }
        .into()
    }
}
