use crate::dimension::Dimensions;
use crate::structs::Coord;
use crate::{CoordTrait, PointTrait};

/// A parsed Point.
#[derive(Clone, Debug, PartialEq)]
pub struct Point<T: Copy = f64> {
    pub(crate) coord: Option<Coord<T>>,
    pub(crate) dim: Dimensions,
}

impl<T: Copy> Point<T> {
    /// Create a new Point from a coordinate and known [Dimensions].
    pub fn new(coord: Option<Coord<T>>, dim: Dimensions) -> Self {
        Self { coord, dim }
    }

    /// Create a new empty point.
    pub fn empty(dim: Dimensions) -> Self {
        Self::new(None, dim)
    }

    /// Creates a new coordinate from X and Y coordinates.
    pub fn from_xy(x: T, y: T) -> Self {
        Self::new(Some(Coord::from_xy(x, y)), Dimensions::Xy)
    }

    /// Creates a new coordinate from X, Y and Z coordinates.
    pub fn from_xyz(x: T, y: T, z: T) -> Self {
        Self::new(Some(Coord::from_xyz(x, y, z)), Dimensions::Xyz)
    }

    /// Creates a new coordinate from X, Y, and M coordinates.
    pub fn from_xym(x: T, y: T, m: T) -> Self {
        Self::new(Some(Coord::from_xyz(x, y, m)), Dimensions::Xym)
    }

    /// Creates a new coordinate from X, Y, Z, and M coordinates.
    pub fn from_xyzm(x: T, y: T, z: T, m: T) -> Self {
        Self::new(Some(Coord::from_xyzm(x, y, z, m)), Dimensions::Xyzm)
    }

    /// Return the [Dimensions] of this geometry.
    pub fn dimension(&self) -> Dimensions {
        self.dim
    }

    /// Access the coordinate of this point.
    pub fn coord(&self) -> Option<&Coord<T>> {
        self.coord.as_ref()
    }

    /// Consume self and return the inner parts.
    pub fn into_inner(self) -> (Option<Coord<T>>, Dimensions) {
        (self.coord, self.dim)
    }

    // Conversion from geo-traits' traits

    /// Create a new point from an object implementing [CoordTrait].
    ///
    /// This infers the dimension from the coordinate.
    pub fn from_coord(coord: impl CoordTrait<T = T>) -> Self {
        Self {
            dim: coord.dim(),
            coord: Some(Coord::new(coord)),
        }
    }

    /// Create a new point from an object implementing [PointTrait].
    ///
    /// This infers the dimension from the coordinate.
    pub fn from_point(point: &impl PointTrait<T = T>) -> Self {
        let dim = point.dim();
        let coord = point.coord().map(|c| Coord::new(c));
        Self { coord, dim }
    }
}

impl<T> From<Point<T>> for super::geometry::Geometry<T>
where
    T: Copy,
{
    fn from(value: Point<T>) -> Self {
        super::geometry::Geometry::Point(value)
    }
}

impl<T: Copy> PointTrait for Point<T> {
    type CoordType<'a>
        = &'a Coord<T>
    where
        Self: 'a;

    fn coord(&self) -> Option<Self::CoordType<'_>> {
        self.coord.as_ref()
    }
}

impl<'a, T: Copy> PointTrait for &'a Point<T> {
    type CoordType<'b>
        = &'a Coord<T>
    where
        Self: 'b;

    fn coord(&self) -> Option<Self::CoordType<'_>> {
        self.coord.as_ref()
    }
}
