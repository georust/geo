use crate::geometry::*;
// use crate::relate::Relate;
use super::{ContainsProperly, impl_contains_properly_from_relate};
use crate::GeoFloat;

impl_contains_properly_from_relate!(GeometryCollection<T>, [Point<T>,Line<T>, LineString<T>, Polygon<T>, MultiPoint<T>, MultiLineString<T>, MultiPolygon<T>, GeometryCollection<T>, Rect<T>, Triangle<T>]);
