use super::{has_disjoint_bboxes, IntersectsTrait};
use crate::coordinate_position::CoordPos;
use crate::CoordinatePosition;
use crate::GeoNum;
use geo_traits::to_geo::ToGeoCoord;
use geo_traits_ext::*;

impl<T, LHS, RHS> IntersectsTrait<PolygonTag, CoordTag, RHS> for LHS
where
    T: GeoNum,
    LHS: PolygonTraitExt<T = T>,
    RHS: CoordTraitExt<T = T>,
{
    fn intersects_trait(&self, rhs: &RHS) -> bool {
        self.coordinate_position(&rhs.to_coord()) != CoordPos::Outside
    }
}

symmetric_intersects_trait_impl!(GeoNum, CoordTraitExt, CoordTag, PolygonTraitExt, PolygonTag);
symmetric_intersects_trait_impl!(GeoNum, PolygonTraitExt, PolygonTag, PointTraitExt, PointTag);

impl<T, LHS, RHS> IntersectsTrait<PolygonTag, LineTag, RHS> for LHS
where
    T: GeoNum,
    LHS: PolygonTraitExt<T = T>,
    RHS: LineTraitExt<T = T>,
{
    fn intersects_trait(&self, line: &RHS) -> bool {
        // Check if line intersects any part of the polygon
        if let Some(exterior) = self.exterior_ext() {
            exterior.intersects_trait(line)
                || self
                    .interiors_ext()
                    .any(|inner| inner.intersects_trait(line))
                || self.intersects_trait(&line.start_ext())
                || self.intersects_trait(&line.end_ext())
        } else {
            false
        }
    }
}

symmetric_intersects_trait_impl!(GeoNum, LineTraitExt, LineTag, PolygonTraitExt, PolygonTag);
symmetric_intersects_trait_impl!(
    GeoNum,
    PolygonTraitExt,
    PolygonTag,
    LineStringTraitExt,
    LineStringTag
);
symmetric_intersects_trait_impl!(
    GeoNum,
    PolygonTraitExt,
    PolygonTag,
    MultiLineStringTraitExt,
    MultiLineStringTag
);

impl<T, LHS, RHS> IntersectsTrait<PolygonTag, RectTag, RHS> for LHS
where
    T: GeoNum,
    LHS: PolygonTraitExt<T = T>,
    RHS: RectTraitExt<T = T>,
{
    fn intersects_trait(&self, rhs: &RHS) -> bool {
        self.intersects_trait(&rhs.to_polygon())
    }
}

symmetric_intersects_trait_impl!(GeoNum, RectTraitExt, RectTag, PolygonTraitExt, PolygonTag);

impl<T, LHS, RHS> IntersectsTrait<PolygonTag, TriangleTag, RHS> for LHS
where
    T: GeoNum,
    LHS: PolygonTraitExt<T = T>,
    RHS: TriangleTraitExt<T = T>,
{
    fn intersects_trait(&self, rhs: &RHS) -> bool {
        self.intersects_trait(&rhs.to_polygon())
    }
}

symmetric_intersects_trait_impl!(
    GeoNum,
    TriangleTraitExt,
    TriangleTag,
    PolygonTraitExt,
    PolygonTag
);

impl<T, LHS, RHS> IntersectsTrait<PolygonTag, PolygonTag, RHS> for LHS
where
    T: GeoNum,
    LHS: PolygonTraitExt<T = T>,
    RHS: PolygonTraitExt<T = T>,
{
    fn intersects_trait(&self, polygon: &RHS) -> bool {
        if has_disjoint_bboxes(self, polygon) {
            return false;
        }

        if let (Some(self_exterior), Some(polygon_exterior)) =
            (self.exterior_ext(), polygon.exterior_ext())
        {
            // self intersects (or contains) any line in polygon
            self.intersects_trait(&polygon_exterior) ||
            polygon.interiors_ext().any(|inner_line_string| self.intersects_trait(&inner_line_string)) ||
            // self is contained inside polygon
            polygon.intersects_trait(&self_exterior)
        } else {
            false
        }
    }
}

// Generate implementations for MultiPolygon by delegating to the Polygon implementation

macro_rules! impl_intersects_multi_polygon_from_polygon {
    ($rhs_type:ident, $rhs_tag:ident) => {
        impl<T, LHS, RHS> IntersectsTrait<MultiPolygonTag, $rhs_tag, RHS> for LHS
        where
            T: GeoNum,
            LHS: MultiPolygonTraitExt<T = T>,
            RHS: $rhs_type<T = T>,
        {
            fn intersects_trait(&self, rhs: &RHS) -> bool {
                if has_disjoint_bboxes(self, rhs) {
                    return false;
                }
                self.polygons_ext().any(|p| p.intersects_trait(rhs))
            }
        }
    };
}

impl_intersects_multi_polygon_from_polygon!(CoordTraitExt, CoordTag);
impl_intersects_multi_polygon_from_polygon!(PointTraitExt, PointTag);
impl_intersects_multi_polygon_from_polygon!(LineStringTraitExt, LineStringTag);
impl_intersects_multi_polygon_from_polygon!(PolygonTraitExt, PolygonTag);
impl_intersects_multi_polygon_from_polygon!(MultiPointTraitExt, MultiPointTag);
impl_intersects_multi_polygon_from_polygon!(MultiLineStringTraitExt, MultiLineStringTag);
impl_intersects_multi_polygon_from_polygon!(MultiPolygonTraitExt, MultiPolygonTag);
impl_intersects_multi_polygon_from_polygon!(GeometryTraitExt, GeometryTag);
impl_intersects_multi_polygon_from_polygon!(GeometryCollectionTraitExt, GeometryCollectionTag);
impl_intersects_multi_polygon_from_polygon!(LineTraitExt, LineTag);
impl_intersects_multi_polygon_from_polygon!(RectTraitExt, RectTag);
impl_intersects_multi_polygon_from_polygon!(TriangleTraitExt, TriangleTag);

symmetric_intersects_trait_impl!(
    GeoNum,
    CoordTraitExt,
    CoordTag,
    MultiPolygonTraitExt,
    MultiPolygonTag
);
symmetric_intersects_trait_impl!(
    GeoNum,
    LineTraitExt,
    LineTag,
    MultiPolygonTraitExt,
    MultiPolygonTag
);
symmetric_intersects_trait_impl!(
    GeoNum,
    RectTraitExt,
    RectTag,
    MultiPolygonTraitExt,
    MultiPolygonTag
);
symmetric_intersects_trait_impl!(
    GeoNum,
    TriangleTraitExt,
    TriangleTag,
    MultiPolygonTraitExt,
    MultiPolygonTag
);
symmetric_intersects_trait_impl!(
    GeoNum,
    PolygonTraitExt,
    PolygonTag,
    MultiPolygonTraitExt,
    MultiPolygonTag
);

#[cfg(test)]
mod tests {
    use crate::*;
    #[test]
    fn geom_intersects_geom() {
        let a = Geometry::<f64>::from(polygon![]);
        let b = Geometry::from(polygon![]);
        assert!(!a.intersects(&b));
    }
}
