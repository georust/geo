/// Checks if `rhs` is completely contained within `self`.
/// More formally, the interior of `rhs` has non-empty
/// (set-theoretic) intersection but neither the interior,
/// nor the boundary of `rhs` intersects the exterior of
/// `self`. In other words, the [DE-9IM] intersection matrix
/// of `(rhs, self)` is `T*F**F***`.
///
/// [DE-9IM]: https://en.wikipedia.org/wiki/DE-9IM
///
/// # Examples
///
/// ```
/// use geo::Contains;
/// use geo::{line_string, point, Polygon};
///
/// let line_string = line_string![
///     (x: 0., y: 0.),
///     (x: 2., y: 0.),
///     (x: 2., y: 2.),
///     (x: 0., y: 2.),
///     (x: 0., y: 0.),
/// ];
///
/// let polygon = Polygon::new(line_string.clone(), vec![]);
///
/// // Point in Point
/// assert!(point!(x: 2., y: 0.).contains(&point!(x: 2., y: 0.)));
///
/// // Point in Linestring
/// assert!(line_string.contains(&point!(x: 2., y: 0.)));
///
/// // Point in Polygon
/// assert!(polygon.contains(&point!(x: 1., y: 1.)));
///
/// // A `LineString`'s endpoints belong to its *boundary*, not its interior,
/// // so a point at an endpoint is not contained, while a point along the
/// // interior is (a consequence of the [DE-9IM] semantics described above):
/// let path = line_string![(x: 0., y: 0.), (x: 2., y: 0.), (x: 2., y: 2.)];
/// assert!(path.contains(&point!(x: 1., y: 0.)));   // interior point
/// assert!(!path.contains(&point!(x: 0., y: 0.)));  // start endpoint
/// assert!(!path.contains(&point!(x: 2., y: 2.)));  // end endpoint
/// ```
///
/// # Performance Note
///
/// The `MultiPolygon.contains(&MultiPoint)` containment check has been optimised for large geometries.
/// Checking many points against many polygons (or many points against a `MultiPolygon`) will be less
/// efficient than building `Multi-` versions (if possible) and checking those.
///
pub trait Contains<Rhs = Self> {
    fn contains(&self, rhs: &Rhs) -> bool;
}

mod coordinate;
mod geometry;
mod geometry_collection;
mod line;
mod line_string;
mod point;
pub(crate) mod polygon;
mod rect;
mod triangle;

macro_rules! impl_contains_from_relate {
    ($for:ty,  [$($target:ty),*]) => {
        $(
            impl<T> Contains<$target> for $for
            where
                T: GeoFloat
            {
                fn contains(&self, target: &$target) -> bool {
                    use $crate::algorithm::Relate;
                    self.relate(target).is_contains()
                }
            }
        )*
    };
}
pub(crate) use impl_contains_from_relate;

macro_rules! impl_contains_geometry_for {
    ($geom_type: ty) => {
        impl<T> Contains<Geometry<T>> for $geom_type
        where
            T: GeoFloat,
        {
            fn contains(&self, geometry: &Geometry<T>) -> bool {
                match geometry {
                    Geometry::Point(g) => self.contains(g),
                    Geometry::Line(g) => self.contains(g),
                    Geometry::LineString(g) => self.contains(g),
                    Geometry::Polygon(g) => self.contains(g),
                    Geometry::MultiPoint(g) => self.contains(g),
                    Geometry::MultiLineString(g) => self.contains(g),
                    Geometry::MultiPolygon(g) => self.contains(g),
                    Geometry::GeometryCollection(g) => self.contains(g),
                    Geometry::Rect(g) => self.contains(g),
                    Geometry::Triangle(g) => self.contains(g),
                }
            }
        }
    };
}
pub(crate) use impl_contains_geometry_for;

// ┌───────┐
// │ Tests │
// └───────┘

#[cfg(test)]
mod test {
    use crate::BoundingRect;
    use crate::Contains;
    use crate::Relate;
    use crate::indexed::IntervalTreeMultiPolygon;
    use crate::line_string;
    use crate::{Coord, Line, LineString, MultiPolygon, Point, Polygon, Rect, Triangle, coord};

    #[test]
    // see https://github.com/georust/geo/issues/452
    fn linestring_contains_point() {
        let line_string = LineString::from(vec![(0., 0.), (3., 3.)]);
        let point_on_line = Point::new(1., 1.);
        assert!(line_string.contains(&point_on_line));
    }
    #[test]
    // V doesn't contain rect because two of its edges intersect with V's exterior boundary
    fn polygon_does_not_contain_polygon() {
        let v = Polygon::new(
            vec![
                (150., 350.),
                (100., 350.),
                (210., 160.),
                (290., 350.),
                (250., 350.),
                (200., 250.),
                (150., 350.),
            ]
            .into(),
            vec![],
        );
        let rect = Polygon::new(
            vec![
                (250., 310.),
                (150., 310.),
                (150., 280.),
                (250., 280.),
                (250., 310.),
            ]
            .into(),
            vec![],
        );
        assert!(!v.contains(&rect));
    }
    #[test]
    // V contains rect because all its vertices are contained, and none of its edges intersect with V's boundaries
    fn polygon_contains_polygon() {
        let v = Polygon::new(
            vec![
                (150., 350.),
                (100., 350.),
                (210., 160.),
                (290., 350.),
                (250., 350.),
                (200., 250.),
                (150., 350.),
            ]
            .into(),
            vec![],
        );
        let rect = Polygon::new(
            vec![
                (185., 237.),
                (220., 237.),
                (220., 220.),
                (185., 220.),
                (185., 237.),
            ]
            .into(),
            vec![],
        );
        assert!(v.contains(&rect));
    }
    #[test]
    // LineString is fully contained
    fn linestring_fully_contained_in_polygon() {
        let poly = Polygon::new(
            LineString::from(vec![(0., 0.), (5., 0.), (5., 6.), (0., 6.), (0., 0.)]),
            vec![],
        );
        let ls = LineString::from(vec![(3.0, 0.5), (3.0, 3.5)]);
        assert!(poly.contains(&ls));
    }
    /// Tests: Point in LineString
    #[test]
    fn empty_linestring_test() {
        let linestring = LineString::empty();
        assert!(!linestring.contains(&Point::new(2., 1.)));
    }
    #[test]
    fn linestring_point_is_vertex_test() {
        let linestring = LineString::from(vec![(0., 0.), (2., 0.), (2., 2.)]);
        // Note: the end points of a linestring are not
        // considered to be "contained"
        assert!(linestring.contains(&Point::new(2., 0.)));
        assert!(!linestring.contains(&Point::new(0., 0.)));
        assert!(!linestring.contains(&Point::new(2., 2.)));
    }
    #[test]
    fn linestring_test() {
        let linestring = LineString::from(vec![(0., 0.), (2., 0.), (2., 2.)]);
        assert!(linestring.contains(&Point::new(1., 0.)));
    }
    /// Tests: Point in Polygon
    #[test]
    fn empty_polygon_test() {
        let poly = Polygon::empty();
        assert!(!poly.contains(&Point::new(2., 1.)));
    }
    #[test]
    fn polygon_with_one_point_test() {
        let linestring = LineString::from(vec![(2., 1.)]);
        let poly = Polygon::new(linestring, Vec::new());
        assert!(!poly.contains(&Point::new(3., 1.)));
    }
    #[test]
    fn polygon_with_one_point_is_vertex_test() {
        let linestring = LineString::from(vec![(2., 1.)]);
        let poly = Polygon::new(linestring, Vec::new());
        assert!(!poly.contains(&Point::new(2., 1.)));
    }
    #[test]
    fn polygon_with_point_on_boundary_test() {
        let linestring = LineString::from(vec![(0., 0.), (2., 0.), (2., 2.), (0., 2.), (0., 0.)]);
        let poly = Polygon::new(linestring, Vec::new());
        assert!(!poly.contains(&Point::new(1., 0.)));
        assert!(!poly.contains(&Point::new(2., 1.)));
        assert!(!poly.contains(&Point::new(1., 2.)));
        assert!(!poly.contains(&Point::new(0., 1.)));
    }
    #[test]
    fn point_in_polygon_test() {
        let linestring = LineString::from(vec![(0., 0.), (2., 0.), (2., 2.), (0., 2.), (0., 0.)]);
        let poly = Polygon::new(linestring, Vec::new());
        assert!(poly.contains(&Point::new(1., 1.)));
    }
    #[test]
    fn point_in_polygon_with_ray_passing_through_a_vertex_test() {
        let linestring = LineString::from(vec![(1., 0.), (0., 1.), (-1., 0.), (0., -1.)]);
        let poly = Polygon::new(linestring, Vec::new());
        assert!(poly.contains(&Point::new(0., 0.)));
    }
    #[test]
    fn point_in_polygon_with_ray_passing_through_a_vertex_and_not_crossing() {
        let linestring = LineString::from(vec![
            (0., 0.),
            (2., 0.),
            (3., 1.),
            (4., 0.),
            (4., 2.),
            (0., 2.),
            (0., 0.),
        ]);
        let poly = Polygon::new(linestring, Vec::new());
        assert!(poly.contains(&Point::new(1., 1.)));
    }
    #[test]
    fn point_out_polygon_test() {
        let linestring = LineString::from(vec![(0., 0.), (2., 0.), (2., 2.), (0., 2.), (0., 0.)]);
        let poly = Polygon::new(linestring, Vec::new());
        assert!(!poly.contains(&Point::new(2.1, 1.)));
        assert!(!poly.contains(&Point::new(1., 2.1)));
        assert!(!poly.contains(&Point::new(2.1, 2.1)));
    }
    #[test]
    fn point_polygon_with_inner_test() {
        let linestring = LineString::from(vec![(0., 0.), (2., 0.), (2., 2.), (0., 2.), (0., 0.)]);
        let inner_linestring = LineString::from(vec![
            [0.5, 0.5],
            [1.5, 0.5],
            [1.5, 1.5],
            [0.0, 1.5],
            [0.0, 0.0],
        ]);
        let poly = Polygon::new(linestring, vec![inner_linestring]);
        assert!(!poly.contains(&Point::new(0.25, 0.25)));
        assert!(!poly.contains(&Point::new(1., 1.)));
        assert!(!poly.contains(&Point::new(1.5, 1.5)));
        assert!(!poly.contains(&Point::new(1.5, 1.)));
    }

    /// Tests: Point in MultiPolygon
    #[test]
    fn empty_multipolygon_test() {
        let multipoly = MultiPolygon::empty();
        assert!(!multipoly.contains(&Point::new(2., 1.)));
    }
    #[test]
    fn empty_multipolygon_two_polygons_test() {
        let poly1 = Polygon::new(
            LineString::from(vec![(0., 0.), (1., 0.), (1., 1.), (0., 1.), (0., 0.)]),
            Vec::new(),
        );
        let poly2 = Polygon::new(
            LineString::from(vec![(2., 0.), (3., 0.), (3., 1.), (2., 1.), (2., 0.)]),
            Vec::new(),
        );
        let multipoly = MultiPolygon::new(vec![poly1, poly2]);
        assert!(multipoly.contains(&Point::new(0.5, 0.5)));
        assert!(multipoly.contains(&Point::new(2.5, 0.5)));
        assert!(!multipoly.contains(&Point::new(1.5, 0.5)));
    }
    #[test]
    fn empty_multipolygon_two_polygons_and_inner_test() {
        let poly1 = Polygon::new(
            LineString::from(vec![(0., 0.), (5., 0.), (5., 6.), (0., 6.), (0., 0.)]),
            vec![LineString::from(vec![
                (1., 1.),
                (4., 1.),
                (4., 4.),
                (1., 1.),
            ])],
        );
        let poly2 = Polygon::new(
            LineString::from(vec![(9., 0.), (14., 0.), (14., 4.), (9., 4.), (9., 0.)]),
            Vec::new(),
        );

        let multipoly = MultiPolygon::new(vec![poly1, poly2]);
        assert!(multipoly.contains(&Point::new(3., 5.)));
        assert!(multipoly.contains(&Point::new(12., 2.)));
        assert!(!multipoly.contains(&Point::new(3., 2.)));
        assert!(!multipoly.contains(&Point::new(7., 2.)));
    }

    #[test]
    fn empty_multipolygon_fast_test() {
        let multipoly = MultiPolygon::<f64>::new(Vec::new());
        assert!(!multipoly.contains(&Point::new(2., 1.)));
    }

    // GEOS gives us 45 points
    #[test]
    fn contains_geos() {
        let zones: MultiPolygon<f64> = geo_test_fixtures::nl_zones();
        let bound = zones.bounding_rect().unwrap();
        let mut coords = vec![];

        // Generate a bunch of points inside the zone bounds
        let size = 20;
        let mut x = bound.min().x;
        for _ in 0..=size {
            let mut y = bound.min().y;
            for _ in 0..=size {
                coords.push(Coord { x, y });
                y += bound.height() / size as f64;
            }

            x += bound.width() / size as f64;
        }

        let indexed = IntervalTreeMultiPolygon::new(&zones);
        let mut inside = 0;
        for c in &coords {
            if indexed.contains(c) {
                inside += 1;
            }
        }
        assert_eq!(inside, 45);
    }

    #[test]
    fn multipolygon_two_polygons_fast_test() {
        let poly1 = Polygon::new(
            LineString::from(vec![(0., 0.), (1., 0.), (1., 1.), (0., 1.), (0., 0.)]),
            Vec::new(),
        );
        let poly2 = Polygon::new(
            LineString::from(vec![(2., 0.), (3., 0.), (3., 1.), (2., 1.), (2., 0.)]),
            Vec::new(),
        );
        let multipoly = MultiPolygon::new(vec![poly1, poly2]);
        assert!(multipoly.contains(&Point::new(0.5, 0.5)));
        assert!(multipoly.contains(&Point::new(2.5, 0.5)));
        assert!(!multipoly.contains(&Point::new(1.5, 0.5)));
    }

    #[test]
    fn multipolygon_two_polygons_and_inner_fast_test() {
        let poly1 = Polygon::new(
            LineString::from(vec![(0., 0.), (5., 0.), (5., 6.), (0., 6.), (0., 0.)]),
            vec![LineString::from(vec![
                (1., 1.),
                (4., 1.),
                (4., 4.),
                (1., 1.),
            ])],
        );
        let poly2 = Polygon::new(
            LineString::from(vec![(9., 0.), (14., 0.), (14., 4.), (9., 4.), (9., 0.)]),
            Vec::new(),
        );

        let multipoly = MultiPolygon::new(vec![poly1, poly2]);
        assert!(multipoly.contains(&Point::new(3., 5.)));
        assert!(multipoly.contains(&Point::new(12., 2.)));
        assert!(!multipoly.contains(&Point::new(3., 2.)));
        assert!(!multipoly.contains(&Point::new(7., 2.)));
    }

    /// Tests: LineString in Polygon
    #[test]
    fn linestring_in_polygon_with_linestring_is_boundary_test() {
        let linestring = LineString::from(vec![(0., 0.), (2., 0.), (2., 2.), (0., 2.), (0., 0.)]);
        let poly = Polygon::new(linestring.clone(), Vec::new());
        assert!(!poly.contains(&linestring));
        assert!(!poly.contains(&LineString::from(vec![(0., 0.), (2., 0.)])));
        assert!(!poly.contains(&LineString::from(vec![(2., 0.), (2., 2.)])));
        assert!(!poly.contains(&LineString::from(vec![(0., 2.), (0., 0.)])));
    }
    #[test]
    fn linestring_outside_polygon_test() {
        let linestring = LineString::from(vec![(0., 0.), (2., 0.), (2., 2.), (0., 2.), (0., 0.)]);
        let poly = Polygon::new(linestring, Vec::new());
        assert!(!poly.contains(&LineString::from(vec![(1., 1.), (3., 0.)])));
        assert!(!poly.contains(&LineString::from(vec![(3., 0.), (5., 2.)])));
    }
    #[test]
    fn linestring_in_inner_polygon_test() {
        let poly = Polygon::new(
            LineString::from(vec![(0., 0.), (5., 0.), (5., 6.), (0., 6.), (0., 0.)]),
            vec![LineString::from(vec![
                (1., 1.),
                (4., 1.),
                (4., 4.),
                (1., 4.),
                (1., 1.),
            ])],
        );
        assert!(!poly.contains(&LineString::from(vec![(2., 2.), (3., 3.)])));
        assert!(!poly.contains(&LineString::from(vec![(2., 2.), (2., 5.)])));
        assert!(!poly.contains(&LineString::from(vec![(3., 0.5), (3., 5.)])));
    }
    #[test]
    fn bounding_rect_in_inner_bounding_rect_test() {
        let bounding_rect_xl =
            Rect::new(coord! { x: -100., y: -200. }, coord! { x: 100., y: 200. });
        let bounding_rect_sm = Rect::new(coord! { x: -10., y: -20. }, coord! { x: 10., y: 20. });
        assert!(bounding_rect_xl.contains(&bounding_rect_sm));
        assert!(!bounding_rect_sm.contains(&bounding_rect_xl));
    }
    #[test]
    fn point_in_line_test() {
        let c = |x, y| coord! { x: x, y: y };
        let p0 = c(2., 4.);
        // vertical line
        let line1 = Line::new(c(2., 0.), c(2., 5.));
        // point on line, but outside line segment
        let line2 = Line::new(c(0., 6.), c(1.5, 4.5));
        // point on line
        let line3 = Line::new(c(0., 6.), c(3., 3.));
        assert!(line1.contains(&Point::from(p0)));
        assert!(!line2.contains(&Point::from(p0)));
        assert!(line3.contains(&Point::from(p0)));
    }
    #[test]
    fn line_in_line_test() {
        let c = |x, y| coord! { x: x, y: y };
        let line0 = Line::new(c(0., 1.), c(3., 4.));
        // first point on line0, second not
        let line1 = Line::new(c(1., 2.), c(2., 2.));
        // co-linear, but extends past the end of line0
        let line2 = Line::new(c(1., 2.), c(4., 5.));
        // contained in line0
        let line3 = Line::new(c(1., 2.), c(3., 4.));
        assert!(!line0.contains(&line1));
        assert!(!line0.contains(&line2));
        assert!(line0.contains(&line3));
    }
    #[test]
    fn linestring_in_line_test() {
        let line = Line::from([(0, 10), (30, 40)]);
        // linestring0 in line
        let linestring0 = LineString::from(vec![(1, 11), (10, 20), (15, 25)]);
        // linestring1 starts and ends in line, but wanders in the middle
        let linestring1 = LineString::from(vec![(1, 11), (20, 20), (15, 25)]);
        // linestring2 is co-linear, but extends beyond line
        let linestring2 = LineString::from(vec![(1, 11), (10, 20), (40, 50)]);
        // no part of linestring3 is contained in line
        let linestring3 = LineString::from(vec![(11, 11), (20, 20), (25, 25)]);
        // a linestring with singleton interior on the boundary of the line
        let linestring4 = LineString::from(vec![(0, 10), (0, 10), (0, 10)]);
        // a linestring with singleton interior that is contained in the line
        let linestring5 = LineString::from(vec![(1, 11), (1, 11), (1, 11)]);
        assert!(line.contains(&linestring0));
        assert!(!line.contains(&linestring1));
        assert!(!line.contains(&linestring2));
        assert!(!line.contains(&linestring3));
        assert!(!line.contains(&linestring4));
        assert!(line.contains(&linestring5));
    }
    #[test]
    fn line_in_polygon_test() {
        let c = |x, y| coord! { x: x, y: y };
        let line = Line::new(c(0.0, 10.0), c(30.0, 40.0));
        let linestring0 = line_string![
            c(-10.0, 0.0),
            c(50.0, 0.0),
            c(50.0, 50.0),
            c(0.0, 50.0),
            c(-10.0, 0.0)
        ];
        let poly0 = Polygon::new(linestring0, Vec::new());
        let linestring1 = line_string![
            c(0.0, 0.0),
            c(0.0, 20.0),
            c(20.0, 20.0),
            c(20.0, 0.0),
            c(0.0, 0.0)
        ];
        let poly1 = Polygon::new(linestring1, Vec::new());
        assert!(poly0.contains(&line));
        assert!(!poly1.contains(&line));
    }
    #[test]
    fn line_in_polygon_edgecases_test() {
        // Some DE-9IM edge cases for checking line is
        // inside polygon The end points of the line can be
        // on the boundary of the polygon.
        let c = |x, y| coord! { x: x, y: y };
        // A non-convex polygon
        let linestring0 = line_string![
            c(0.0, 0.0),
            c(1.0, 1.0),
            c(1.0, -1.0),
            c(-1.0, -1.0),
            c(-1.0, 1.0)
        ];
        let poly = Polygon::new(linestring0, Vec::new());

        assert!(poly.contains(&Line::new(c(0.0, 0.0), c(1.0, -1.0))));
        assert!(poly.contains(&Line::new(c(-1.0, 1.0), c(1.0, -1.0))));
        assert!(!poly.contains(&Line::new(c(-1.0, 1.0), c(1.0, 1.0))));
    }
    #[test]
    fn line_in_linestring_edgecases() {
        let c = |x, y| coord! { x: x, y: y };
        use crate::line_string;
        let mut ls = line_string![c(0, 0), c(1, 0), c(0, 1), c(-1, 0)];
        assert!(!ls.contains(&Line::from([(0, 0), (0, 0)])));
        ls.close();
        assert!(ls.contains(&Line::from([(0, 0), (0, 0)])));
        assert!(ls.contains(&Line::from([(-1, 0), (1, 0)])));
    }
    #[test]
    fn line_in_linestring_test() {
        let line0 = Line::from([(1., 1.), (2., 2.)]);
        // line0 is completely contained in the second segment
        let linestring0 = LineString::from(vec![(0., 0.5), (0.5, 0.5), (3., 3.)]);
        // line0 is contained in the last three segments
        let linestring1 = LineString::from(vec![
            (0., 0.5),
            (0.5, 0.5),
            (1.2, 1.2),
            (1.5, 1.5),
            (3., 3.),
        ]);
        // line0 endpoints are contained in the linestring, but the fourth point is off the line
        let linestring2 = LineString::from(vec![
            (0., 0.5),
            (0.5, 0.5),
            (1.2, 1.2),
            (1.5, 0.),
            (2., 2.),
            (3., 3.),
        ]);
        assert!(linestring0.contains(&line0));
        assert!(linestring1.contains(&line0));
        assert!(!linestring2.contains(&line0));
    }

    #[test]
    fn integer_bounding_rects() {
        let p: Point<i32> = Point::new(10, 20);
        let bounding_rect: Rect<i32> = Rect::new(coord! { x: 0, y: 0 }, coord! { x: 100, y: 100 });
        assert!(bounding_rect.contains(&p));
        assert!(!bounding_rect.contains(&Point::new(-10, -10)));

        let smaller_bounding_rect: Rect<i32> =
            Rect::new(coord! { x: 10, y: 10 }, coord! { x: 20, y: 20 });
        assert!(bounding_rect.contains(&smaller_bounding_rect));
    }

    #[test]
    fn triangle_not_contains_point_on_edge() {
        let t = Triangle::from([(0.0, 0.0), (2.0, 0.0), (2.0, 2.0)]);
        let p = Point::new(1.0, 0.0);
        assert!(!t.contains(&p));
    }

    #[test]
    fn triangle_not_contains_point_on_vertex() {
        let t = Triangle::from([(0.0, 0.0), (2.0, 0.0), (2.0, 2.0)]);
        let p = Point::new(2.0, 0.0);
        assert!(!t.contains(&p));
    }

    #[test]
    fn triangle_contains_point_inside() {
        let t = Triangle::from([(0.0, 0.0), (2.0, 0.0), (2.0, 2.0)]);
        let p = Point::new(1.0, 0.5);
        assert!(t.contains(&p));
    }

    #[test]
    fn triangle_not_contains_point_above() {
        let t = Triangle::from([(0.0, 0.0), (2.0, 0.0), (2.0, 2.0)]);
        let p = Point::new(1.0, 1.5);
        assert!(!t.contains(&p));
    }

    #[test]
    fn triangle_not_contains_point_below() {
        let t = Triangle::from([(0.0, 0.0), (2.0, 0.0), (2.0, 2.0)]);
        let p = Point::new(-1.0, 0.5);
        assert!(!t.contains(&p));
    }

    #[test]
    fn triangle_contains_neg_point() {
        let t = Triangle::from([(0.0, 0.0), (-2.0, 0.0), (-2.0, -2.0)]);
        let p = Point::new(-1.0, -0.5);
        assert!(t.contains(&p));
    }

    #[test]
    // https://github.com/georust/geo/issues/473
    fn triangle_contains_collinear_points() {
        let origin: Coord = (0., 0.).into();
        let tri = Triangle::new(origin, origin, origin);
        let pt: Point = (0., 1.23456).into();
        assert!(!tri.contains(&pt));
        let pt: Point = (0., 0.).into();
        assert!(!tri.contains(&pt));
        let origin: Coord = (0., 0.).into();
        let tri = Triangle::new((1., 1.).into(), origin, origin);
        let pt: Point = (1., 1.).into();
        assert!(!tri.contains(&pt));
        let pt: Point = (0.5, 0.5).into();
        assert!(!tri.contains(&pt));
    }

    #[test]
    fn rect_contains_polygon() {
        let rect = Rect::new(coord! { x: 90., y: 150. }, coord! { x: 300., y: 360. });
        let poly = Polygon::new(
            line_string![
                (x: 150., y: 350.),
                (x: 100., y: 350.),
                (x: 210., y: 160.),
                (x: 290., y: 350.),
                (x: 250., y: 350.),
                (x: 200., y: 250.),
                (x: 150., y: 350.),
            ],
            vec![],
        );
        assert_eq!(rect.contains(&poly), rect.relate(&poly).is_contains());
    }

    #[test]
    fn rect_contains_touching_polygon() {
        let rect = Rect::new(coord! { x: 90., y: 150. }, coord! { x: 300., y: 360. });
        let touching_poly = Polygon::new(
            line_string![
                (x: 150., y: 350.),
                (x: 90.,  y: 350.),
                (x: 210., y: 160.),
                (x: 290., y: 350.),
                (x: 250., y: 350.),
                (x: 200., y: 250.),
                (x: 150., y: 350.),
            ],
            vec![],
        );
        assert_eq!(
            rect.contains(&touching_poly),
            rect.relate(&touching_poly).is_contains()
        );

        let touching_rect = Rect::new(coord! { x: 90., y: 200. }, coord! { x: 200., y: 300. });
        assert_eq!(
            rect.contains(&touching_rect),
            rect.relate(&touching_rect).is_contains()
        );
    }

    #[test]
    fn rect_contains_empty_polygon() {
        let rect = Rect::new(coord! { x: 90., y: 150. }, coord! { x: 300., y: 360. });
        let empty_poly = Polygon::empty();
        assert_eq!(
            rect.contains(&empty_poly),
            rect.relate(&empty_poly).is_contains()
        );
    }

    #[test]
    fn rect_contains_polygon_empty_area() {
        let rect = Rect::new(coord! { x: 90., y: 150. }, coord! { x: 300., y: 360. });
        let empty_poly = Polygon::new(
            line_string![
                (x: 100., y: 200.),
                (x: 100., y: 200.),
                (x: 100., y: 200.),
                (x: 100., y: 200.),
            ],
            vec![],
        );
        assert_eq!(
            rect.contains(&empty_poly),
            rect.relate(&empty_poly).is_contains()
        );
    }

    #[test]
    fn rect_contains_rect_polygon() {
        let rect = Rect::new(coord! { x: 90., y: 150. }, coord! { x: 300., y: 360. });
        let rect_poly = rect.to_polygon();
        assert_eq!(
            rect.contains(&rect_poly),
            rect.relate(&rect_poly).is_contains()
        );
    }

    #[test]
    fn rect_contains_polygon_in_boundary() {
        let rect = Rect::new(coord! { x: 90. , y: 150. }, coord! { x: 300., y: 360. });
        let poly_one_border =
            Rect::new(coord! { x: 90. , y: 150. }, coord! { x: 90., y: 360. }).to_polygon();
        assert_eq!(
            rect.contains(&poly_one_border),
            rect.relate(&poly_one_border).is_contains()
        );

        let poly_two_borders = Polygon::new(
            line_string![
                (x: 90., y: 150.),
                (x: 300., y: 150.),
                (x: 90., y: 150.),
                (x: 90., y: 360.),
                (x: 90., y: 150.),
            ],
            vec![],
        );
        assert_eq!(
            rect.contains(&poly_two_borders),
            rect.relate(&poly_two_borders).is_contains()
        );

        let poly_two_borders_triangle = Polygon::new(
            line_string![
                (x: 90., y: 150.),
                (x: 300., y: 150.),
                (x: 90., y: 360.),
                (x: 90., y: 150.),
            ],
            vec![],
        );
        assert_eq!(
            rect.contains(&poly_two_borders_triangle),
            rect.relate(&poly_two_borders_triangle).is_contains()
        );
    }

    #[test]
    fn rect_contains_polygon_in_boundary_with_hole() {
        let rect = Rect::new(coord! { x: 90. , y: 150. }, coord! { x: 300., y: 360. });
        let poly_two_borders_triangle_with_hole = Polygon::new(
            line_string![
                (x: 90., y: 150.),
                (x: 300., y: 150.),
                (x: 90., y: 360.),
                (x: 90., y: 150.),
            ],
            vec![line_string![
                (x: 90., y: 150.),
                (x: 300., y: 150.),
                (x: 90., y: 360.),
                (x: 90., y: 150.),
            ]],
        );
        assert_eq!(
            rect.contains(&poly_two_borders_triangle_with_hole),
            rect.relate(&poly_two_borders_triangle_with_hole)
                .is_contains()
        );
    }

    #[test]
    fn rect_empty_contains_polygon() {
        let rect = Rect::new(coord! { x: 90. , y: 150. }, coord! { x: 90., y: 150. });
        let poly = Polygon::new(
            line_string![
                (x: 150., y: 350.),
                (x: 100., y: 350.),
                (x: 210., y: 160.),
                (x: 290., y: 350.),
                (x: 250., y: 350.),
                (x: 200., y: 250.),
                (x: 150., y: 350.),
            ],
            vec![],
        );
        assert_eq!(rect.contains(&poly), rect.relate(&poly).is_contains());

        let rect_poly = rect.to_polygon();
        assert_eq!(
            rect.contains(&rect_poly),
            rect.relate(&rect_poly).is_contains()
        );
    }

    #[test]
    fn rect_contains_point() {
        let rect = Rect::new(coord! { x: 90., y: 150. }, coord! { x: 300., y: 360. });

        let point1 = Point::new(100., 200.);
        assert_eq!(rect.contains(&point1), rect.relate(&point1).is_contains());

        let point2 = Point::new(90., 200.);
        assert_eq!(rect.contains(&point2), rect.relate(&point2).is_contains());
    }

    #[test]
    fn exhaustive_compile_test() {
        use geo_types::*;
        let c = Coord { x: 0., y: 0. };
        let pt: Point = Point::new(0., 0.);
        let ls = line_string![(0., 0.).into(), (1., 1.).into()];
        let multi_ls = MultiLineString::new(vec![ls.clone()]);
        let ln: Line = Line::new((0., 0.), (1., 1.));

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
        let multi_poly = MultiPolygon::new(vec![poly.clone()]);

        let _ = c.contains(&c);
        let _ = c.contains(&pt);
        let _ = c.contains(&ln);
        let _ = c.contains(&ls);
        let _ = c.contains(&poly);
        let _ = c.contains(&rect);
        let _ = c.contains(&tri);
        let _ = c.contains(&geom);
        let _ = c.contains(&gc);
        let _ = c.contains(&multi_point);
        let _ = c.contains(&multi_ls);
        let _ = c.contains(&multi_poly);

        let _ = pt.contains(&c);
        let _ = pt.contains(&pt);
        let _ = pt.contains(&ln);
        let _ = pt.contains(&ls);
        let _ = pt.contains(&poly);
        let _ = pt.contains(&rect);
        let _ = pt.contains(&tri);
        let _ = pt.contains(&geom);
        let _ = pt.contains(&gc);
        let _ = pt.contains(&multi_point);
        let _ = pt.contains(&multi_ls);
        let _ = pt.contains(&multi_poly);

        let _ = ln.contains(&c);
        let _ = ln.contains(&pt);
        let _ = ln.contains(&ln);
        let _ = ln.contains(&ls);
        let _ = ln.contains(&poly);
        let _ = ln.contains(&rect);
        let _ = ln.contains(&tri);
        let _ = ln.contains(&geom);
        let _ = ln.contains(&gc);
        let _ = ln.contains(&multi_point);
        let _ = ln.contains(&multi_ls);
        let _ = ln.contains(&multi_poly);

        let _ = ls.contains(&c);
        let _ = ls.contains(&pt);
        let _ = ls.contains(&ln);
        let _ = ls.contains(&ls);
        let _ = ls.contains(&poly);
        let _ = ls.contains(&rect);
        let _ = ls.contains(&tri);
        let _ = ls.contains(&geom);
        let _ = ls.contains(&gc);
        let _ = ls.contains(&multi_point);
        let _ = ls.contains(&multi_ls);
        let _ = ls.contains(&multi_poly);

        let _ = poly.contains(&c);
        let _ = poly.contains(&pt);
        let _ = poly.contains(&ln);
        let _ = poly.contains(&ls);
        let _ = poly.contains(&poly);
        let _ = poly.contains(&rect);
        let _ = poly.contains(&tri);
        let _ = poly.contains(&geom);
        let _ = poly.contains(&gc);
        let _ = poly.contains(&multi_point);
        let _ = poly.contains(&multi_ls);
        let _ = poly.contains(&multi_poly);

        let _ = rect.contains(&c);
        let _ = rect.contains(&pt);
        let _ = rect.contains(&ln);
        let _ = rect.contains(&ls);
        let _ = rect.contains(&poly);
        let _ = rect.contains(&rect);
        let _ = rect.contains(&tri);
        let _ = rect.contains(&geom);
        let _ = rect.contains(&gc);
        let _ = rect.contains(&multi_point);
        let _ = rect.contains(&multi_ls);
        let _ = rect.contains(&multi_poly);

        let _ = tri.contains(&c);
        let _ = tri.contains(&pt);
        let _ = tri.contains(&ln);
        let _ = tri.contains(&ls);
        let _ = tri.contains(&poly);
        let _ = tri.contains(&rect);
        let _ = tri.contains(&tri);
        let _ = tri.contains(&geom);
        let _ = tri.contains(&gc);
        let _ = tri.contains(&multi_point);
        let _ = tri.contains(&multi_ls);
        let _ = tri.contains(&multi_poly);

        let _ = geom.contains(&c);
        let _ = geom.contains(&pt);
        let _ = geom.contains(&ln);
        let _ = geom.contains(&ls);
        let _ = geom.contains(&poly);
        let _ = geom.contains(&rect);
        let _ = geom.contains(&tri);
        let _ = geom.contains(&geom);
        let _ = geom.contains(&gc);
        let _ = geom.contains(&multi_point);
        let _ = geom.contains(&multi_ls);
        let _ = geom.contains(&multi_poly);

        let _ = gc.contains(&c);
        let _ = gc.contains(&pt);
        let _ = gc.contains(&ln);
        let _ = gc.contains(&ls);
        let _ = gc.contains(&poly);
        let _ = gc.contains(&rect);
        let _ = gc.contains(&tri);
        let _ = gc.contains(&geom);
        let _ = gc.contains(&gc);
        let _ = gc.contains(&multi_point);
        let _ = gc.contains(&multi_ls);
        let _ = gc.contains(&multi_poly);

        let _ = multi_point.contains(&c);
        let _ = multi_point.contains(&pt);
        let _ = multi_point.contains(&ln);
        let _ = multi_point.contains(&ls);
        let _ = multi_point.contains(&poly);
        let _ = multi_point.contains(&rect);
        let _ = multi_point.contains(&tri);
        let _ = multi_point.contains(&geom);
        let _ = multi_point.contains(&gc);
        let _ = multi_point.contains(&multi_point);
        let _ = multi_point.contains(&multi_ls);
        let _ = multi_point.contains(&multi_poly);

        let _ = multi_ls.contains(&c);
        let _ = multi_ls.contains(&pt);
        let _ = multi_ls.contains(&ln);
        let _ = multi_ls.contains(&ls);
        let _ = multi_ls.contains(&poly);
        let _ = multi_ls.contains(&rect);
        let _ = multi_ls.contains(&tri);
        let _ = multi_ls.contains(&geom);
        let _ = multi_ls.contains(&gc);
        let _ = multi_ls.contains(&multi_point);
        let _ = multi_ls.contains(&multi_ls);
        let _ = multi_ls.contains(&multi_poly);

        let _ = multi_poly.contains(&c);
        let _ = multi_poly.contains(&pt);
        let _ = multi_poly.contains(&ln);
        let _ = multi_poly.contains(&ls);
        let _ = multi_poly.contains(&poly);
        let _ = multi_poly.contains(&rect);
        let _ = multi_poly.contains(&tri);
        let _ = multi_poly.contains(&geom);
        let _ = multi_poly.contains(&gc);
        let _ = multi_poly.contains(&multi_point);
        let _ = multi_poly.contains(&multi_ls);
        let _ = multi_poly.contains(&multi_poly);
    }
}

#[cfg(test)]
mod hegel_props {
    use crate::coordinate_position::CoordPos;
    use crate::utils::pbt_gens::{coords, disjoint_multi_polygons, polygons_with_holes};
    use crate::{
        Contains, ContainsProperly, Coord, CoordinatePosition, Covers, Geometry, Intersects, Line,
        LineString, MultiLineString, MultiPoint, MultiPolygon, Point, Polygon, Rect, Relate,
        Triangle, Within, coord,
    };
    use hegel::generators::{self, Generator, PrintableGenerator};

    /// Geometries on the integer grid `[-4, 4]^2` that `Validation` accepts.
    ///
    /// Small integer coordinates make the geometries interact constantly and
    /// keep the kernel's arithmetic exact. Validity is by construction:
    /// endpoints of a `Line` are distinct, a `LineString` has at least two
    /// distinct coordinates, a `Triangle`'s vertices are not collinear, and
    /// polygons are built from rectangles and triangles.
    ///
    /// `Relate` documents only two preconditions — no `NaN` coordinates, and
    /// no geometry collection with overlapping polygons — but degenerate
    /// geometry reaches assertions inside it, so the domain here stays with
    /// what the crate's own validity model admits. The reproducers at the end
    /// of this module pin what happens outside it.
    fn relatable_geometries() -> impl PrintableGenerator<Geometry<f64>> {
        fn grid(tc: &hegel::TestCase) -> Coord<f64> {
            let component = || generators::integers::<i8>().min_value(-4).max_value(4);
            let (x, y) = tc.draw_silent(generators::tuples!(component(), component()));
            coord! { x: x as f64, y: y as f64 }
        }
        fn distinct_pair(tc: &hegel::TestCase) -> (Coord<f64>, Coord<f64>) {
            let a = grid(tc);
            let mut b = grid(tc);
            if a == b {
                b.x += 1.0;
            }
            (a, b)
        }
        // Rects with a zero-width or zero-height extent are degenerate, and
        // `Relate` and `Covers` disagree on them — see
        // `relate_and_covers_disagree_on_a_degenerate_rect`.
        fn rect(tc: &hegel::TestCase) -> Rect<f64> {
            let a = grid(tc);
            let mut b = grid(tc);
            if b.x == a.x {
                b.x += 1.0;
            }
            if b.y == a.y {
                b.y += 1.0;
            }
            Rect::new(a, b)
        }
        // Paths are x-monotone, hence simple: `Relate` and `Contains` disagree
        // on a self-intersecting line string against one of its own segments —
        // see `relate_and_contains_disagree_on_a_self_intersecting_line_string`.
        fn path(tc: &hegel::TestCase) -> LineString<f64> {
            let n = tc.draw_silent(generators::integers::<usize>().min_value(2).max_value(6));
            let mut x = -4.0;
            LineString::new(
                (0..n)
                    .map(|_| {
                        let coord = coord! { x: x, y: grid(tc).y };
                        x += tc.draw_silent(generators::integers::<i8>().min_value(1).max_value(2))
                            as f64;
                        coord
                    })
                    .collect(),
            )
        }
        fn triangle(tc: &hegel::TestCase) -> Triangle<f64> {
            let (a, b) = distinct_pair(tc);
            let mut c = grid(tc);
            let collinear = robust::orient2d(
                robust::Coord { x: a.x, y: a.y },
                robust::Coord { x: b.x, y: b.y },
                robust::Coord { x: c.x, y: c.y },
            ) == 0.0;
            if collinear {
                c.x += b.y - a.y;
                c.y += a.x - b.x;
            }
            Triangle::new(a, b, c)
        }
        hegel::one_of!(
            hegel::compose!(|tc| { Geometry::Point(grid(tc).into()) }),
            hegel::compose!(|tc| {
                Geometry::MultiPoint(MultiPoint::new(
                    (0..tc.draw_silent(generators::integers::<usize>().max_value(4)))
                        .map(|_| grid(tc).into())
                        .collect(),
                ))
            }),
            hegel::compose!(|tc| {
                let (a, b) = distinct_pair(tc);
                Geometry::Line(Line::new(a, b))
            }),
            hegel::compose!(|tc| { Geometry::LineString(path(tc)) }),
            hegel::compose!(|tc| {
                Geometry::MultiLineString(MultiLineString::new(
                    (0..tc.draw_silent(generators::integers::<usize>().min_value(1).max_value(3)))
                        .map(|_| path(tc))
                        .collect(),
                ))
            }),
            hegel::compose!(|tc| { Geometry::Rect(rect(tc)) }),
            hegel::compose!(|tc| { Geometry::Triangle(triangle(tc)) }),
            hegel::compose!(|tc| { Geometry::Polygon(triangle(tc).to_polygon()) }),
            hegel::compose!(|tc| {
                Geometry::MultiPolygon(MultiPolygon::new(vec![rect(tc).to_polygon()]))
            }),
        )
        .print_as_debug()
    }

    fn relatable_pairs() -> impl PrintableGenerator<(Geometry<f64>, Geometry<f64>)> {
        generators::tuples!(relatable_geometries(), relatable_geometries())
    }

    // `Contains` is documented as the DE-9IM relation the `IntersectionMatrix`
    // calls `is_contains`, and most impls are generated from `Relate`. The
    // specialized ones — `Rect`, `Triangle`, `Line`, `Polygon` against points —
    // have to agree with the general path.
    #[hegel::test]
    fn contains_agrees_with_relate(tc: hegel::TestCase) {
        let (a, b) = tc.draw(relatable_pairs());
        assert_eq!(a.contains(&b), a.relate(&b).is_contains());
    }

    #[hegel::test]
    fn covers_agrees_with_relate(tc: hegel::TestCase) {
        let (a, b) = tc.draw(relatable_pairs());
        assert_eq!(a.covers(&b), a.relate(&b).is_covers());
    }

    #[hegel::test]
    fn intersects_agrees_with_relate(tc: hegel::TestCase) {
        let (a, b) = tc.draw(relatable_pairs());
        assert_eq!(a.intersects(&b), a.relate(&b).is_intersects());
    }

    // `ContainsProperly` is documented to delegate to
    // `IntersectionMatrix::is_contains_properly`, with a faster specialized
    // path "when checking between `Polygon` and `MultiPolygon`". The left-hand
    // geometry is valid here: the fast path disagrees with `Relate` when it
    // holds a zero-area member, pinned by
    // `contains_properly_on_a_degenerate_member_disagrees_with_relate`.
    #[hegel::test]
    fn contains_properly_agrees_with_relate(tc: hegel::TestCase) {
        let a = tc.draw(polygons_with_holes());
        let b = tc.draw(relatable_geometries());
        assert_eq!(a.contains_properly(&b), a.relate(&b).is_contains_properly());
    }

    #[hegel::test]
    fn multi_polygon_contains_properly_agrees_with_relate(tc: hegel::TestCase) {
        let a = tc.draw(disjoint_multi_polygons());
        let b = tc.draw(relatable_geometries());
        assert_eq!(a.contains_properly(&b), a.relate(&b).is_contains_properly());
    }

    // `Contains` requires the interior of `rhs` to have "non-empty
    // (set-theoretic) intersection" with `self`, which is strictly stronger
    // than `Intersects` — "either boundary or interior of Self has non-empty
    // intersection with the boundary or interior of Rhs".
    #[hegel::test]
    fn contains_implies_intersects(tc: hegel::TestCase) {
        let (a, b) = tc.draw(relatable_pairs());
        if a.contains(&b) {
            assert!(
                a.intersects(&b),
                "{a:?} contains {b:?} but does not intersect it"
            );
        }
    }

    // `Covers` "does not distinguish between points in the boundary and in the
    // interior of geometries", and its first documented mask is exactly the
    // `is_contains` mask, so containment implies covering.
    #[hegel::test]
    fn contains_implies_covers(tc: hegel::TestCase) {
        let (a, b) = tc.draw(relatable_pairs());
        if a.contains(&b) {
            assert!(a.covers(&b), "{a:?} contains {b:?} but does not cover it");
        }
    }

    // "If Geometry `b` has any interaction with the boundary of Geometry `a`,
    // then the result is `false`", so the `is_contains_properly` mask
    // `T**FF*FF*` is the `is_contains` mask `T*****FF*` with two more cells
    // forced empty.
    #[hegel::test]
    fn contains_properly_implies_contains(tc: hegel::TestCase) {
        let (a, b) = tc.draw(relatable_pairs());
        let matrix = a.relate(&b);
        if matrix.is_contains_properly() {
            assert!(matrix.is_contains());
        }
    }

    // `Within` is documented as "equivalent to `Contains` with the arguments
    // swapped"; the same swap must show up in the intersection matrix as
    // `is_within`.
    #[hegel::test]
    fn within_is_contains_with_the_arguments_swapped(tc: hegel::TestCase) {
        let (a, b) = tc.draw(relatable_pairs());
        assert_eq!(a.is_within(&b), b.contains(&a));
        assert_eq!(b.relate(&a).is_contains(), a.relate(&b).is_within());
    }

    // The matrix is indexed by the position of a point of `a` against a point
    // of `b`, so swapping the arguments transposes it.
    #[hegel::test]
    fn swapping_the_arguments_transposes_the_intersection_matrix(tc: hegel::TestCase) {
        let (a, b) = tc.draw(relatable_pairs());
        let forward = a.relate(&b);
        let backward = b.relate(&a);
        for lhs in [CoordPos::Inside, CoordPos::OnBoundary, CoordPos::Outside] {
            for rhs in [CoordPos::Inside, CoordPos::OnBoundary, CoordPos::Outside] {
                assert_eq!(forward.get(lhs, rhs), backward.get(rhs, lhs));
            }
        }
    }

    // `Polygon::contains` for a coordinate is defined as the coordinate being
    // `Inside`, and `Polygon::intersects` as its not being `Outside`.
    #[hegel::test]
    fn coordinate_position_decides_polygon_containment(tc: hegel::TestCase) {
        let polygon = tc.draw(polygons_with_holes());
        let coord = tc.draw(coords(2e3));
        let position = polygon.coordinate_position(&coord);
        assert_eq!(polygon.contains(&coord), position == CoordPos::Inside);
        assert_eq!(
            polygon.intersects(&Point::from(coord)),
            position != CoordPos::Outside
        );
    }

    // KNOWN FAILURE, raised in #1609: `RelateOperation` labels its graph
    // nodes through a map keyed with `total_cmp`, which separates `-0.0` from
    // `0.0`, while the segment intersection that creates those nodes compares
    // with `==`, which does not. A ring holding a negative zero therefore
    // leaves a node labelled for only one of the two geometries and
    // `Node::update_intersection_matrix` panics on its own `assert!` — in
    // release builds as well. Related to #1578, which is the same
    // ordering mismatch in the sweep module.
    #[test]
    #[ignore = "open question, see #1609: Relate panics on a ring containing a negative zero"]
    fn relate_panics_on_a_ring_holding_a_negative_zero() {
        let point = Point::new(0.0, 0.0);
        let polygon = Polygon::new(
            vec![
                coord! { x: 1.0, y: 0.0 },
                coord! { x: 0.0, y: 0.0 },
                coord! { x: 1.0, y: -0.0 },
            ]
            .into(),
            vec![],
        );
        assert!(!point.relate(&polygon).is_contains());
    }

    // KNOWN FAILURE, raised in #1609: the specialized
    // `MultiPolygon::contains_properly` path only asks whether the right-hand
    // geometry's coordinates fall inside some member. Here the right-hand
    // polygon is a zero-area member of the left-hand multi polygon, so its
    // point is inside the triangle but also on the left-hand boundary, and the
    // documented `T**FF*FF*` mask does not hold: `Relate::is_contains_properly`
    // says false where the specialized path says true.
    #[test]
    #[ignore = "open question, see #1609: MultiPolygon::contains_properly disagrees with Relate on a zero-area member"]
    fn contains_properly_on_a_degenerate_member_disagrees_with_relate() {
        let degenerate: Polygon<f64> =
            Polygon::new(vec![coord! { x: 0.25, y: 0.25 }].into(), vec![]);
        let triangle: Polygon<f64> = Polygon::new(
            vec![
                coord! { x: 0.0, y: 0.0 },
                coord! { x: 0.0, y: 1.0 },
                coord! { x: 1.0, y: 0.0 },
            ]
            .into(),
            vec![],
        );
        let a = MultiPolygon::new(vec![degenerate.clone(), triangle]);
        let b = MultiPolygon::new(vec![degenerate]);
        assert_eq!(a.contains_properly(&b), a.relate(&b).is_contains_properly());
    }

    // KNOWN FAILURE, raised in #1609: the line string's last segment is
    // exactly the line, so the line's interior cannot meet the line string's
    // exterior — but `Relate` reports `EI` as one-dimensional and `II` as a
    // single point. The line string is self-intersecting (its last segment
    // crosses its first), which `InvalidLineString` permits.
    #[test]
    #[ignore = "open question, see #1609: Relate and Contains disagree on a self-intersecting line string against its own segment"]
    fn relate_and_contains_disagree_on_a_self_intersecting_line_string() {
        let line_string: LineString<f64> = vec![
            coord! { x: 0.0, y: 0.0 },
            coord! { x: 1.0, y: 0.0 },
            coord! { x: 0.0, y: -1.0 },
            coord! { x: 2.0, y: 2.0 },
        ]
        .into();
        let line = Line::new(coord! { x: 0.0, y: -1.0 }, coord! { x: 2.0, y: 2.0 });
        assert_eq!(
            line_string.contains(&line),
            line_string.relate(&line).is_contains()
        );
    }

    // KNOWN FAILURE, raised in #1609: `Covers` gives different answers for
    // the same two geometries depending on whether they are passed as concrete
    // types or wrapped in the `Geometry` enum, which only dispatches to the
    // concrete impls.
    #[test]
    #[ignore = "open question, see #1609: Geometry::covers disagrees with the concrete impl it dispatches to"]
    fn covers_does_not_depend_on_the_geometry_wrapper() {
        let line_string: LineString<f64> = vec![
            coord! { x: 0.0, y: 0.0 },
            coord! { x: 1.0, y: 0.0 },
            coord! { x: 0.0, y: -1.0 },
            coord! { x: 2.0, y: 2.0 },
        ]
        .into();
        let line = Line::new(coord! { x: 0.0, y: -1.0 }, coord! { x: 2.0, y: 2.0 });
        assert_eq!(
            line_string.covers(&line),
            Geometry::LineString(line_string.clone()).covers(&Geometry::Line(line))
        );
    }

    // KNOWN FAILURE, raised in #1609: a `Rect` with zero height is a
    // segment, which cannot cover the unit square, and `Covers` agrees — but
    // `Relate::is_covers` reports that it does. `InvalidRect` only rejects
    // non-finite coordinates, so a degenerate rect is inside the crate's own
    // validity model.
    #[test]
    #[ignore = "open question, see #1609: Relate::is_covers and Covers disagree on a zero-height Rect"]
    fn relate_and_covers_disagree_on_a_degenerate_rect() {
        let segment = Rect::new(coord! { x: 0.0, y: 0.0 }, coord! { x: 1.0, y: 0.0 });
        let square = Rect::new(coord! { x: 0.0, y: 0.0 }, coord! { x: 1.0, y: 1.0 });
        assert_eq!(segment.covers(&square), segment.relate(&square).is_covers());
    }

    // KNOWN FAILURE, raised in #1609: `Relate` reports a zero-length `Line`
    // as one-dimensional, so the interior-interior cell disagrees with the
    // transposed matrix and with `HasDimensions`, which documents a "degenerate
    // line" as a point. Relating the equivalent `Point` gives the expected
    // `F0FFFF212`.
    #[test]
    #[ignore = "open question, see #1609: Relate treats a zero-length Line as one-dimensional"]
    fn relate_treats_a_zero_length_line_as_a_point() {
        let line = Line::new(coord! { x: 0.0, y: 0.0 }, coord! { x: 0.0, y: 0.0 });
        let rect = Rect::new(coord! { x: 0.0, y: 0.0 }, coord! { x: 1.0, y: 1.0 });
        assert_eq!(
            line.relate(&rect).get(CoordPos::Inside, CoordPos::Inside),
            rect.relate(&line).get(CoordPos::Inside, CoordPos::Inside)
        );
    }

    // KNOWN FAILURE, raised in #1609: a `Polygon` whose exterior is empty is
    // empty, and `Relate` agrees. `Intersects` for a `Line` also checks the
    // interior rings, so with one present it reports an intersection.
    #[test]
    #[ignore = "open question, see #1609: Intersects and Relate disagree on an empty polygon with interior rings"]
    fn an_empty_polygon_with_interior_rings_intersects_nothing() {
        let polygon = Polygon::new(
            LineString::new(vec![]),
            vec![
                vec![
                    coord! { x: 0.0, y: 0.0 },
                    coord! { x: 0.0, y: 1.0 },
                    coord! { x: 1.0, y: 1.0 },
                    coord! { x: 1.0, y: 0.0 },
                ]
                .into(),
            ],
        );
        let line = Line::new(coord! { x: -1.0, y: 0.5 }, coord! { x: 2.0, y: 0.5 });
        assert_eq!(
            polygon.intersects(&line),
            polygon.relate(&line).is_intersects()
        );
    }
}
