use super::{Contains, impl_contains_from_relate, impl_contains_geometry_for};
use crate::geometry::*;
use crate::indexed::IntervalTreeMultiPolygon;
use crate::{GeoFloat, GeoNum};
use crate::{HasDimensions, Relate};

use crate::{Coord, LineString, MultiPolygon, Polygon};

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
        rhs.iter().all(|point| indexed.contains(&point.0))
    }
}

impl<T: GeoNum> Contains<Coord<T>> for IntervalTreeMultiPolygon<T> {
    fn contains(&self, rhs: &Coord<T>) -> bool {
        self.containment(*rhs)
    }
}

impl<T: GeoNum> Contains<Point<T>> for IntervalTreeMultiPolygon<T> {
    fn contains(&self, rhs: &Point<T>) -> bool {
        let c = Coord::from(*rhs);
        self.containment(c)
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

impl_contains_geometry_for!(MultiPolygon<T>);

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Convert, MultiPoint, Relate, coord, polygon, wkt};

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
    fn test_point_on_boundary_indexed_polygon() {
        // Square around the origin with an extra vertex on the right side
        let square: MultiPolygon = wkt!(MULTIPOLYGON(((-1 1, 1 1, 1 -1, -1 -1, -1 1)))).convert();
        let square_index = IntervalTreeMultiPolygon::new(&square);

        assert!(!square_index.contains(&Coord { x: -1.0, y: 1.0 }));

        assert!(!square_index.contains(&Coord { x: -1.0, y: 0.5 }));

        assert!(!square_index.contains(&Coord { x: -1.0, y: 0.0 }));

        assert!(!square_index.contains(&Coord { x: -2.0, y: 0.0 }));
    }

    #[test]
    fn test_external_point_horizontal_from_tip() {
        let triangle: MultiPolygon = wkt!(MULTIPOLYGON (((-1 0, 0 1, 1 0, -1 0)))).convert();
        let triangle_index = IntervalTreeMultiPolygon::new(&triangle);

        assert!(!triangle.contains(&Coord { x: -0.75, y: 1.0 }));
        assert!(!triangle_index.contains(&Coord { x: -0.75, y: 1.0 }));

        assert!(!triangle.contains(&Coord { x: -0.5, y: 0.5 }));
        assert!(!triangle_index.contains(&Coord { x: -0.5, y: 0.5 }));

        assert!(!triangle.contains(&Coord { x: 0.0, y: 1.0 }));
        assert!(!triangle_index.contains(&Coord { x: 0.0, y: 1.0 }));

        assert!(!triangle.contains(&Coord { x: 0.75, y: 1.0 }));
        assert!(!triangle_index.contains(&Coord { x: 0.75, y: 1.0 }));
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

        assert!(square_index.contains(&Coord { x: 0.0, y: 0.0 }));
    }

    #[test]
    fn test_polygon_should_never_contain_empty_multipoint() {
        let [pt_a, pt_b, pt_c, pt_d, _pt_edge, _pt_mid, _pt_out] = make_test_pts();

        let poly = polygon![pt_a, pt_b, pt_c, pt_d, pt_a];
        let empty: MultiPoint<f64> = MultiPoint::empty();

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

    #[test]
    fn test_hollow_square_ccw_exterior_cw_interior() {
        // Standard OGC winding: exterior shell CCW, interior ring CW
        let hollow_square = wkt!(MULTIPOLYGON(((-2.0 -2.0,2.0 -2.0,2.0 2.0,-2.0 2.0,-2.0 -2.0),(-1.0 -1.0,-1.0 1.0,1.0 1.0,1.0 -1.0,-1.0 -1.0))));
        let hollow_square_index = IntervalTreeMultiPolygon::new(&hollow_square);
        // Point in the hole should not be contained
        assert!(!hollow_square_index.contains(&Coord { x: 0.0, y: 0.0 }));
        // Point in the solid part should be contained
        assert!(hollow_square_index.contains(&Coord { x: 1.5, y: 0.0 }));
    }

    #[test]
    fn test_hollow_square_cw_exterior_ccw_interior() {
        // Non-standard winding: exterior shell CW, interior ring CCW
        let hollow_square = wkt!(MULTIPOLYGON(((-2.0 -2.0,-2.0 2.0,2.0 2.0,2.0 -2.0,-2.0 -2.0),(-1.0 -1.0,1.0 -1.0,1.0 1.0,-1.0 1.0,-1.0 -1.0))));
        let hollow_square_index = IntervalTreeMultiPolygon::new(&hollow_square);
        // Point in the hole should not be contained
        assert!(!hollow_square_index.contains(&Coord { x: 0.0, y: 0.0 }));
        // Point in the solid part should be contained
        assert!(hollow_square_index.contains(&Coord { x: 1.5, y: 0.0 }));
    }
}
