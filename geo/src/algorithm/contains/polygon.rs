use super::{Contains, impl_contains_from_relate, impl_contains_geometry_for};
use crate::algorithm::LinesIter;
use crate::geometry::*;
use crate::{GeoFloat, GeoNum};
use crate::{HasDimensions, Relate};
use std::cmp::Ordering;
use std::ops::ControlFlow;

use crate::intersects::value_in_between;
use crate::kernels::{Kernel, Orientation};
use crate::{Coord, LineString, MultiPolygon, Polygon};
use sif_itree::ITree;

// ┌─────────────────────────────┐
// │ Implementations for Polygon │
// └─────────────────────────────┘
impl<T> Contains<Coord<T>> for Polygon<T>
where
    T: GeoNum,
{
    fn contains(&self, coord: &Coord<T>) -> bool {
        use crate::coordinate_position::{CoordPos, CoordinatePosition};

        self.coordinate_position(coord) == CoordPos::Inside
    }
}

impl<T> Contains<Point<T>> for Polygon<T>
where
    T: GeoNum,
{
    fn contains(&self, p: &Point<T>) -> bool {
        self.contains(&p.0)
    }
}

impl<T> Contains<MultiPoint<T>> for Polygon<T>
where
    T: GeoNum,
{
    fn contains(&self, mp: &MultiPoint<T>) -> bool {
        use crate::coordinate_position::{CoordPos, CoordinatePosition};
        // at least one point must be fully within
        // others can be on the boundary
        mp.iter().any(|p| self.contains(p))
            && mp
                .iter()
                .all(|p| self.coordinate_position(&p.0) != CoordPos::Outside)
    }
}

impl_contains_from_relate!(Polygon<T>, [Line<T>, LineString<T>, Polygon<T>, MultiLineString<T>, MultiPolygon<T>, GeometryCollection<T>, Rect<T>, Triangle<T>]);
impl_contains_geometry_for!(Polygon<T>);

// ┌──────────────────────────────────┐
// │ Implementations for MultiPolygon │
// └──────────────────────────────────┘

impl<T> Contains<Coord<T>> for MultiPolygon<T>
where
    T: GeoNum,
{
    fn contains(&self, coord: &Coord<T>) -> bool {
        self.iter().any(|poly| poly.contains(coord))
    }
}

impl<T> Contains<Point<T>> for MultiPolygon<T>
where
    T: GeoNum,
{
    fn contains(&self, p: &Point<T>) -> bool {
        self.contains(&p.0)
    }
}

impl<T: GeoNum> Contains<MultiPoint<T>> for MultiPolygon<T> {
    fn contains(&self, rhs: &MultiPoint<T>) -> bool {
        if self.is_empty() || rhs.is_empty() {
            return false;
        }
        // Create IndexedMultiPolygon once and reuse for all point checks
        let indexed = IntervalTreeMultiPolygon::new(self);
        rhs.iter().all(|point| indexed.contains_point(point.0))
    }
}

impl<F> Contains<Line<F>> for MultiPolygon<F>
where
    F: GeoFloat,
{
    fn contains(&self, rhs: &Line<F>) -> bool {
        rhs.relate(self).is_within()
    }
}

impl<F> Contains<LineString<F>> for MultiPolygon<F>
where
    F: GeoFloat,
{
    fn contains(&self, rhs: &LineString<F>) -> bool {
        rhs.relate(self).is_within()
    }
}

impl<F> Contains<MultiLineString<F>> for MultiPolygon<F>
where
    F: GeoFloat,
{
    fn contains(&self, rhs: &MultiLineString<F>) -> bool {
        rhs.relate(self).is_within()
    }
}

impl<F> Contains<Polygon<F>> for MultiPolygon<F>
where
    F: GeoFloat,
{
    fn contains(&self, rhs: &Polygon<F>) -> bool {
        rhs.relate(self).is_within()
    }
}

impl<F> Contains<MultiPolygon<F>> for MultiPolygon<F>
where
    F: GeoFloat,
{
    fn contains(&self, rhs: &MultiPolygon<F>) -> bool {
        rhs.relate(self).is_within()
    }
}

impl<F> Contains<GeometryCollection<F>> for MultiPolygon<F>
where
    F: GeoFloat,
{
    fn contains(&self, rhs: &GeometryCollection<F>) -> bool {
        rhs.relate(self).is_within()
    }
}

impl<F> Contains<Rect<F>> for MultiPolygon<F>
where
    F: GeoFloat,
{
    fn contains(&self, rhs: &Rect<F>) -> bool {
        rhs.relate(self).is_within()
    }
}

impl<F> Contains<Triangle<F>> for MultiPolygon<F>
where
    F: GeoFloat,
{
    fn contains(&self, rhs: &Triangle<F>) -> bool {
        rhs.relate(self).is_within()
    }
}

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

/// A MultiPolygon backed by an [interval tree](https://en.wikipedia.org/wiki/Interval_tree) for fast containment queries
pub(crate) struct IntervalTreeMultiPolygon<T: GeoNum> {
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

    pub fn contains_point(&self, point: Coord<T>) -> bool {
        // Use winding number algorithm with robust predicates
        // Based on coord_pos_relative_to_ring in coordinate_position.rs
        let mut winding_number = 0;

        // Query for segments whose Y-interval contains point.y
        let point_y = YValue::from(point.y);

        let result = self
            .y_interval_tree
            .query(point_y..=point_y, |(_, segment)| {
                // Early rejection: segment must extend to the right of point
                if segment.x_max <= point.x {
                    return ControlFlow::Continue(());
                }

                let seg = segment.segment;

                // Apply winding number algorithm using robust predicates
                let mut contribution = 0;
                if seg.0.y <= point.y {
                    if seg.1.y >= point.y {
                        let o = <T>::Ker::orient2d(seg.0, seg.1, point);
                        if o == Orientation::CounterClockwise && seg.1.y != point.y {
                            contribution = 1;
                        } else if o == Orientation::Collinear
                            && value_in_between(point.x, seg.0.x, seg.1.x)
                        {
                            // Point on boundary!
                            return ControlFlow::Break(false);
                        }
                    }
                } else {
                    let o = <T>::Ker::orient2d(seg.0, seg.1, point);
                    if o == Orientation::Clockwise {
                        contribution = -1;
                    } else if o == Orientation::Collinear
                        && value_in_between(point.x, seg.0.x, seg.1.x)
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::{MultiPoint, Relate, coord, polygon};

    fn make_test_pts() -> [Coord<f64>; 7] {
        let pt_a = coord! {x: 0., y: 0.};
        let pt_b = coord! {x: 10., y: 0.};
        let pt_c = coord! {x: 10., y: 10.};
        let pt_d = coord! {x: 0., y: 10.};

        let pt_edge = coord! {x: 0., y: 5.};
        let pt_mid = coord! {x: 5., y: 5.};
        let pt_out = coord! {x: 11., y: 11.};
        [pt_a, pt_b, pt_c, pt_d, pt_edge, pt_mid, pt_out]
    }

    #[test]
    fn test_point_inside_indexed_polygon() {
        // Square around the origin with an extra vertex on the right side
        let square = MultiPolygon::new(vec![polygon!(
            exterior: [
                (x: -1., y: 1.),
                (x:  1., y: 1.),
                (x:  1., y: 0.),
                (x:  1., y: -1.),
                (x: -1., y: -1.),
            ],
            interiors: [],
        )]);

        let square_index = IntervalTreeMultiPolygon::new(&square);

        assert!(square_index.contains_point(Coord { x: 0.0, y: 0.0 }));
    }

    #[test]
    fn test_polygon_should_never_contain_empty_multipoint() {
        let [pt_a, pt_b, pt_c, pt_d, _pt_edge, _pt_mid, _pt_out] = make_test_pts();

        let poly = polygon![pt_a, pt_b, pt_c, pt_d, pt_a];
        let empty: MultiPoint<f64> = MultiPoint::new(Vec::new());

        // contains implementation follows `Relate`` trait
        assert!(!poly.contains(&empty));
        assert!(!poly.relate(&empty).is_contains());
    }

    #[test]
    fn test_polygon_should_contains_multipoint() {
        let [pt_a, pt_b, pt_c, pt_d, pt_edge, pt_mid, _pt_out] = make_test_pts();

        let poly = polygon![pt_a, pt_b, pt_c, pt_d, pt_a];

        // contains requires at least one point fully within the polygon
        let mp_a_mid = MultiPoint::from(vec![pt_a, pt_mid]);
        let mp_bc_mid = MultiPoint::from(vec![pt_a, pt_b, pt_mid]);
        let mp_bc_edge_mid = MultiPoint::from(vec![pt_a, pt_b, pt_edge, pt_mid]);

        assert!(poly.contains(&mp_a_mid));
        assert!(poly.contains(&mp_bc_mid));
        assert!(poly.contains(&mp_bc_edge_mid));
    }

    #[test]
    fn test_polygon_should_not_contains_multipoint_on_edge() {
        let [pt_a, pt_b, pt_c, pt_d, pt_edge, _pt_mid, _pt_out] = make_test_pts();

        let poly = polygon![pt_a, pt_b, pt_c, pt_d, pt_a];

        // contains should return false if all points lie on the boundary of the polygon
        let mp_a = MultiPoint::from(vec![pt_a]);
        let mp_bc = MultiPoint::from(vec![pt_a, pt_b]);
        let mp_bc_edge = MultiPoint::from(vec![pt_a, pt_b, pt_edge]);

        assert!(!poly.contains(&mp_a));
        assert!(!poly.contains(&mp_bc));
        assert!(!poly.contains(&mp_bc_edge));
    }

    #[test]
    fn test_polygon_should_not_contains_out_multipoint() {
        let [pt_a, pt_b, pt_c, pt_d, pt_edge, _pt_mid, pt_out] = make_test_pts();

        let poly = polygon![pt_a, pt_b, pt_c, pt_d, pt_a];

        // contains should return false if any point lies outside the polygon
        let mp_a_out = MultiPoint::from(vec![pt_a, pt_out]);
        let mp_bc_out = MultiPoint::from(vec![pt_a, pt_b, pt_out]);
        let mp_bc_edge_out = MultiPoint::from(vec![pt_a, pt_b, pt_edge, pt_out]);

        assert!(!poly.contains(&mp_a_out));
        assert!(!poly.contains(&mp_bc_out));
        assert!(!poly.contains(&mp_bc_edge_out));
    }
}
