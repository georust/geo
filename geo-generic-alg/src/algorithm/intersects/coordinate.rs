use geo_traits_ext::{CoordTag, CoordTraitExt, PointTag, PointTraitExt};
use geo_traits::to_geo::ToGeoCoord;

use super::{IntersectsTrait, Intersects};
use crate::*;

impl<T> Intersects<Coord<T>> for Coord<T>
where
    T: CoordNum,
{
    fn intersects(&self, rhs: &Coord<T>) -> bool {
        self == rhs
    }
}

// The other side of this is handled via a blanket impl.
impl<T> Intersects<Point<T>> for Coord<T>
where
    T: CoordNum,
{
    fn intersects(&self, rhs: &Point<T>) -> bool {
        self == &rhs.0
    }
}

///// New Code

impl<T, LHS, RHS> IntersectsTrait<CoordTag, CoordTag, RHS> for LHS
where
    T: CoordNum,
    LHS: CoordTraitExt<T = T>,
    RHS: CoordTraitExt<T = T>,
{
    fn intersects_trait(&self, rhs: &RHS) -> bool {
        self.to_coord() == rhs.to_coord()
    }
}

// The other side of this is handled via a blanket impl.
impl<T, LHS, RHS> IntersectsTrait<CoordTag, PointTag, RHS> for LHS
where
    T: CoordNum,
    LHS: CoordTraitExt<T = T>,
    RHS: PointTraitExt<T = T>,
{
    fn intersects_trait(&self, rhs: &RHS) -> bool {
        rhs.coord().is_some_and(|c| self.to_coord() == c.to_coord())
    }
}
