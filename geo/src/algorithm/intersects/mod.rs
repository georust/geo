use crate::bounding_rect::BoundingRect;
use crate::*;

/// Checks if the geometry Self intersects the geometry Rhs.
/// More formally, either boundary or interior of Self has
/// non-empty (set-theoretic) intersection with the boundary
/// or interior of Rhs. In other words, the [DE-9IM]
/// intersection matrix for (Self, Rhs) is _not_ `FF*FF****`.
///
/// This predicate is symmetric: `a.intersects(b)` iff
/// `b.intersects(a)`.
///
/// [DE-9IM]: https://en.wikipedia.org/wiki/DE-9IM
///
/// # Examples
///
/// ```
/// use geo::algorithm::intersects::Intersects;
/// use geo::line_string;
///
/// let line_string_a = line_string![
///     (x: 3., y: 2.),
///     (x: 7., y: 6.),
/// ];
///
/// let line_string_b = line_string![
///     (x: 3., y: 4.),
///     (x: 8., y: 4.),
/// ];
///
/// let line_string_c = line_string![
///     (x: 9., y: 2.),
///     (x: 11., y: 5.),
/// ];
///
/// assert!(line_string_a.intersects(&line_string_b));
/// assert!(!line_string_a.intersects(&line_string_c));
/// ```
pub trait Intersects<Rhs = Self> {
    fn intersects(&self, rhs: &Rhs) -> bool;
}

// Since `Intersects` is symmetric, we use a macro to
// implement `T: Intersects<S>` if `S: Intersects<T>` is
// available.
//
// As a convention, we typically provide explicit impl.
// whenever the Rhs is a "simpler geometry" than the target
// type, and use the macro for the reverse impl. However,
// when there is a blanket implementations (eg. Point from
// Coordinate, MultiPoint from Point), we need to provide
// the reverse (where Self is "simpler" than Rhs).
macro_rules! symmetric_intersects_impl {
    ($t:ty, $k:ty) => {
        impl<T> $crate::algorithm::intersects::Intersects<$k> for $t
        where
            $k: $crate::algorithm::intersects::Intersects<$t>,
            T: CoordNum,
        {
            fn intersects(&self, rhs: &$k) -> bool {
                rhs.intersects(self)
            }
        }
    };
}

mod collections;
mod coordinate;
mod line;
mod line_string;
mod point;
mod polygon;
mod rect;
mod triangle;

// Helper function to check value lies between min and max.
// Only makes sense if min <= max (or always false)
#[inline]
fn value_in_range<T>(value: T, min: T, max: T) -> bool
where
    T: std::cmp::PartialOrd,
{
    value >= min && value <= max
}

// Helper function to check value lies between two bounds,
// where the ordering of the bounds is not known
#[inline]
fn value_in_between<T>(value: T, bound_1: T, bound_2: T) -> bool
where
    T: std::cmp::PartialOrd,
{
    if bound_1 < bound_2 {
        value_in_range(value, bound_1, bound_2)
    } else {
        value_in_range(value, bound_2, bound_1)
    }
}

// Helper function to check point lies inside rect given by
// bounds.  The first bound need not be min.
#[inline]
fn point_in_rect<T>(value: Coordinate<T>, bound_1: Coordinate<T>, bound_2: Coordinate<T>) -> bool
where
    T: CoordNum,
{
    value_in_between(value.x, bound_1.x, bound_2.x)
        && value_in_between(value.y, bound_1.y, bound_2.y)
}

// A cheap bbox check to see if we can skip the more expensive intersection computation
fn has_disjoint_bboxes<T, A, B>(a: &A, b: &B) -> bool
where
    T: CoordNum,
    A: BoundingRect<T>,
    B: BoundingRect<T>,
{
    let mut disjoint_bbox = false;
    if let Some(a_bbox) = a.bounding_rect().into() {
        if let Some(b_bbox) = b.bounding_rect().into() {
            if !a_bbox.intersects(&b_bbox) {
                disjoint_bbox = true;
            }
        }
    }
    disjoint_bbox
}

#[cfg(test)]
mod test {
    use crate::algorithm::intersects::Intersects;
    use crate::{
        coord, line_string, polygon, Geometry, Line, LineString, MultiLineString, MultiPoint,
        MultiPolygon, Point, Polygon, Rect,
    };

    /// Tests: intersection LineString and LineString
    #[test]
    fn empty_linestring1_test() {
        let linestring = line_string![(x: 3., y: 2.), (x: 7., y: 6.)];
        assert!(!line_string![].intersects(&linestring));
    }
    #[test]
    fn empty_linestring2_test() {
        let linestring = line_string![(x: 3., y: 2.), (x: 7., y: 6.)];
        assert!(!linestring.intersects(&LineString::new(Vec::new())));
    }
    #[test]
    fn empty_all_linestring_test() {
        let empty: LineString<f64> = line_string![];
        assert!(!empty.intersects(&empty));
    }
    #[test]
    fn intersect_linestring_test() {
        let ls1 = line_string![(x: 3., y: 2.), (x: 7., y: 6.)];
        let ls2 = line_string![(x: 3., y: 4.), (x: 8., y: 4.)];
        assert!(ls1.intersects(&ls2));
    }
    #[test]
    fn parallel_linestrings_test() {
        let ls1 = line_string![(x: 3., y: 2.), (x: 7., y: 6.)];
        let ls2 = line_string![(x: 3., y: 1.), (x: 7., y: 5.)];
        assert!(!ls1.intersects(&ls2));
    }
    /// Tests: intersection LineString and Polygon
    #[test]
    fn linestring_in_polygon_test() {
        let poly = polygon![
            (x: 0., y: 0.),
            (x: 5., y: 0.),
            (x: 5., y: 6.),
            (x: 0., y: 6.),
            (x: 0., y: 0.),
        ];
        let ls = line_string![(x: 2., y: 2.), (x: 3., y: 3.)];
        assert!(poly.intersects(&ls));
    }
    #[test]
    fn linestring_on_boundary_polygon_test() {
        let poly = Polygon::new(
            LineString::from(vec![(0., 0.), (5., 0.), (5., 6.), (0., 6.), (0., 0.)]),
            Vec::new(),
        );
        assert!(poly.intersects(&LineString::from(vec![(0., 0.), (5., 0.)])));
        assert!(poly.intersects(&LineString::from(vec![(5., 0.), (5., 6.)])));
        assert!(poly.intersects(&LineString::from(vec![(5., 6.), (0., 6.)])));
        assert!(poly.intersects(&LineString::from(vec![(0., 6.), (0., 0.)])));
    }
    #[test]
    fn intersect_linestring_polygon_test() {
        let poly = Polygon::new(
            LineString::from(vec![(0., 0.), (5., 0.), (5., 6.), (0., 6.), (0., 0.)]),
            Vec::new(),
        );
        assert!(poly.intersects(&LineString::from(vec![(2., 2.), (6., 6.)])));
    }
    #[test]
    fn linestring_outside_polygon_test() {
        let poly = Polygon::new(
            LineString::from(vec![(0., 0.), (5., 0.), (5., 6.), (0., 6.), (0., 0.)]),
            Vec::new(),
        );
        assert!(!poly.intersects(&LineString::from(vec![(7., 2.), (9., 4.)])));
    }
    #[test]
    fn linestring_in_inner_polygon_test() {
        let e = LineString::from(vec![(0., 0.), (5., 0.), (5., 6.), (0., 6.), (0., 0.)]);
        let v = vec![LineString::from(vec![
            (1., 1.),
            (4., 1.),
            (4., 4.),
            (1., 4.),
            (1., 1.),
        ])];
        let poly = Polygon::new(e, v);
        assert!(!poly.intersects(&LineString::from(vec![(2., 2.), (3., 3.)])));
        assert!(poly.intersects(&LineString::from(vec![(2., 2.), (4., 4.)])));
    }
    #[test]
    fn linestring_traverse_polygon_test() {
        let e = LineString::from(vec![(0., 0.), (5., 0.), (5., 6.), (0., 6.), (0., 0.)]);
        let v = vec![LineString::from(vec![
            (1., 1.),
            (4., 1.),
            (4., 4.),
            (1., 4.),
            (1., 1.),
        ])];
        let poly = Polygon::new(e, v);
        assert!(poly.intersects(&LineString::from(vec![(2., 0.5), (2., 5.)])));
    }
    #[test]
    fn linestring_in_inner_with_2_inner_polygon_test() {
        //                                        (8,9)
        //     (2,8)                                |                                      (14,8)
        //      ------------------------------------|------------------------------------------
        //      |                                   |                                         |
        //      |     (4,7)            (6,7)        |                                         |
        //      |       ------------------          |                    (11,7)               |
        //      |                                   |                       |                 |
        //      |     (4,6)                (7,6)    |     (9,6)             |     (12,6)      |
        //      |       ----------------------      |       ----------------|---------        |
        //      |       |                    |      |       |               |        |        |
        //      |       |       (6,5)        |      |       |               |        |        |
        //      |       |        /           |      |       |               |        |        |
        //      |       |       /            |      |       |               |        |        |
        //      |       |     (5,4)          |      |       |               |        |        |
        //      |       |                    |      |       |               |        |        |
        //      |       ----------------------      |       ----------------|---------        |
        //      |     (4,3)                (7,3)    |     (9,3)             |     (12,3)      |
        //      |                                   |                    (11,2.5)             |
        //      |                                   |                                         |
        //      ------------------------------------|------------------------------------------
        //    (2,2)                                 |                                      (14,2)
        //                                        (8,1)
        //
        let e = LineString::from(vec![(2., 2.), (14., 2.), (14., 8.), (2., 8.), (2., 2.)]);
        let v = vec![
            LineString::from(vec![(4., 3.), (7., 3.), (7., 6.), (4., 6.), (4., 3.)]),
            LineString::from(vec![(9., 3.), (12., 3.), (12., 6.), (9., 6.), (9., 3.)]),
        ];
        let poly = Polygon::new(e, v);
        assert!(!poly.intersects(&LineString::from(vec![(5., 4.), (6., 5.)])));
        assert!(poly.intersects(&LineString::from(vec![(11., 2.5), (11., 7.)])));
        assert!(poly.intersects(&LineString::from(vec![(4., 7.), (6., 7.)])));
        assert!(poly.intersects(&LineString::from(vec![(8., 1.), (8., 9.)])));
    }
    #[test]
    fn polygons_do_not_intersect() {
        let p1 = Polygon::new(
            LineString::from(vec![(1., 3.), (3., 3.), (3., 5.), (1., 5.), (1., 3.)]),
            Vec::new(),
        );
        let p2 = Polygon::new(
            LineString::from(vec![
                (10., 30.),
                (30., 30.),
                (30., 50.),
                (10., 50.),
                (10., 30.),
            ]),
            Vec::new(),
        );

        assert!(!p1.intersects(&p2));
        assert!(!p2.intersects(&p1));
    }
    #[test]
    fn polygons_overlap() {
        let p1 = Polygon::new(
            LineString::from(vec![(1., 3.), (3., 3.), (3., 5.), (1., 5.), (1., 3.)]),
            Vec::new(),
        );
        let p2 = Polygon::new(
            LineString::from(vec![(2., 3.), (4., 3.), (4., 7.), (2., 7.), (2., 3.)]),
            Vec::new(),
        );

        assert!(p1.intersects(&p2));
        assert!(p2.intersects(&p1));
    }
    #[test]
    fn polygon_contained() {
        let p1 = Polygon::new(
            LineString::from(vec![(1., 3.), (4., 3.), (4., 6.), (1., 6.), (1., 3.)]),
            Vec::new(),
        );
        let p2 = Polygon::new(
            LineString::from(vec![(2., 4.), (3., 4.), (3., 5.), (2., 5.), (2., 4.)]),
            Vec::new(),
        );

        assert!(p1.intersects(&p2));
        assert!(p2.intersects(&p1));
    }
    #[test]
    fn polygons_conincident() {
        let p1 = Polygon::new(
            LineString::from(vec![(1., 3.), (4., 3.), (4., 6.), (1., 6.), (1., 3.)]),
            Vec::new(),
        );
        let p2 = Polygon::new(
            LineString::from(vec![(1., 3.), (4., 3.), (4., 6.), (1., 6.), (1., 3.)]),
            Vec::new(),
        );

        assert!(p1.intersects(&p2));
        assert!(p2.intersects(&p1));
    }
    #[test]
    fn polygon_intersects_bounding_rect_test() {
        // Polygon poly =
        //
        // (0,8)               (12,8)
        //  ┌──────────────────────┐
        //  │         (7,7) (11,7) │
        //  │             ┌──────┐ │
        //  │             │      │ │
        //  │             │(hole)│ │
        //  │             │      │ │
        //  │             │      │ │
        //  │             └──────┘ │
        //  │         (7,4) (11,4) │
        //  │                      │
        //  │                      │
        //  │                      │
        //  │                      │
        //  │                      │
        //  └──────────────────────┘
        // (0,0)               (12,0)
        let poly = Polygon::new(
            LineString::from(vec![(0., 0.), (12., 0.), (12., 8.), (0., 8.), (0., 0.)]),
            vec![LineString::from(vec![
                (7., 4.),
                (11., 4.),
                (11., 7.),
                (7., 7.),
                (7., 4.),
            ])],
        );
        let b1 = Rect::new(coord! { x: 11., y: 1. }, coord! { x: 13., y: 2. });
        let b2 = Rect::new(coord! { x: 2., y: 2. }, coord! { x: 8., y: 5. });
        let b3 = Rect::new(coord! { x: 8., y: 5. }, coord! { x: 10., y: 6. });
        let b4 = Rect::new(coord! { x: 1., y: 1. }, coord! { x: 3., y: 3. });
        // overlaps
        assert!(poly.intersects(&b1));
        // contained in exterior, overlaps with hole
        assert!(poly.intersects(&b2));
        // completely contained in the hole
        assert!(!poly.intersects(&b3));
        // completely contained in the polygon
        assert!(poly.intersects(&b4));
        // conversely,
        assert!(b1.intersects(&poly));
        assert!(b2.intersects(&poly));
        assert!(!b3.intersects(&poly));
        assert!(b4.intersects(&poly));
    }
    #[test]
    fn bounding_rect_test() {
        let bounding_rect_xl =
            Rect::new(coord! { x: -100., y: -200. }, coord! { x: 100., y: 200. });
        let bounding_rect_sm = Rect::new(coord! { x: -10., y: -20. }, coord! { x: 10., y: 20. });
        let bounding_rect_s2 = Rect::new(coord! { x: 0., y: 0. }, coord! { x: 20., y: 30. });
        // confirmed using GEOS
        assert!(bounding_rect_xl.intersects(&bounding_rect_sm));
        assert!(bounding_rect_sm.intersects(&bounding_rect_xl));
        assert!(bounding_rect_sm.intersects(&bounding_rect_s2));
        assert!(bounding_rect_s2.intersects(&bounding_rect_sm));
    }
    #[test]
    fn rect_intersection_consistent_with_poly_intersection_test() {
        let bounding_rect_xl =
            Rect::new(coord! { x: -100., y: -200. }, coord! { x: 100., y: 200. });
        let bounding_rect_sm = Rect::new(coord! { x: -10., y: -20. }, coord! { x: 10., y: 20. });
        let bounding_rect_s2 = Rect::new(coord! { x: 0., y: 0. }, coord! { x: 20., y: 30. });

        assert!(bounding_rect_xl.to_polygon().intersects(&bounding_rect_sm));
        assert!(bounding_rect_xl.intersects(&bounding_rect_sm.to_polygon()));
        assert!(bounding_rect_xl
            .to_polygon()
            .intersects(&bounding_rect_sm.to_polygon()));

        assert!(bounding_rect_sm.to_polygon().intersects(&bounding_rect_xl));
        assert!(bounding_rect_sm.intersects(&bounding_rect_xl.to_polygon()));
        assert!(bounding_rect_sm
            .to_polygon()
            .intersects(&bounding_rect_xl.to_polygon()));

        assert!(bounding_rect_sm.to_polygon().intersects(&bounding_rect_s2));
        assert!(bounding_rect_sm.intersects(&bounding_rect_s2.to_polygon()));
        assert!(bounding_rect_sm
            .to_polygon()
            .intersects(&bounding_rect_s2.to_polygon()));

        assert!(bounding_rect_s2.to_polygon().intersects(&bounding_rect_sm));
        assert!(bounding_rect_s2.intersects(&bounding_rect_sm.to_polygon()));
        assert!(bounding_rect_s2
            .to_polygon()
            .intersects(&bounding_rect_sm.to_polygon()));
    }
    #[test]
    fn point_intersects_line_test() {
        let p0 = Point::new(2., 4.);
        // vertical line
        let line1 = Line::from([(2., 0.), (2., 5.)]);
        // point on line, but outside line segment
        let line2 = Line::from([(0., 6.), (1.5, 4.5)]);
        // point on line
        let line3 = Line::from([(0., 6.), (3., 3.)]);
        // point above line with positive slope
        let line4 = Line::from([(1., 2.), (5., 3.)]);
        // point below line with positive slope
        let line5 = Line::from([(1., 5.), (5., 6.)]);
        // point above line with negative slope
        let line6 = Line::from([(1., 2.), (5., -3.)]);
        // point below line with negative slope
        let line7 = Line::from([(1., 6.), (5., 5.)]);
        assert!(line1.intersects(&p0));
        assert!(p0.intersects(&line1));
        assert!(!line2.intersects(&p0));
        assert!(!p0.intersects(&line2));
        assert!(line3.intersects(&p0));
        assert!(p0.intersects(&line3));
        assert!(!line4.intersects(&p0));
        assert!(!p0.intersects(&line4));
        assert!(!line5.intersects(&p0));
        assert!(!p0.intersects(&line5));
        assert!(!line6.intersects(&p0));
        assert!(!p0.intersects(&line6));
        assert!(!line7.intersects(&p0));
        assert!(!p0.intersects(&line7));
    }
    #[test]
    fn line_intersects_line_test() {
        let line0 = Line::from([(0., 0.), (3., 4.)]);
        let line1 = Line::from([(2., 0.), (2., 5.)]);
        let line2 = Line::from([(0., 7.), (5., 4.)]);
        let line3 = Line::from([(0., 0.), (-3., -4.)]);
        assert!(line0.intersects(&line0));
        assert!(line0.intersects(&line1));
        assert!(!line0.intersects(&line2));
        assert!(line0.intersects(&line3));

        assert!(line1.intersects(&line0));
        assert!(line1.intersects(&line1));
        assert!(!line1.intersects(&line2));
        assert!(!line1.intersects(&line3));

        assert!(!line2.intersects(&line0));
        assert!(!line2.intersects(&line1));
        assert!(line2.intersects(&line2));
        assert!(!line1.intersects(&line3));
    }
    #[test]
    fn line_intersects_linestring_test() {
        let line0 = Line::from([(0., 0.), (3., 4.)]);
        let linestring0 = LineString::from(vec![(0., 1.), (1., 0.), (2., 0.)]);
        let linestring1 = LineString::from(vec![(0.5, 0.2), (1., 0.), (2., 0.)]);
        assert!(line0.intersects(&linestring0));
        assert!(!line0.intersects(&linestring1));
        assert!(linestring0.intersects(&line0));
        assert!(!linestring1.intersects(&line0));
    }
    #[test]
    fn line_intersects_polygon_test() {
        let line0 = Line::from([(0.5, 0.5), (2., 1.)]);
        let poly0 = Polygon::new(
            LineString::from(vec![(0., 0.), (1., 2.), (1., 0.), (0., 0.)]),
            vec![],
        );
        let poly1 = Polygon::new(
            LineString::from(vec![(1., -1.), (2., -1.), (2., -2.), (1., -1.)]),
            vec![],
        );
        // line contained in the hole
        let poly2 = Polygon::new(
            LineString::from(vec![(-1., -1.), (-1., 10.), (10., -1.), (-1., -1.)]),
            vec![LineString::from(vec![
                (0., 0.),
                (3., 4.),
                (3., 0.),
                (0., 0.),
            ])],
        );
        assert!(line0.intersects(&poly0));
        assert!(poly0.intersects(&line0));

        assert!(!line0.intersects(&poly1));
        assert!(!poly1.intersects(&line0));

        assert!(!line0.intersects(&poly2));
        assert!(!poly2.intersects(&line0));
    }
    #[test]
    // See https://github.com/georust/geo/issues/419
    fn rect_test_419() {
        let a = Rect::new(
            coord! {
                x: 9.228515625,
                y: 46.83013364044739,
            },
            coord! {
                x: 9.2724609375,
                y: 46.86019101567026,
            },
        );
        let b = Rect::new(
            coord! {
                x: 9.17953,
                y: 46.82018,
            },
            coord! {
                x: 9.26309,
                y: 46.88099,
            },
        );
        assert!(a.intersects(&b));
        assert!(b.intersects(&a));
    }

    #[test]
    fn compile_test_geom_geom() {
        // This test should check existence of all
        // combinations of geometry types.
        let geom: Geometry<_> = Line::from([(0.5, 0.5), (2., 1.)]).into();
        assert!(geom.intersects(&geom));
    }

    #[test]
    fn exhaustive_compile_test() {
        use geo_types::{GeometryCollection, Triangle};
        let pt: Point<f64> = Point::new(0., 0.);
        let ln: Line<f64> = Line::new((0., 0.), (1., 1.));
        let ls = line_string![(0., 0.).into(), (1., 1.).into()];
        let poly = Polygon::new(LineString::from(vec![(0., 0.), (1., 1.), (1., 0.)]), vec![]);
        let rect = Rect::new(coord! { x: 10., y: 20. }, coord! { x: 30., y: 10. });
        let tri = Triangle::new(
            coord! { x: 0., y: 0. },
            coord! { x: 10., y: 20. },
            coord! { x: 20., y: -10. },
        );
        let geom = Geometry::Point(pt);
        let gc = GeometryCollection::new_from(vec![geom.clone()]);
        let multi_point = MultiPoint::new(vec![pt]);
        let multi_ls = MultiLineString::new(vec![ls.clone()]);
        let multi_poly = MultiPolygon::new(vec![poly.clone()]);

        let _ = pt.intersects(&pt);
        let _ = pt.intersects(&ln);
        let _ = pt.intersects(&ls);
        let _ = pt.intersects(&poly);
        let _ = pt.intersects(&rect);
        let _ = pt.intersects(&tri);
        let _ = pt.intersects(&geom);
        let _ = pt.intersects(&gc);
        let _ = pt.intersects(&multi_point);
        let _ = pt.intersects(&multi_ls);
        let _ = pt.intersects(&multi_poly);
        let _ = ln.intersects(&pt);
        let _ = ln.intersects(&ln);
        let _ = ln.intersects(&ls);
        let _ = ln.intersects(&poly);
        let _ = ln.intersects(&rect);
        let _ = ln.intersects(&tri);
        let _ = ln.intersects(&geom);
        let _ = ln.intersects(&gc);
        let _ = ln.intersects(&multi_point);
        let _ = ln.intersects(&multi_ls);
        let _ = ln.intersects(&multi_poly);
        let _ = ls.intersects(&pt);
        let _ = ls.intersects(&ln);
        let _ = ls.intersects(&ls);
        let _ = ls.intersects(&poly);
        let _ = ls.intersects(&rect);
        let _ = ls.intersects(&tri);
        let _ = ls.intersects(&geom);
        let _ = ls.intersects(&gc);
        let _ = ls.intersects(&multi_point);
        let _ = ls.intersects(&multi_ls);
        let _ = ls.intersects(&multi_poly);
        let _ = poly.intersects(&pt);
        let _ = poly.intersects(&ln);
        let _ = poly.intersects(&ls);
        let _ = poly.intersects(&poly);
        let _ = poly.intersects(&rect);
        let _ = poly.intersects(&tri);
        let _ = poly.intersects(&geom);
        let _ = poly.intersects(&gc);
        let _ = poly.intersects(&multi_point);
        let _ = poly.intersects(&multi_ls);
        let _ = poly.intersects(&multi_poly);
        let _ = rect.intersects(&pt);
        let _ = rect.intersects(&ln);
        let _ = rect.intersects(&ls);
        let _ = rect.intersects(&poly);
        let _ = rect.intersects(&rect);
        let _ = rect.intersects(&tri);
        let _ = rect.intersects(&geom);
        let _ = rect.intersects(&gc);
        let _ = rect.intersects(&multi_point);
        let _ = rect.intersects(&multi_ls);
        let _ = rect.intersects(&multi_poly);
        let _ = tri.intersects(&pt);
        let _ = tri.intersects(&ln);
        let _ = tri.intersects(&ls);
        let _ = tri.intersects(&poly);
        let _ = tri.intersects(&rect);
        let _ = tri.intersects(&tri);
        let _ = tri.intersects(&geom);
        let _ = tri.intersects(&gc);
        let _ = tri.intersects(&multi_point);
        let _ = tri.intersects(&multi_ls);
        let _ = tri.intersects(&multi_poly);
        let _ = geom.intersects(&pt);
        let _ = geom.intersects(&ln);
        let _ = geom.intersects(&ls);
        let _ = geom.intersects(&poly);
        let _ = geom.intersects(&rect);
        let _ = geom.intersects(&tri);
        let _ = geom.intersects(&geom);
        let _ = geom.intersects(&gc);
        let _ = geom.intersects(&multi_point);
        let _ = geom.intersects(&multi_ls);
        let _ = geom.intersects(&multi_poly);
        let _ = gc.intersects(&pt);
        let _ = gc.intersects(&ln);
        let _ = gc.intersects(&ls);
        let _ = gc.intersects(&poly);
        let _ = gc.intersects(&rect);
        let _ = gc.intersects(&tri);
        let _ = gc.intersects(&geom);
        let _ = gc.intersects(&gc);
        let _ = gc.intersects(&multi_point);
        let _ = gc.intersects(&multi_ls);
        let _ = gc.intersects(&multi_poly);
        let _ = multi_point.intersects(&pt);
        let _ = multi_point.intersects(&ln);
        let _ = multi_point.intersects(&ls);
        let _ = multi_point.intersects(&poly);
        let _ = multi_point.intersects(&rect);
        let _ = multi_point.intersects(&tri);
        let _ = multi_point.intersects(&geom);
        let _ = multi_point.intersects(&gc);
        let _ = multi_point.intersects(&multi_point);
        let _ = multi_point.intersects(&multi_ls);
        let _ = multi_point.intersects(&multi_poly);
        let _ = multi_ls.intersects(&pt);
        let _ = multi_ls.intersects(&ln);
        let _ = multi_ls.intersects(&ls);
        let _ = multi_ls.intersects(&poly);
        let _ = multi_ls.intersects(&rect);
        let _ = multi_ls.intersects(&tri);
        let _ = multi_ls.intersects(&geom);
        let _ = multi_ls.intersects(&gc);
        let _ = multi_ls.intersects(&multi_point);
        let _ = multi_ls.intersects(&multi_ls);
        let _ = multi_ls.intersects(&multi_poly);
        let _ = multi_poly.intersects(&pt);
        let _ = multi_poly.intersects(&ln);
        let _ = multi_poly.intersects(&ls);
        let _ = multi_poly.intersects(&poly);
        let _ = multi_poly.intersects(&rect);
        let _ = multi_poly.intersects(&tri);
        let _ = multi_poly.intersects(&geom);
        let _ = multi_poly.intersects(&gc);
        let _ = multi_poly.intersects(&multi_point);
        let _ = multi_poly.intersects(&multi_ls);
        let _ = multi_poly.intersects(&multi_poly);
    }
}
