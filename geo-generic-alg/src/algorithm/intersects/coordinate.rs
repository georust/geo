use geo_traits_ext::{CoordTag, CoordTraitExt, PointTag, PointTraitExt};

use super::IntersectsTrait;
use crate::*;

impl<T, LHS, RHS> IntersectsTrait<CoordTag, CoordTag, RHS> for LHS
where
    T: CoordNum,
    LHS: CoordTraitExt<T = T>,
    RHS: CoordTraitExt<T = T>,
{
    fn intersects_trait(&self, rhs: &RHS) -> bool {
        self.geo_coord() == rhs.geo_coord()
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
        rhs.geo_coord().is_some_and(|c| self.geo_coord() == c)
    }
}
