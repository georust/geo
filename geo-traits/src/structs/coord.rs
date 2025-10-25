use crate::CoordTrait;

use crate::dimension::Dimensions;

/// A parsed coordinate.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Coord<T: Copy> {
    /// The x-coordinate.
    pub x: T,
    /// The y-coordinate.
    pub y: T,
    /// The z-coordinate.
    pub z: Option<T>,
    /// The m-coordinate.
    pub m: Option<T>,
}

impl<T: Copy> Coord<T> {
    /// Creates a new coordinate from a coordinate trait.
    pub fn new(coord: impl CoordTrait<T = T>) -> Self {
        let x = coord.x();
        let y = coord.y();

        match coord.dim() {
            Dimensions::Xyzm | Dimensions::Unknown(_) => Self {
                x,
                y,
                z: coord.nth(2),
                m: coord.nth(3),
            },
            Dimensions::Xyz => Self {
                x,
                y,
                z: coord.nth(2),
                m: None,
            },
            Dimensions::Xym => Self {
                x,
                y,
                z: None,
                m: coord.nth(2),
            },
            Dimensions::Xy => Self {
                x,
                y,
                z: None,
                m: None,
            },
        }
    }

    /// Return the [Dimensions] of this coord.
    pub fn dimension(&self) -> Dimensions {
        match (self.z.is_some(), self.m.is_some()) {
            (true, true) => Dimensions::Xyzm,
            (true, false) => Dimensions::Xyz,
            (false, true) => Dimensions::Xym,
            (false, false) => Dimensions::Xy,
        }
    }
}

impl<T: Copy> CoordTrait for Coord<T> {
    type T = T;

    fn dim(&self) -> Dimensions {
        self.dimension().into()
    }

    fn x(&self) -> Self::T {
        self.x
    }

    fn y(&self) -> Self::T {
        self.y
    }

    fn nth_or_panic(&self, n: usize) -> Self::T {
        let has_z = self.z.is_some();
        let has_m = self.m.is_some();
        match n {
            0 => self.x,
            1 => self.y,
            2 => {
                if has_z {
                    self.z.unwrap()
                } else if has_m {
                    self.m.unwrap()
                } else {
                    panic!("n out of range")
                }
            }
            3 => {
                if has_z && has_m {
                    self.m.unwrap()
                } else {
                    panic!("n out of range")
                }
            }
            _ => panic!("n out of range"),
        }
    }
}

impl<T: Copy> CoordTrait for &Coord<T> {
    type T = T;

    fn dim(&self) -> Dimensions {
        self.dimension().into()
    }

    fn x(&self) -> Self::T {
        self.x
    }

    fn y(&self) -> Self::T {
        self.y
    }

    fn nth_or_panic(&self, n: usize) -> Self::T {
        let has_z = self.z.is_some();
        let has_m = self.m.is_some();
        match n {
            0 => self.x,
            1 => self.y,
            2 => {
                if has_z {
                    self.z.unwrap()
                } else if has_m {
                    self.m.unwrap()
                } else {
                    panic!("n out of range")
                }
            }
            3 => {
                if has_z && has_m {
                    self.m.unwrap()
                } else {
                    panic!("n out of range")
                }
            }
            _ => panic!("n out of range"),
        }
    }
}
