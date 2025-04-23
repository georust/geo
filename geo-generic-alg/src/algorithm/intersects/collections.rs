use geo_traits_ext::*;

use super::has_disjoint_bboxes;
use super::IntersectsTrait;
use crate::geometry_trait_ext_delegate_impl;
use crate::GeoNum;

macro_rules! impl_intersects_geometry {
    ($rhs_type:ident, $rhs_tag:ident) => {
        impl<T, LHS, RHS> IntersectsTrait<GeometryTag, $rhs_tag, RHS> for LHS
        where
            T: GeoNum,
            LHS: GeometryTraitExt<T = T>,
            RHS: $rhs_type<T = T>,
        {
            geometry_trait_ext_delegate_impl! {
                fn intersects_trait(&self, rhs: &RHS) -> bool;
            }
        }
    };
}

impl_intersects_geometry!(CoordTraitExt, CoordTag);
impl_intersects_geometry!(PointTraitExt, PointTag);
impl_intersects_geometry!(LineStringTraitExt, LineStringTag);
impl_intersects_geometry!(PolygonTraitExt, PolygonTag);
impl_intersects_geometry!(MultiPointTraitExt, MultiPointTag);
impl_intersects_geometry!(MultiLineStringTraitExt, MultiLineStringTag);
impl_intersects_geometry!(MultiPolygonTraitExt, MultiPolygonTag);
impl_intersects_geometry!(GeometryTraitExt, GeometryTag);
impl_intersects_geometry!(GeometryCollectionTraitExt, GeometryCollectionTag);
impl_intersects_geometry!(LineTraitExt, LineTag);
impl_intersects_geometry!(RectTraitExt, RectTag);
impl_intersects_geometry!(TriangleTraitExt, TriangleTag);

symmetric_intersects_trait_impl!(
    GeoNum,
    CoordTraitExt,
    CoordTag,
    GeometryTraitExt,
    GeometryTag
);
symmetric_intersects_trait_impl!(GeoNum, LineTraitExt, LineTag, GeometryTraitExt, GeometryTag);
symmetric_intersects_trait_impl!(GeoNum, RectTraitExt, RectTag, GeometryTraitExt, GeometryTag);
symmetric_intersects_trait_impl!(
    GeoNum,
    TriangleTraitExt,
    TriangleTag,
    GeometryTraitExt,
    GeometryTag
);
symmetric_intersects_trait_impl!(
    GeoNum,
    PolygonTraitExt,
    PolygonTag,
    GeometryTraitExt,
    GeometryTag
);

// Generate implementations for GeometryCollection by delegating to the Geometry implementation
macro_rules! impl_intersects_geometry_collection_from_geometry {
    ($rhs_type:ident, $rhs_tag:ident) => {
        impl<T, LHS, RHS> IntersectsTrait<GeometryCollectionTag, $rhs_tag, RHS> for LHS
        where
            T: GeoNum,
            LHS: GeometryCollectionTraitExt<T = T>,
            RHS: $rhs_type<T = T>,
        {
            fn intersects_trait(&self, rhs: &RHS) -> bool {
                if has_disjoint_bboxes(self, rhs) {
                    return false;
                }
                self.geometries_ext().any(|geom| geom.intersects_trait(rhs))
            }
        }
    };
}

impl_intersects_geometry_collection_from_geometry!(CoordTraitExt, CoordTag);
impl_intersects_geometry_collection_from_geometry!(PointTraitExt, PointTag);
impl_intersects_geometry_collection_from_geometry!(LineStringTraitExt, LineStringTag);
impl_intersects_geometry_collection_from_geometry!(PolygonTraitExt, PolygonTag);
impl_intersects_geometry_collection_from_geometry!(MultiPointTraitExt, MultiPointTag);
impl_intersects_geometry_collection_from_geometry!(MultiLineStringTraitExt, MultiLineStringTag);
impl_intersects_geometry_collection_from_geometry!(MultiPolygonTraitExt, MultiPolygonTag);
impl_intersects_geometry_collection_from_geometry!(GeometryTraitExt, GeometryTag);
impl_intersects_geometry_collection_from_geometry!(
    GeometryCollectionTraitExt,
    GeometryCollectionTag
);
impl_intersects_geometry_collection_from_geometry!(LineTraitExt, LineTag);
impl_intersects_geometry_collection_from_geometry!(RectTraitExt, RectTag);
impl_intersects_geometry_collection_from_geometry!(TriangleTraitExt, TriangleTag);

symmetric_intersects_trait_impl!(
    GeoNum,
    CoordTraitExt,
    CoordTag,
    GeometryCollectionTraitExt,
    GeometryCollectionTag
);
symmetric_intersects_trait_impl!(
    GeoNum,
    LineTraitExt,
    LineTag,
    GeometryCollectionTraitExt,
    GeometryCollectionTag
);
symmetric_intersects_trait_impl!(
    GeoNum,
    RectTraitExt,
    RectTag,
    GeometryCollectionTraitExt,
    GeometryCollectionTag
);
symmetric_intersects_trait_impl!(
    GeoNum,
    TriangleTraitExt,
    TriangleTag,
    GeometryCollectionTraitExt,
    GeometryCollectionTag
);
symmetric_intersects_trait_impl!(
    GeoNum,
    PolygonTraitExt,
    PolygonTag,
    GeometryCollectionTraitExt,
    GeometryCollectionTag
);
