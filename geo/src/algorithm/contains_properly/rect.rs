use super::{ContainsProperly, impl_contains_properly_from_relate};
use crate::GeoFloat;
use crate::geometry::*;

impl_contains_properly_from_relate!(Rect<T>, [
Point<T>,MultiPoint<T>,
Line<T>, LineString<T>, MultiLineString<T>,
Polygon<T>,MultiPolygon<T>,
GeometryCollection<T>,
Rect<T>,Triangle<T>
]);
