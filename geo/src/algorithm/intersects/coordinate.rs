use super::Intersects;
use crate::*;

impl<T> Intersects<Coord<T>> for Coord<T>
where
    T: CoordNum,
{
    fn intersects(&self, rhs: &Coord<T>) -> bool {
        self == rhs
    }
}

symmetric_intersects_impl!(Coord<T>, LineString<T>);
symmetric_intersects_impl!(Coord<T>, MultiLineString<T>);

symmetric_intersects_impl!(Coord<T>, Line<T>);

// The other side of this is handled via a blanket impl.
impl<T> Intersects<Point<T>> for Coord<T>
where
    T: CoordNum,
{
    fn intersects(&self, rhs: &Point<T>) -> bool {
        self == &rhs.0
    }
}
symmetric_intersects_impl!(Coord<T>, MultiPoint<T>);

symmetric_intersects_impl!(Coord<T>, Polygon<T>);
symmetric_intersects_impl!(Coord<T>, MultiPolygon<T>);

symmetric_intersects_impl!(Coord<T>, Rect<T>);

symmetric_intersects_impl!(Coord<T>, Triangle<T>);
