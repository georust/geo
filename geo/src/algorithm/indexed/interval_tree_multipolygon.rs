use std::{cmp::Ordering, ops::ControlFlow};

use geo_types::{Coord, Line, MultiPolygon};
use sif_itree::ITree;

use crate::algorithm::kernels::Kernel;
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
    is_exterior: bool,
}

/// A [`MultiPolygon`] backed by an [interval tree](https://en.wikipedia.org/wiki/Interval_tree) for fast containment queries
pub struct IntervalTreeMultiPolygon<T: GeoNum> {
    y_interval_tree: ITree<YValue<T>, YIntervalSegment<T>>,
}

impl<T: GeoNum> IntervalTreeMultiPolygon<T> {
    pub fn new(mp: &MultiPolygon<T>) -> Self {
        // Chain all exterior and interior line segments from all polygons
        let segments =
            mp.0.iter()
                .flat_map(|polygon| {
                    polygon
                        .exterior()
                        .lines_iter()
                        .map(|line| (line, true))
                        .chain(
                            polygon
                                .interiors()
                                .iter()
                                .flat_map(|interior| interior.lines_iter())
                                .map(|line| (line, false)),
                        )
                })
                .map(|(line, is_exterior)| Self::create_segment(line, is_exterior));

        Self {
            y_interval_tree: ITree::new(segments),
        }
    }

    fn create_segment(
        line: Line<T>,
        is_exterior: bool,
    ) -> sif_itree::Item<YValue<T>, YIntervalSegment<T>> {
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
                is_exterior,
            },
        )
    }

    pub(crate) fn containment(&self, coord: Coord<T>) -> bool {
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
                            return ControlFlow::Break(false);
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
                        return ControlFlow::Break(false);
                    }
                }

                // Exterior rings contribute positively, interior rings (holes) contribute negatively
                if segment.is_exterior {
                    winding_number += contribution;
                } else {
                    winding_number -= contribution;
                }
                ControlFlow::Continue(())
            });

        match result {
            ControlFlow::Break(r) => r,
            ControlFlow::Continue(()) => winding_number != 0,
        }
    }
}
