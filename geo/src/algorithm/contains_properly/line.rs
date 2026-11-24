use super::{ContainsProperly, value_in_range_exclusive};
use crate::algorithm::kernels::Kernel;
use crate::geometry::*;
use crate::{BoundingRect, CoordsIter, GeoNum, HasDimensions, Orientation};

impl<G, T> ContainsProperly<G> for Line<T>
where
    T: GeoNum,
    G: CoordsIter<Scalar = T> + HasDimensions + BoundingRect<T>,
{
    fn contains_properly(&self, rhs: &G) -> bool {
        if HasDimensions::is_empty(rhs) {
            return false;
        }

        // orient the bounds once
        let x_bound = if self.start.x < self.end.x {
            (self.start.x, self.end.x)
        } else {
            (self.end.x, self.start.x)
        };
        let y_bound = if self.start.y < self.end.y {
            (self.start.y, self.end.y)
        } else {
            (self.end.y, self.start.y)
        };

        rhs.coords_iter().all(|c| {
            value_in_range_exclusive(c.x, x_bound.0, x_bound.1)
                && value_in_range_exclusive(c.y, y_bound.0, y_bound.1)
                && T::Ker::orient2d(self.start, self.end, c) == Orientation::Collinear
        })
    }
}
