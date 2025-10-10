use super::{Covers, impl_covers_from_intersects, impl_covers_from_relate};
use crate::geometry::*;
use crate::{GeoFloat, GeoNum};

impl_covers_from_intersects!(Polygon<T>, [Point<T>, MultiPoint<T>]);
impl_covers_from_relate!(Polygon<T>, [
Line<T>,
LineString<T>,  MultiLineString<T>,
Rect<T>, Triangle<T>,
Polygon<T>,  MultiPolygon<T>,
Geometry<T>, GeometryCollection<T>
]);

impl_covers_from_intersects!(MultiPolygon<T>, [Point<T>, MultiPoint<T>]);
impl_covers_from_relate!(MultiPolygon<T>, [
Line<T>,
LineString<T>,  MultiLineString<T>,
Rect<T>, Triangle<T>,
Polygon<T>,  MultiPolygon<T>,
Geometry<T>, GeometryCollection<T>
]);
