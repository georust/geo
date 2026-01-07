//! Monotone Chains are a way of partitioning the segments of a linestring to allow for fast searching of intersections. They have the following properties:
//!
//! 1. the segments within a monotone chain never intersect each other
//! 2. the envelope of any contiguous subset of the segments in a monotone chain is equal to the envelope of the endpoints of the subset.
//!
//! Property 1 means that there is no need to test pairs of segments from within the same monotone chain for intersection.  
//! Property 2 allows an efficient binary search to be used to find the intersection points of two monotone chains. For many types of real-world data, these properties eliminate a large number of segment comparisons, producing substantial speed gains.  
//!
//! This module provides geometries backed by monotone chains.
//!

// primitives
mod segment;
pub(crate) use segment::MonotoneChainSegment;
mod chain;
pub use chain::MonotoneChain;

// geometries
pub(crate) mod geometry;
pub use geometry::{
    MonotoneChainGeometry, MonotoneChainLineString, MonotoneChainMultiLineString,
    MonotoneChainMultiPolygon, MonotoneChainPolygon,
};

// iterators
mod chain_iter;
pub use chain_iter::{MonotoneChainIter, MonotoneChains};

// debug utils
mod util;

use crate::algorithm::dimensions::HasDimensions;
use crate::algorithm::kernels::Kernel;
use crate::coordinate_position::CoordPos;
use crate::intersects::value_in_between;
use crate::{BoundingRect, Coord, GeoNum, Intersects, Orientation};

/// [`Kernel::orient2d`] for [`MonotoneChainSegment`]
///
impl<'a, T: GeoNum> MonotoneChainSegment<'a, T> {
    pub fn orient2d(&self, p: &Coord<T>) -> Orientation {
        if self.ls().len() == 1 {
            return Orientation::Collinear;
        }
        if self.ls().len() == 2 || !self.bounding_rect().intersects(p) {
            return T::Ker::orient2d(
                self.ls()[0],
                *p,
                *self.ls().last().expect("LineString should not be empty"),
            );
        }

        let (a, b) = self.divide();
        debug_assert!(self.ls().len() > 2);
        let b = b.expect("b should not be None");
        // the only way b is None is if self.ls().len() >= 2
        // which has been checked above

        // if intersect neither bbox, then either a or b can be used
        // if both intersect, then collinear (intersects the point)
        // if only one intersects, then take that to recurse

        match (
            a.bounding_rect().intersects(p),
            b.bounding_rect().intersects(p),
        ) {
            (true, true) => Orientation::Collinear,
            (true, false) => a.orient2d(p),
            (false, true) => b.orient2d(p),
            (false, false) => a.orient2d(p), // either works
        }
    }
}

/// Calculate the position of a `Coord` relative to a
/// closed `LineString`.
pub fn coord_pos_relative_to_ring<T>(
    coord: Coord<T>,
    linestring: &MonotoneChainLineString<'_, T>,
) -> CoordPos
where
    T: GeoNum,
{
    debug_assert!(linestring.geometry().is_closed());

    // LineString without points
    if linestring.geometry().is_empty() {
        return CoordPos::Outside;
    }
    if linestring.geometry().0.len() == 1 {
        // If LineString has one point, it will not generate
        // any lines.  So, we handle this edge case separately.
        return if coord == linestring.geometry().0[0] {
            CoordPos::OnBoundary
        } else {
            CoordPos::Outside
        };
    }

    // Use winding number algorithm with on boundary short-cicuit
    // See: https://en.wikipedia.org/wiki/Point_in_polygon#Winding_number_algorithm
    let mut winding_number = 0;
    for line in linestring.chain().segment_iter() {
        // Edge Crossing Rules:
        //   1. an upward edge includes its starting endpoint, and excludes its final endpoint;
        //   2. a downward edge excludes its starting endpoint, and includes its final endpoint;
        //   3. horizontal edges are excluded
        //   4. the edge-ray intersection point must be strictly right of the coord.
        if line.ls()[0].y <= coord.y {
            if line.ls().last().unwrap().y >= coord.y {
                let o = line.orient2d(&coord);
                if o == Orientation::CounterClockwise && line.ls().last().unwrap().y != coord.y {
                    winding_number += 1
                } else if o == Orientation::Collinear
                    && value_in_between(coord.x, line.ls()[0].x, line.ls().last().unwrap().x)
                {
                    return CoordPos::OnBoundary;
                }
            };
        } else if line.ls().last().unwrap().y <= coord.y {
            let o = line.orient2d(&coord);
            if o == Orientation::Clockwise {
                winding_number -= 1
            } else if o == Orientation::Collinear
                && value_in_between(coord.x, line.ls()[0].x, line.ls().last().unwrap().x)
            {
                return CoordPos::OnBoundary;
            }
        }
    }
    if winding_number == 0 {
        CoordPos::Outside
    } else {
        CoordPos::Inside
    }
}
