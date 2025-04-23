use geo_traits::{to_geo::ToGeoCoord, CoordTrait};
use geo_traits_ext::{CoordTag, CoordTraitExt, LineTag, LineTraitExt, MultiPointTag, MultiPointTraitExt, PointTag, PointTraitExt, RectTag, RectTraitExt, TriangleTag, TriangleTraitExt};

use super::{Intersects, IntersectsTrait};
use crate::*;

impl<T> Intersects<Coord<T>> for Rect<T>
where
    T: CoordNum,
{
    fn intersects(&self, rhs: &Coord<T>) -> bool {
        rhs.x >= self.min().x
            && rhs.y >= self.min().y
            && rhs.x <= self.max().x
            && rhs.y <= self.max().y
    }
}
symmetric_intersects_impl!(Coord<T>, Rect<T>);
symmetric_intersects_impl!(Rect<T>, Point<T>);
symmetric_intersects_impl!(Rect<T>, MultiPoint<T>);

impl<T> Intersects<Rect<T>> for Rect<T>
where
    T: CoordNum,
{
    fn intersects(&self, other: &Rect<T>) -> bool {
        if self.max().x < other.min().x {
            return false;
        }

        if self.max().y < other.min().y {
            return false;
        }

        if self.min().x > other.max().x {
            return false;
        }

        if self.min().y > other.max().y {
            return false;
        }

        true
    }
}

// Same logic as Polygon<T>: Intersects<Line<T>>, but avoid
// an allocation.
impl<T> Intersects<Line<T>> for Rect<T>
where
    T: GeoNum,
{
    fn intersects(&self, rhs: &Line<T>) -> bool {
        let lt = self.min();
        let rb = self.max();
        let lb = Coord::from((lt.x, rb.y));
        let rt = Coord::from((rb.x, lt.y));
        // If either rhs.{start,end} lies inside Rect, then true
        self.intersects(&rhs.start)
            || self.intersects(&rhs.end)
            || Line::new(lt, rt).intersects(rhs)
            || Line::new(rt, rb).intersects(rhs)
            || Line::new(lb, rb).intersects(rhs)
            || Line::new(lt, lb).intersects(rhs)
    }
}
symmetric_intersects_impl!(Line<T>, Rect<T>);

impl<T> Intersects<Triangle<T>> for Rect<T>
where
    T: GeoNum,
{
    fn intersects(&self, rhs: &Triangle<T>) -> bool {
        self.intersects(&rhs.to_polygon())
    }
}
symmetric_intersects_impl!(Triangle<T>, Rect<T>);


///// New Code

impl<T, LHS, RHS> IntersectsTrait<RectTag, CoordTag, RHS> for LHS
where
    T: CoordNum,
    LHS: RectTraitExt<T = T>,
    RHS: CoordTraitExt<T = T>,
{
    fn intersects_trait(&self, rhs: &RHS) -> bool {
        let lhs_x = rhs.x();
        let lhs_y = rhs.y();

        lhs_x >= self.min().x()
            && lhs_y >= self.min().y()
            && lhs_x <= self.max().x()
            && lhs_y <= self.max().y()
    }
}

symmetric_intersects_trait_impl!(CoordNum, CoordTraitExt, CoordTag, RectTraitExt, RectTag);
symmetric_intersects_trait_impl!(CoordNum, RectTraitExt, RectTag, PointTraitExt, PointTag);
symmetric_intersects_trait_impl!(CoordNum, RectTraitExt, RectTag, MultiPointTraitExt, MultiPointTag);

impl<T, LHS, RHS> IntersectsTrait<RectTag, RectTag, RHS> for LHS
where
    T: CoordNum,
    LHS: RectTraitExt<T = T>,
    RHS: RectTraitExt<T = T>,
{
    fn intersects_trait(&self, other: &RHS) -> bool {
        if self.max().x() < other.min().x() {
            return false;
        }

        if self.max().y() < other.min().y() {
            return false;
        }

        if self.min().x() > other.max().x() {
            return false;
        }

        if self.min().y() > other.max().y() {
            return false;
        }

        true
    }
}

// Same logic as polygon x line, but avoid an allocation.
impl<T, LHS, RHS> IntersectsTrait<RectTag, LineTag, RHS> for LHS
where
    T: GeoNum,
    LHS: RectTraitExt<T = T>,
    RHS: LineTraitExt<T = T>,
{
    fn intersects_trait(&self, rhs: &RHS) -> bool {
        let lt = self.min().to_coord();
        let rb = self.max().to_coord();
        let lb = Coord::from((lt.x, rb.y));
        let rt = Coord::from((rb.x, lt.y));

        // If either rhs.{start,end} lies inside Rect, then true
        self.intersects_trait(&rhs.start_ext())
            || self.intersects_trait(&rhs.end_ext())
            || Line::new(lt, rt).intersects_trait(rhs)
            || Line::new(rt, rb).intersects_trait(rhs)
            || Line::new(lb, rb).intersects_trait(rhs)
            || Line::new(lt, lb).intersects_trait(rhs)
    }
}

symmetric_intersects_trait_impl!(GeoNum, LineTraitExt, LineTag, RectTraitExt, RectTag);

impl<T, LHS, RHS> IntersectsTrait<RectTag, TriangleTag, RHS> for LHS
where
    T: CoordNum,
    LHS: RectTraitExt<T = T>,
    RHS: TriangleTraitExt<T = T>,
{
    fn intersects_trait(&self, _other: &RHS) -> bool {
        // TODO: Once we have rect x polygon implemented, we can uncomment this
        // self.intersects_trait(&other.to_polygon())
        false
    }
}

symmetric_intersects_trait_impl!(CoordNum, TriangleTraitExt, TriangleTag, RectTraitExt, RectTag);
