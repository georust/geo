use crate::dimension::Dimensions;
use crate::structs::Coord;
use crate::PointTrait;

/// A parsed Point.
#[derive(Clone, Debug, PartialEq)]
pub struct Point<T: Copy> {
    pub(crate) coord: Option<Coord<T>>,
    pub(crate) dim: Dimensions,
}

impl<T: Copy> Point<T> {
    /// Create a new Point from a coordinate and known [Dimension].
    pub fn new(coord: Option<Coord<T>>, dim: Dimensions) -> Self {
        Self { coord, dim }
    }

    /// Create a new point from a valid [Coord].
    ///
    /// This infers the dimension from the coordinate.
    pub fn from_coord(coord: Coord<T>) -> Self {
        Self {
            dim: coord.dimension(),
            coord: Some(coord),
        }
    }

    /// Create a new empty point.
    pub fn empty(dim: Dimensions) -> Self {
        Self::new(None, dim)
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
