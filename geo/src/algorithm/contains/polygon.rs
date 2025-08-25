use super::{impl_contains_from_relate, impl_contains_geometry_for, Contains};
use crate::geometry::*;
use crate::{GeoFloat, GeoNum};
use crate::{HasDimensions, Relate};

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
        rhs.iter().all(|point| self.contains(point))
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

// Extension trait for optimized point-in-multipolygon test
pub trait ContainsPointFast<T: GeoFloat + rstar::RTreeNum> {
    /// An optimized point-in-multipolygon test using an interval tree.
    fn contains_point_fast(&self, point: &Point<T>) -> bool;
}

impl<T> ContainsPointFast<T> for MultiPolygon<T>
where
    T: GeoFloat + rstar::RTreeNum,
{
    fn contains_point_fast(&self, point: &Point<T>) -> bool {
        // Handle empty multipolygon
        if self.0.is_empty() {
            return false;
        }
        let imp = IndexedMultiPolygon::new(self);
        imp.contains_point(point.0)
    }
}

use crate::{Coord, LineString, MultiPolygon, Polygon};
use rstar::{RTree, AABB};

struct YIntervalSegment<F: GeoFloat> {
    y_min: F,
    y_max: F,
    segment: (Coord<F>, Coord<F>),
}

impl<F: GeoFloat + rstar::RTreeNum> rstar::RTreeObject for YIntervalSegment<F> {
    type Envelope = AABB<[F; 2]>;

    fn envelope(&self) -> Self::Envelope {
        // Use 2D AABB with x-extent covering the segment's x-range
        let x_min = self.segment.0.x.min(self.segment.1.x);
        let x_max = self.segment.0.x.max(self.segment.1.x);
        AABB::from_corners([x_min, self.y_min], [x_max, self.y_max])
    }
}

pub struct IndexedMultiPolygon<F: GeoFloat + rstar::RTreeNum> {
    y_interval_tree: RTree<YIntervalSegment<F>>,
    geometry: MultiPolygon<F>,
}

impl<F: GeoFloat + rstar::RTreeNum> IndexedMultiPolygon<F> {
    pub fn new(mp: &MultiPolygon<F>) -> Self {
        let mp = mp.clone();
        let mut segments = Vec::new();

        for polygon in &mp.0 {
            Self::add_ring_segments(&mut segments, polygon.exterior());
            for interior in polygon.interiors() {
                Self::add_ring_segments(&mut segments, interior);
            }
        }

        // RTree requires at least one element for 1D trees, so handle empty case
        let y_interval_tree = if segments.is_empty() {
            RTree::new()
        } else {
            RTree::bulk_load(segments)
        };

        Self {
            y_interval_tree,
            geometry: mp,
        }
    }

    fn add_ring_segments(segments: &mut Vec<YIntervalSegment<F>>, ring: &LineString<F>) {
        for window in ring.coords().collect::<Vec<_>>().windows(2) {
            let (p1, p2) = (window[0], window[1]);
            segments.push(YIntervalSegment {
                y_min: p1.y.min(p2.y),
                y_max: p1.y.max(p2.y),
                segment: (*p1, *p2),
            });
        }
    }

    pub fn contains_point(&self, point: Coord<F>) -> bool {
        // Query the R-tree for segments whose bounding box intersects with a horizontal ray
        // from the point extending to the right.
        // We use a degenerate AABB (a horizontal line segment) for the query.
        let query_envelope = AABB::from_corners(
            [point.x, point.y],
            [F::infinity(), point.y], // Same y-coordinate creates a horizontal line
        );

        let candidates = self
            .y_interval_tree
            .locate_in_envelope_intersecting(&query_envelope)
            .filter(|seg| {
                // Additional filtering: segment must actually cross the y-coordinate
                // and extend to the right of the point
                seg.segment.0.x.max(seg.segment.1.x) > point.x
            });

        let mut crossings = 0;
        for segment in candidates {
            if self.ray_intersects_segment(point, segment.segment) {
                crossings += 1;
            }
        }

        crossings % 2 == 1
    }

    fn ray_intersects_segment(&self, point: Coord<F>, seg: (Coord<F>, Coord<F>)) -> bool {
        let (p1, p2) = seg;

        // Skip horizontal segments
        if (p1.y - p2.y).abs() < F::epsilon() {
            return false;
        }

        let y_min = p1.y.min(p2.y);
        let y_max = p1.y.max(p2.y);

        if point.y < y_min || point.y > y_max {
            return false;
        }

        // Handle endpoint intersections
        if (point.y - y_min).abs() < F::epsilon() {
            return p1.y < p2.y;
        }
        if (point.y - y_max).abs() < F::epsilon() {
            return p2.y < p1.y;
        }

        // Calculate intersection X coordinate
        let t = (point.y - p1.y) / (p2.y - p1.y);
        let intersection_x = p1.x + t * (p2.x - p1.x);

        intersection_x > point.x
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{coord, polygon, MultiPoint, Relate};

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
