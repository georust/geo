use geo_traits_ext::*;

use super::IntersectsTrait;
use crate::*;

// Generate implementations for Point<T> by delegating to Coord<T>
macro_rules! impl_intersects_point_from_coord {
    ($num_type:ident, $rhs_type:ident, $rhs_tag:ident) => {
        impl<T, LHS, RHS> IntersectsTrait<PointTag, $rhs_tag, RHS> for LHS
        where
            T: $num_type,
            LHS: PointTraitExt<T = T>,
            RHS: $rhs_type<T = T>,
        {
            fn intersects_trait(&self, rhs: &RHS) -> bool {
                self.coord_ext().is_some_and(|c| c.intersects_trait(rhs))
            }
        }
    };
}

impl_intersects_point_from_coord!(CoordNum, CoordTraitExt, CoordTag);
impl_intersects_point_from_coord!(CoordNum, PointTraitExt, PointTag);
impl_intersects_point_from_coord!(GeoNum, LineStringTraitExt, LineStringTag);
impl_intersects_point_from_coord!(GeoNum, PolygonTraitExt, PolygonTag);
impl_intersects_point_from_coord!(CoordNum, MultiPointTraitExt, MultiPointTag);
impl_intersects_point_from_coord!(GeoNum, MultiLineStringTraitExt, MultiLineStringTag);
impl_intersects_point_from_coord!(GeoNum, MultiPolygonTraitExt, MultiPolygonTag);
impl_intersects_point_from_coord!(GeoNum, GeometryTraitExt, GeometryTag);
impl_intersects_point_from_coord!(GeoNum, GeometryCollectionTraitExt, GeometryCollectionTag);
impl_intersects_point_from_coord!(GeoNum, LineTraitExt, LineTag);
impl_intersects_point_from_coord!(CoordNum, RectTraitExt, RectTag);
impl_intersects_point_from_coord!(GeoNum, TriangleTraitExt, TriangleTag);

// Generate implementations for MultiPoint<T> by delegating to Point<T>
macro_rules! impl_intersects_multipoint_from_point {
    ($num_type:ident, $rhs_type:ident, $rhs_tag:ident) => {
        impl<T, LHS, RHS> IntersectsTrait<MultiPointTag, $rhs_tag, RHS> for LHS
        where
            T: $num_type,
            LHS: MultiPointTraitExt<T = T>,
            RHS: $rhs_type<T = T>,
        {
            fn intersects_trait(&self, rhs: &RHS) -> bool {
                self.points_ext().any(|p| p.intersects_trait(rhs))
            }
        }
    };
}

impl_intersects_multipoint_from_point!(CoordNum, CoordTraitExt, CoordTag);
impl_intersects_multipoint_from_point!(CoordNum, PointTraitExt, PointTag);
impl_intersects_multipoint_from_point!(GeoNum, LineStringTraitExt, LineStringTag);
impl_intersects_multipoint_from_point!(GeoNum, PolygonTraitExt, PolygonTag);
impl_intersects_multipoint_from_point!(CoordNum, MultiPointTraitExt, MultiPointTag);
impl_intersects_multipoint_from_point!(GeoNum, MultiLineStringTraitExt, MultiLineStringTag);
impl_intersects_multipoint_from_point!(GeoNum, MultiPolygonTraitExt, MultiPolygonTag);
impl_intersects_multipoint_from_point!(GeoNum, GeometryTraitExt, GeometryTag);
impl_intersects_multipoint_from_point!(GeoNum, GeometryCollectionTraitExt, GeometryCollectionTag);
impl_intersects_multipoint_from_point!(GeoNum, LineTraitExt, LineTag);
impl_intersects_multipoint_from_point!(CoordNum, RectTraitExt, RectTag);
impl_intersects_multipoint_from_point!(GeoNum, TriangleTraitExt, TriangleTag);

symmetric_intersects_trait_impl!(
    CoordNum,
    CoordTraitExt,
    CoordTag,
    MultiPointTraitExt,
    MultiPointTag
);
symmetric_intersects_trait_impl!(
    GeoNum,
    LineTraitExt,
    LineTag,
    MultiPointTraitExt,
    MultiPointTag
);
symmetric_intersects_trait_impl!(
    GeoNum,
    TriangleTraitExt,
    TriangleTag,
    MultiPointTraitExt,
    MultiPointTag
);
symmetric_intersects_trait_impl!(
    GeoNum,
    PolygonTraitExt,
    PolygonTag,
    MultiPointTraitExt,
    MultiPointTag
);
