use super::{Contains, impl_contains_from_relate, impl_contains_geometry_for};
use crate::GeoFloat;
use crate::geometry::*;
use crate::geometry_cow::GeometryCow;

impl<T> Contains<Coord<T>> for GeometryCollection<T>
where
    T: GeoFloat,
{
    /// Point containment uses the union semantics of the collection, as
    /// `relate` does: a point on the shared boundary of two adjacent
    /// polygon elements lies in the interior of their union and is
    /// contained. A member-wise check cannot express this, so the
    /// evaluation goes through the relate engine.
    fn contains(&self, coord: &Coord<T>) -> bool {
        let point = Point::from(*coord);
        crate::algorithm::relate::relateng::relate_ng::eval(
            &GeometryCow::from(self),
            &GeometryCow::from(&point),
            &mut crate::algorithm::relate::relateng::relate_predicate::contains(),
        )
    }
}

impl<T> Contains<Point<T>> for GeometryCollection<T>
where
    T: GeoFloat,
{
    fn contains(&self, point: &Point<T>) -> bool {
        self.contains(&point.0)
    }
}

impl_contains_from_relate!(GeometryCollection<T>, [Line<T>, LineString<T>, Polygon<T>, MultiPoint<T>, MultiLineString<T>, MultiPolygon<T>, GeometryCollection<T>, Rect<T>, Triangle<T>]);
impl_contains_geometry_for!(GeometryCollection<T>);
