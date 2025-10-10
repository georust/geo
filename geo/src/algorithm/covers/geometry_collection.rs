use super::{Covers, impl_covers_from_relate, impl_covers_geometry_for};
use crate::covers::impl_covers_from_intersects;
use crate::geometry::*;
use crate::{GeoFloat, GeoNum};

impl_covers_from_intersects!(GeometryCollection<T>,[Point<T>,MultiPoint<T>]);
impl_covers_from_relate!(GeometryCollection<T>, [
Line<T>,
LineString<T>, MultiLineString<T>,
Rect<T>, Triangle<T>,
Polygon<T>, MultiPolygon<T>,
GeometryCollection<T>
]);
impl_covers_geometry_for!(GeometryCollection<T>);
