use crate::dimension::Dimensions;
use crate::CoordTrait;

/// A parsed coordinate.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Coord<T: Copy = f64> {
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
    /// Creates a new coordinate.
    pub fn new(x: T, y: T, z: Option<T>, m: Option<T>) -> Self {
        Self { x, y, z, m }
    }

    /// Creates a new coordinate from a coordinate trait.
    pub fn from_coord(coord: &impl CoordTrait<T = T>) -> Self {
        let x = coord.x();
        let y = coord.y();

        match coord.dim() {
            Dimensions::Xy | Dimensions::Unknown(2) => Self {
                x,
                y,
                z: None,
                m: None,
            },
            Dimensions::Xyz | Dimensions::Unknown(3) => Self {
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
            // For >4 dimension of Unknown, we don't know how to handle it;
            // we simply discard these extra dimensions.
            Dimensions::Xyzm | Dimensions::Unknown(_) => Self {
                x,
                y,
                z: coord.nth(2),
                m: coord.nth(3),
            },
        }
    }

    /// Creates a new coordinate from X and Y coordinates.
    pub fn from_xy(x: T, y: T) -> Self {
        Self {
            x,
            y,
            z: None,
            m: None,
        }
    }

    /// Creates a new coordinate from X, Y and Z coordinates.
    pub fn from_xyz(x: T, y: T, z: T) -> Self {
        Self {
            x,
            y,
            z: Some(z),
            m: None,
        }
    }

    /// Creates a new coordinate from X, Y, and M coordinates.
    pub fn from_xym(x: T, y: T, m: T) -> Self {
        Self {
            x,
            y,
            z: None,
            m: Some(m),
        }
    }

    /// Creates a new coordinate from X, Y, Z, and M coordinates.
    pub fn from_xyzm(x: T, y: T, z: T, m: T) -> Self {
        Self {
            x,
            y,
            z: Some(z),
            m: Some(m),
        }
    }
}

impl<T: Copy> CoordTrait for Coord<T> {
    type T = T;

    fn dim(&self) -> Dimensions {
        match (self.z.is_some(), self.m.is_some()) {
            (true, true) => Dimensions::Xyzm,
            (true, false) => Dimensions::Xyz,
            (false, true) => Dimensions::Xym,
            (false, false) => Dimensions::Xy,
        }
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
        (*self).dim()
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

/// Convert from a tuple of (X, Y).
impl<T: Copy> From<(T, T)> for Coord<T> {
    fn from((x, y): (T, T)) -> Self {
        Self::new(x, y, None, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CoordTrait;

    #[derive(Clone, Copy)]
    struct DummyCoord<T: Copy> {
        dims: Dimensions,
        coords: [T; 4],
    }

    impl<T: Copy> DummyCoord<T> {
        fn new(coords: [T; 4], dims: Dimensions) -> Self {
            Self { dims, coords }
        }
    }

    impl<T: Copy> CoordTrait for DummyCoord<T> {
        type T = T;

        fn dim(&self) -> Dimensions {
            self.dims
        }

        fn nth_or_panic(&self, n: usize) -> Self::T {
            assert!(n < self.dims.size());
            self.coords[n]
        }

        fn x(&self) -> Self::T {
            self.coords[0]
        }

        fn y(&self) -> Self::T {
            self.coords[1]
        }
    }

    #[test]
    fn coord_new_from_tuple_xy() {
        let coord = Coord::from_coord(&(1_i32, 2_i32));
        assert_eq!(coord.x, 1);
        assert_eq!(coord.y, 2);
        assert_eq!(coord.z, None);
        assert_eq!(coord.m, None);
        assert_eq!(CoordTrait::dim(&coord), Dimensions::Xy);
    }

    #[test]
    fn coord_new_from_xyz() {
        let source = DummyCoord::new([1.0_f64, 2.0, 3.0, 0.0], Dimensions::Xyz);
        let coord = Coord::from_coord(&source);
        assert_eq!(coord.z, Some(3.0));
        assert_eq!(coord.m, None);
        assert_eq!(coord.nth_or_panic(2), 3.0);
        assert_eq!(CoordTrait::dim(&coord), Dimensions::Xyz);
    }

    #[test]
    fn coord_new_from_xym() {
        let source = DummyCoord::new([4_u32, 5, 6, 0], Dimensions::Xym);
        let coord = Coord::from_coord(&source);
        assert_eq!(coord.z, None);
        assert_eq!(coord.m, Some(6));
        assert_eq!(coord.nth_or_panic(2), 6);
        assert_eq!(CoordTrait::dim(&coord), Dimensions::Xym);
    }

    #[test]
    fn coord_new_from_xyzm() {
        let source = DummyCoord::new([7_i16, 8, 9, 10], Dimensions::Xyzm);
        let coord = Coord::from_coord(&source);
        assert_eq!(coord.z, Some(9));
        assert_eq!(coord.m, Some(10));
        assert_eq!(coord.nth_or_panic(2), 9);
        assert_eq!(coord.nth_or_panic(3), 10);
        assert_eq!(CoordTrait::dim(&coord), Dimensions::Xyzm);
    }

    #[test]
    fn coord_from_tuple_xy() {
        let coord: Coord<i32> = Coord::from((11, 12));
        assert_eq!(coord, Coord::from_xy(11, 12));
        assert_eq!(coord.z, None);
        assert_eq!(coord.m, None);
        assert_eq!(CoordTrait::dim(&coord), Dimensions::Xy);
    }
}
