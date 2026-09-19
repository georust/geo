use std::{cmp::Ordering, ops::ControlFlow};

use geo_types::{Coord, Line, LineString, MultiPolygon, Polygon};
use sif_itree::ITree;

use crate::algorithm::kernels::Kernel;
use crate::coordinate_position::CoordPos;
use crate::intersects::value_in_between;
use crate::{GeoNum, LinesIter, Orientation};

/// A Y-coordinate wrapper that implements `Ord` for use in the interval tree.
/// Unlike a full point comparison, this only compares Y values, avoiding
/// unnecessary X comparisons since we know X is always zero in our usage.
#[derive(PartialEq, Clone, Copy)]
struct YValue<T: GeoNum>(T);

impl<T: GeoNum> std::fmt::Debug for YValue<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Y").field(&self.0).finish()
    }
}

impl<T: GeoNum> PartialOrd for YValue<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Direct Y-value comparison, no X coordinate overhead
impl<T: GeoNum> Ord for YValue<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl<T: GeoNum> Eq for YValue<T> {}

impl<T: GeoNum> From<T> for YValue<T> {
    fn from(y: T) -> Self {
        YValue(y)
    }
}

struct YIntervalSegment<T: GeoNum> {
    segment: (Coord<T>, Coord<T>),
    /// Pre-computed maximum x-coordinate of the segment endpoints.
    /// Used for early rejection of segments that don't extend far enough right.
    /// This avoids computing max(seg.0.x, seg.1.x) for every point query.
    x_max: T,
}

/// A [`MultiPolygon`] backed by an [interval tree] for fast point-in-polygon containment queries.
///
/// Use this when you need to test many points against the same fixed (Multi)Polygon — for example,
/// classifying a large point cloud against administrative boundaries, or repeatedly probing a
/// single complex polygon. The index is built once from a [`MultiPolygon`] and amortises its
/// construction cost across subsequent queries; for one-off checks, the plain
/// [`Contains`](crate::Contains) implementation on `MultiPolygon` is preferable.
///
/// Internally, every line segment from each ring (exterior and interior) is stored in an interval
/// tree keyed by its y-extent, together with its precomputed maximum x. A point query then visits
/// only those segments whose y-interval covers the query point's y, applies an x-based early
/// rejection, and runs a robust winding-number test on the survivors. This avoids the linear scan
/// over every segment that a naive ray cast would perform.
///
/// Queries are exposed via the [`Contains`](crate::Contains) and
/// [`Intersects`](crate::Intersects) traits for both [`Coord`] and [`Point`](crate::Point), and
/// follow the same semantics as their `MultiPolygon` counterparts: `contains` is true only for
/// points strictly in the interior, whereas `intersects` is also true for points on the boundary.
///
/// # Example
///
/// ```
/// use geo::indexed::IntervalTreeMultiPolygon;
/// use geo::{Contains, Intersects, MultiPolygon, Point, wkt};
///
/// // A square with a square hole.
/// let mp: MultiPolygon = wkt!(
///     MULTIPOLYGON(((0.0 0.0, 4.0 0.0, 4.0 4.0, 0.0 4.0, 0.0 0.0),
///                   (1.0 1.0, 1.0 3.0, 3.0 3.0, 3.0 1.0, 1.0 1.0)))
/// );
///
/// // Build the index once, then reuse it across many queries.
/// let indexed = IntervalTreeMultiPolygon::new(&mp);
///
/// assert!(indexed.contains(&Point::new(0.5, 0.5))); // inside the shell
/// assert!(!indexed.contains(&Point::new(2.0, 2.0))); // inside the hole
/// assert!(!indexed.contains(&Point::new(5.0, 5.0))); // outside
///
/// // Unlike `contains`, `intersects` includes the boundary.
/// assert!(indexed.intersects(&Point::new(0.0, 2.0))); // on the shell
/// assert!(indexed.intersects(&Point::new(1.0, 2.0))); // on the hole's edge
/// assert!(!indexed.contains(&Point::new(0.0, 2.0)));
/// ```
///
/// [interval tree]: https://en.wikipedia.org/wiki/Interval_tree
pub struct IntervalTreeMultiPolygon<T: GeoNum> {
    y_interval_tree: ITree<YValue<T>, YIntervalSegment<T>>,
}

impl<T: GeoNum> IntervalTreeMultiPolygon<T> {
    pub fn new(mp: &MultiPolygon<T>) -> Self {
        Self::from_rings(mp.0.iter().flat_map(polygon_rings))
    }

    /// Builds the index for a single [`Polygon`].
    // Consumed by the RelateNG port (see RELATENG_PLAN.md); remove the allow
    // when it lands.
    #[allow(dead_code)]
    pub(crate) fn from_polygon(polygon: &Polygon<T>) -> Self {
        Self::from_rings(polygon_rings(polygon))
    }

    /// Builds the index from an iterator of rings (exterior and interior line
    /// segments are treated alike).
    pub(crate) fn from_rings<'a>(rings: impl Iterator<Item = &'a LineString<T>>) -> Self
    where
        T: 'a,
    {
        let segments = rings
            .flat_map(|ring| ring.lines_iter())
            .map(Self::create_segment);

        Self {
            y_interval_tree: ITree::new(segments),
        }
    }

    fn create_segment(line: Line<T>) -> sif_itree::Item<YValue<T>, YIntervalSegment<T>> {
        let p1 = line.start;
        let p2 = line.end;

        let (y_min_val, y_max_val) = if p1.y.total_cmp(&p2.y).is_le() {
            (p1.y, p2.y)
        } else {
            (p2.y, p1.y)
        };
        let y_min = YValue::from(y_min_val);
        let y_max = YValue::from(y_max_val);

        // Pre-compute maximum x for early rejection during queries
        let x_max = if p1.x.total_cmp(&p2.x).is_ge() {
            p1.x
        } else {
            p2.x
        };

        (
            y_min..y_max,
            YIntervalSegment {
                segment: (p1, p2),
                x_max,
            },
        )
    }

    /// Locates a coordinate with nonzero-winding semantics.
    ///
    /// Shells and holes must be oppositely wound for a hole to read as
    /// `Outside`; this matches the existing `Contains` behaviour built on
    /// this method. `OnBoundary` detection is exact.
    pub(crate) fn containment(&self, coord: Coord<T>) -> CoordPos {
        match self.winding_number(coord) {
            ControlFlow::Break(pos) => pos,
            ControlFlow::Continue(winding_number) => {
                if winding_number != 0 {
                    CoordPos::Inside
                } else {
                    CoordPos::Outside
                }
            }
        }
    }

    /// Locates a coordinate with even-odd (parity) semantics, as JTS's
    /// `RayCrossingCounter` does.
    ///
    /// Unlike [`Self::containment`], the result does not depend on ring
    /// orientation, so invalid input (a hole wound the same way as its
    /// shell) still locates points in the hole as `Outside`. The two methods
    /// agree on all valid input. `OnBoundary` detection is exact.
    // Consumed by the RelateNG port (see RELATENG_PLAN.md); remove the allow
    // when it lands.
    #[allow(dead_code)]
    pub(crate) fn containment_parity(&self, coord: Coord<T>) -> CoordPos {
        // Each ray crossing contributes +1 or -1 to the winding number, so
        // the parity of the winding sum equals the parity of the crossing
        // count.
        match self.winding_number(coord) {
            ControlFlow::Break(pos) => pos,
            ControlFlow::Continue(winding_number) => {
                if winding_number % 2 != 0 {
                    CoordPos::Inside
                } else {
                    CoordPos::Outside
                }
            }
        }
    }

    /// Shared traversal: breaks with `OnBoundary` when the coordinate lies
    /// exactly on a segment, otherwise yields the winding number.
    fn winding_number(&self, coord: Coord<T>) -> ControlFlow<CoordPos, i32> {
        // Use winding number algorithm with robust predicates
        // Based on coord_pos_relative_to_ring in coordinate_position.rs
        let mut winding_number = 0;

        // Query for segments whose Y-interval contains point.y
        let point_y = YValue::from(coord.y);

        let result = self
            .y_interval_tree
            .query(point_y..=point_y, |(_, segment)| {
                // Early rejection: segment must extend to the right of point
                if segment.x_max < coord.x {
                    return ControlFlow::Continue(());
                }

                let seg = segment.segment;

                // Apply winding number algorithm using robust predicates
                let mut contribution = 0;
                if seg.0.y <= coord.y {
                    if seg.1.y >= coord.y {
                        let o = <T>::Ker::orient2d(seg.0, seg.1, coord);
                        if o == Orientation::CounterClockwise && seg.1.y != coord.y {
                            contribution = 1;
                        } else if o == Orientation::Collinear
                            && value_in_between(coord.x, seg.0.x, seg.1.x)
                        {
                            // Point on boundary!
                            return ControlFlow::Break(CoordPos::OnBoundary);
                        }
                    }
                } else {
                    let o = <T>::Ker::orient2d(seg.0, seg.1, coord);
                    if o == Orientation::Clockwise {
                        contribution = -1;
                    } else if o == Orientation::Collinear
                        && value_in_between(coord.x, seg.0.x, seg.1.x)
                    {
                        // Point on boundary!
                        return ControlFlow::Break(CoordPos::OnBoundary);
                    }
                }

                // Add the contribution directly: the winding order determines the sign
                winding_number += contribution;
                ControlFlow::Continue(())
            });

        match result {
            ControlFlow::Break(pos) => ControlFlow::Break(pos),
            ControlFlow::Continue(()) => ControlFlow::Continue(winding_number),
        }
    }
}

/// Iterates the exterior ring and all interior rings of a polygon.
fn polygon_rings<T: GeoNum>(polygon: &Polygon<T>) -> impl Iterator<Item = &LineString<T>> {
    std::iter::once(polygon.exterior()).chain(polygon.interiors().iter())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate_position::CoordinatePosition;
    use crate::wkt;

    // Probe coordinates for the 4x4 square with a 1..3 square hole used in
    // the fixtures below.
    fn probes() -> Vec<Coord<f64>> {
        vec![
            Coord { x: 0.5, y: 0.5 },  // between shell and hole
            Coord { x: 2.0, y: 2.0 },  // inside the hole
            Coord { x: 5.0, y: 5.0 },  // outside the shell
            Coord { x: 0.0, y: 2.0 },  // on the shell boundary
            Coord { x: 1.0, y: 2.0 },  // on the hole boundary
            Coord { x: 0.0, y: 0.0 },  // on a shell vertex
            Coord { x: 3.0, y: 3.0 },  // on a hole vertex
            Coord { x: -1.0, y: 0.0 }, // outside, collinear with a shell edge
        ]
    }

    #[test]
    fn parity_matches_coordinate_position_on_valid_input() {
        // Hole wound opposite to the shell (valid input, both conventions).
        let ccw_shell_cw_hole = wkt!(MULTIPOLYGON(
            ((0. 0., 4. 0., 4. 4., 0. 4., 0. 0.),
             (1. 1., 1. 3., 3. 3., 3. 1., 1. 1.))
        ));
        let cw_shell_ccw_hole = wkt!(MULTIPOLYGON(
            ((0. 0., 0. 4., 4. 4., 4. 0., 0. 0.),
             (1. 1., 3. 1., 3. 3., 1. 3., 1. 1.))
        ));
        for mp in [ccw_shell_cw_hole, cw_shell_ccw_hole] {
            let index = IntervalTreeMultiPolygon::new(&mp);
            for coord in probes() {
                let expected = mp.coordinate_position(&coord);
                assert_eq!(index.containment_parity(coord), expected, "{coord:?}");
                // The two accumulators agree on valid input.
                assert_eq!(index.containment(coord), expected, "{coord:?}");
            }
        }
    }

    #[test]
    fn parity_is_orientation_independent_on_invalid_input() {
        // The hole is wound the same way as the shell: invalid per OGC.
        let same_wound = wkt!(MULTIPOLYGON(
            ((0. 0., 4. 0., 4. 4., 0. 4., 0. 0.),
             (1. 1., 3. 1., 3. 3., 1. 3., 1. 1.))
        ));
        let index = IntervalTreeMultiPolygon::new(&same_wound);
        let in_hole = Coord { x: 2.0, y: 2.0 };

        // Parity matches the unindexed locator: the hole is outside.
        assert_eq!(index.containment_parity(in_hole), CoordPos::Outside);
        assert_eq!(same_wound.coordinate_position(&in_hole), CoordPos::Outside);
        // The nonzero-winding accumulator diverges here (winding number 2).
        // This pins the existing `Contains` behaviour on invalid input; a
        // change to this result must be deliberate.
        assert_eq!(index.containment(in_hole), CoordPos::Inside);
    }

    #[test]
    fn from_polygon_matches_multi_polygon_index() {
        let polygon = wkt!(POLYGON(
            (0. 0., 4. 0., 4. 4., 0. 4., 0. 0.),
            (1. 1., 1. 3., 3. 3., 3. 1., 1. 1.)
        ));
        let from_polygon = IntervalTreeMultiPolygon::from_polygon(&polygon);
        let from_multi = IntervalTreeMultiPolygon::new(&MultiPolygon::new(vec![polygon.clone()]));
        for coord in probes() {
            assert_eq!(
                from_polygon.containment_parity(coord),
                from_multi.containment_parity(coord),
                "{coord:?}"
            );
            assert_eq!(
                from_polygon.containment_parity(coord),
                polygon.coordinate_position(&coord),
                "{coord:?}"
            );
        }
    }
}
