use super::{Covers, impl_covers_from_intersects};
use crate::GeoNum;
use crate::geometry::*;

impl_covers_from_intersects!(Triangle<T>, [
Point<T>,MultiPoint<T>,
Line<T>,
LineString<T>, MultiLineString<T>,
Rect<T>, Triangle<T>,
Polygon<T>,  MultiPolygon<T>,
Geometry<T>, GeometryCollection<T>
]);
