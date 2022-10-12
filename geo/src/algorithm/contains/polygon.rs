use super::{impl_contains_from_relate, impl_contains_geometry_for, Contains};
use crate::geometry::*;
use crate::Relate;
use crate::{GeoFloat, GeoNum};

// ┌─────────────────────────────┐
// │ Implementations for Polygon │
// └─────────────────────────────┘
impl<T> Contains<Coordinate<T>> for Polygon<T>
where
    T: GeoNum,
{
    fn contains(&self, coord: &Coordinate<T>) -> bool {
        use crate::coordinate_position::{CoordPos, CoordinatePosition};

        self.coordinate_position(coord) == CoordPos::Inside
    }
}

impl<T> Contains<Point<T>> for Polygon<T>
where
    T: GeoNum,
{
    fn contains(&self, p: &Point<T>) -> bool {
        self.contains(&p.0)
    }
}

impl_contains_from_relate!(Polygon<T>, [Line<T>, LineString<T>, Polygon<T>, MultiPoint<T>, MultiLineString<T>, MultiPolygon<T>, GeometryCollection<T>, Rect<T>, Triangle<T>]);
impl_contains_geometry_for!(Polygon<T>);

// ┌──────────────────────────────────┐
// │ Implementations for MultiPolygon │
// └──────────────────────────────────┘

impl<T> Contains<Coordinate<T>> for MultiPolygon<T>
where
    T: GeoNum,
{
    fn contains(&self, coord: &Coordinate<T>) -> bool {
        self.iter().any(|poly| poly.contains(coord))
    }
}

impl<T> Contains<Point<T>> for MultiPolygon<T>
where
    T: GeoNum,
{
    fn contains(&self, p: &Point<T>) -> bool {
        self.contains(&p.0)
    }
}

impl<T: GeoNum> Contains<MultiPoint<T>> for MultiPolygon<T> {
    fn contains(&self, rhs: &MultiPoint<T>) -> bool {
        rhs.iter().all(|point| self.contains(point))
    }
}

impl<F> Contains<Line<F>> for MultiPolygon<F>
where
    F: GeoFloat,
{
    fn contains(&self, rhs: &Line<F>) -> bool {
        rhs.relate(self).is_within()
    }
}

impl<F> Contains<LineString<F>> for MultiPolygon<F>
where
    F: GeoFloat,
{
    fn contains(&self, rhs: &LineString<F>) -> bool {
        rhs.relate(self).is_within()
    }
}

impl<F> Contains<MultiLineString<F>> for MultiPolygon<F>
where
    F: GeoFloat,
{
    fn contains(&self, rhs: &MultiLineString<F>) -> bool {
        rhs.relate(self).is_within()
    }
}

impl<F> Contains<Polygon<F>> for MultiPolygon<F>
where
    F: GeoFloat,
{
    fn contains(&self, rhs: &Polygon<F>) -> bool {
        rhs.relate(self).is_within()
    }
}

impl<F> Contains<MultiPolygon<F>> for MultiPolygon<F>
where
    F: GeoFloat,
{
    fn contains(&self, rhs: &MultiPolygon<F>) -> bool {
        rhs.relate(self).is_within()
    }
}

impl<F> Contains<GeometryCollection<F>> for MultiPolygon<F>
where
    F: GeoFloat,
{
    fn contains(&self, rhs: &GeometryCollection<F>) -> bool {
        rhs.relate(self).is_within()
    }
}

impl<F> Contains<Rect<F>> for MultiPolygon<F>
where
    F: GeoFloat,
{
    fn contains(&self, rhs: &Rect<F>) -> bool {
        rhs.relate(self).is_within()
    }
}

impl<F> Contains<Triangle<F>> for MultiPolygon<F>
where
    F: GeoFloat,
{
    fn contains(&self, rhs: &Triangle<F>) -> bool {
        rhs.relate(self).is_within()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn gh_issue_921() {
        use crate::algorithm::{Contains, Intersects};
        use crate::geometry::{Point, Polygon};
        use crate::{point, polygon};

        // > Just wanted to note that the contains functions shows some really weird behaviour for i32 types close to i32 max.
        // > crashes
        let poly_32: Polygon<i32> = polygon![(x: 107374182, y: 107374182), (x: 207374182, y: 107374182) , (x: 207374182, y: 207374182) , (x: 107374182, y: 207374182)];
        let point: Point<i32> = point!(x: 107374182, y: 107374182);

        // Note that polygons DO NOT contain points that are on their boundary - otherwise it would be a synonym for `intersects`.
        // For the point to be `contained` it must be within the polygons interior.
        // See: http://lin-ear-th-inking.blogspot.be/2007/06/subtleties-of-ogc-covers-spatial.html
        assert!(!poly_32.contains(&point));

        // they do intersect though
        assert!(poly_32.intersects(&point));
    }
}
