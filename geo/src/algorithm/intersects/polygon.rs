use super::{Intersects, has_disjoint_bboxes};
use crate::coordinate_position::CoordPos;
use crate::indexed::IntervalTreeMultiPolygon;
use crate::{BoundingRect, CoordinatePosition, CoordsIter, LinesIter};
use crate::{
    Coord, CoordNum, GeoNum, Line, LineString, MultiLineString, MultiPoint, MultiPolygon, Point,
    Polygon, Rect, Triangle,
};

impl<T> Intersects<Coord<T>> for Polygon<T>
where
    T: GeoNum,
{
    fn intersects(&self, p: &Coord<T>) -> bool {
        self.coordinate_position(p) != CoordPos::Outside
    }
}

symmetric_intersects_impl!(Polygon<T>, LineString<T>);
symmetric_intersects_impl!(Polygon<T>, MultiLineString<T>);

impl<T> Intersects<Line<T>> for Polygon<T>
where
    T: GeoNum,
{
    fn intersects(&self, line: &Line<T>) -> bool {
        self.exterior().intersects(line)
            || self.interiors().iter().any(|inner| inner.intersects(line))
            || self.intersects(&line.start)
            || self.intersects(&line.end)
    }
}

symmetric_intersects_impl!(Polygon<T>, Point<T>);
symmetric_intersects_impl!(Polygon<T>, MultiPoint<T>);

impl<T: GeoNum> Intersects<Coord<T>> for IntervalTreeMultiPolygon<T> {
    fn intersects(&self, rhs: &Coord<T>) -> bool {
        self.containment(*rhs) != CoordPos::Outside
    }
}

impl<T: GeoNum> Intersects<Point<T>> for IntervalTreeMultiPolygon<T> {
    fn intersects(&self, rhs: &Point<T>) -> bool {
        self.containment(rhs.0) != CoordPos::Outside
    }
}

impl<T> Intersects<Polygon<T>> for Polygon<T>
where
    T: GeoNum,
{
    fn intersects(&self, polygon: &Polygon<T>) -> bool {
        if has_disjoint_bboxes(self, polygon) {
            return false;
        }

        // if there are no line intersections among exteriors and interiors,
        // then either one fully contains the other
        // or they are disjoint

        // check 1 point of each polygon being within the other
        self.exterior().coords_iter().take(1).any(|p|polygon.intersects(&p))
        || polygon.exterior().coords_iter().take(1).any(|p|self.intersects(&p))
        // exterior exterior
        || self.exterior().lines_iter().any(|self_line| polygon.exterior().lines_iter().any(|poly_line| self_line.intersects(&poly_line)))
        // exterior interior
        ||self.interiors().iter().any(|inner_line_string| polygon.exterior().intersects(inner_line_string))
        ||polygon.interiors().iter().any(|inner_line_string| self.exterior().intersects(inner_line_string))

        // interior interior (not needed)
        /*
           suppose interior-interior is a required check
           this requires that there are no ext-ext intersections
           and that there are no ext-int intersections
           and that self-ext[0] not intersects other
           and other-ext[0] not intersects self
           and there is some intersection between self and other

           if ext-ext disjoint, then one ext ring must be within the other ext ring

           suppose self-ext is within other-ext and self-ext[0] is not intersects other
           then self-ext[0] must be within an interior hole of other-ext
           if self-ext does not intersect the interior ring which contains self-ext[0],
           then self is contained within other interior hole
           and hence self and other cannot intersect
           therefore for self to intersect other, some part of the self-ext must intersect the other-int ring
           However, this is a contradiction because one of the premises for requiring this check is that self-ext ring does not intersect any other-int ring

           By symmetry, the mirror case of other-ext ring within self-ext ring is also true

           therefore, if there cannot exist and int-int intersection when all the prior checks are false
           and so we can skip the interior-interior check
        */
    }
}

symmetric_intersects_impl!(Polygon<T>, MultiPolygon<T>);

symmetric_intersects_impl!(Polygon<T>, Rect<T>);

symmetric_intersects_impl!(Polygon<T>, Triangle<T>);

// Blanket implementation for MultiPolygon
impl<G, T> Intersects<G> for MultiPolygon<T>
where
    T: GeoNum,
    Polygon<T>: Intersects<G>,
    G: BoundingRect<T>,
{
    fn intersects(&self, rhs: &G) -> bool {
        if has_disjoint_bboxes(self, rhs) {
            return false;
        }
        self.iter().any(|p| p.intersects(rhs))
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    #[test]
    fn geom_intersects_geom() {
        let a = Geometry::<f64>::from(polygon![]);
        let b = Geometry::from(polygon![]);
        assert!(!a.intersects(&b));
    }

    mod interval_tree_multipolygon {
        use crate::indexed::IntervalTreeMultiPolygon;
        use crate::*;

        /// Assert that the indexed `Intersects`/`Contains` impls agree with the unindexed
        /// `MultiPolygon` impls and with `Relate`, and that they match the expectation.
        fn assert_agrees(mp: &MultiPolygon, coord: Coord, intersects: bool, contains: bool) {
            let indexed = IntervalTreeMultiPolygon::new(mp);
            let point = Point::from(coord);

            assert_eq!(
                indexed.intersects(&coord),
                intersects,
                "indexed.intersects(coord) at {coord:?}"
            );
            assert_eq!(
                indexed.intersects(&point),
                intersects,
                "indexed.intersects(point) at {coord:?}"
            );
            assert_eq!(
                indexed.contains(&coord),
                contains,
                "indexed.contains(coord) at {coord:?}"
            );
            assert_eq!(
                indexed.contains(&point),
                contains,
                "indexed.contains(point) at {coord:?}"
            );

            // Sanity check against the unindexed implementations...
            assert_eq!(
                mp.intersects(&coord),
                intersects,
                "MultiPolygon::intersects at {coord:?}"
            );
            assert_eq!(
                mp.contains(&coord),
                contains,
                "MultiPolygon::contains at {coord:?}"
            );

            // ...and against Relate, which is the reference implementation.
            let im = mp.relate(&point);
            assert_eq!(
                im.is_intersects(),
                intersects,
                "Relate intersects {coord:?}"
            );
            assert_eq!(im.is_contains(), contains, "Relate contains {coord:?}");
        }

        #[test]
        fn interior_point_both_intersects_and_contains() {
            let mp = wkt!(MULTIPOLYGON(((0. 0.,4. 0.,4. 4.,0. 4.,0. 0.))));
            assert_agrees(&mp, coord! { x: 2., y: 2. }, true, true);
        }

        #[test]
        fn point_on_edge_intersects_but_is_not_contained() {
            let mp = wkt!(MULTIPOLYGON(((0. 0.,4. 0.,4. 4.,0. 4.,0. 0.))));

            // Midpoint of each of the four edges
            assert_agrees(&mp, coord! { x: 2., y: 0. }, true, false);
            assert_agrees(&mp, coord! { x: 4., y: 2. }, true, false);
            assert_agrees(&mp, coord! { x: 2., y: 4. }, true, false);
            assert_agrees(&mp, coord! { x: 0., y: 2. }, true, false);
        }

        #[test]
        fn point_on_vertex_intersects_but_is_not_contained() {
            let mp = wkt!(MULTIPOLYGON(((0. 0.,4. 0.,4. 4.,0. 4.,0. 0.))));

            for vertex in [
                coord! { x: 0., y: 0. },
                coord! { x: 4., y: 0. },
                coord! { x: 4., y: 4. },
                coord! { x: 0., y: 4. },
            ] {
                assert_agrees(&mp, vertex, true, false);
            }
        }

        #[test]
        fn exterior_point_neither() {
            let mp = wkt!(MULTIPOLYGON(((0. 0.,4. 0.,4. 4.,0. 4.,0. 0.))));

            assert_agrees(&mp, coord! { x: 5., y: 2. }, false, false);
            // Outside, but sharing a y-value with the shell — exercises the interval tree's
            // x-based early rejection in both directions.
            assert_agrees(&mp, coord! { x: -1., y: 2. }, false, false);
            assert_agrees(&mp, coord! { x: 2., y: -1. }, false, false);
        }

        #[test]
        fn hole_interior_is_outside_but_hole_boundary_intersects() {
            // A 4x4 square with a 2x2 hole in the middle.
            let mp = wkt!(MULTIPOLYGON(
                ((0. 0.,4. 0.,4. 4.,0. 4.,0. 0.),(1. 1.,1. 3.,3. 3.,3. 1.,1. 1.))
            ));

            // Strictly inside the hole: not part of the polygon at all.
            assert_agrees(&mp, coord! { x: 2., y: 2. }, false, false);

            // On the hole's boundary: part of the polygon's boundary, so it intersects
            // but is not contained.
            assert_agrees(&mp, coord! { x: 1., y: 2. }, true, false);
            assert_agrees(&mp, coord! { x: 2., y: 1. }, true, false);
            assert_agrees(&mp, coord! { x: 1., y: 1. }, true, false);

            // In the solid ring between shell and hole.
            assert_agrees(&mp, coord! { x: 0.5, y: 2. }, true, true);
        }

        #[test]
        fn multiple_polygons() {
            let mp = wkt!(MULTIPOLYGON(
                ((0. 0.,2. 0.,2. 2.,0. 2.,0. 0.)),
                ((5. 0.,7. 0.,7. 2.,5. 2.,5. 0.))
            ));

            // Interior of each component
            assert_agrees(&mp, coord! { x: 1., y: 1. }, true, true);
            assert_agrees(&mp, coord! { x: 6., y: 1. }, true, true);

            // Boundary of each component
            assert_agrees(&mp, coord! { x: 2., y: 1. }, true, false);
            assert_agrees(&mp, coord! { x: 5., y: 1. }, true, false);

            // The gap between them
            assert_agrees(&mp, coord! { x: 3.5, y: 1. }, false, false);
        }

        #[test]
        fn agrees_with_relate_over_a_grid() {
            // A concave "C" shape, so the sample grid hits interiors, boundaries,
            // vertices, and the concave notch.
            let mp = wkt!(MULTIPOLYGON(
                ((0. 0.,3. 0.,3. 1.,1. 1.,1. 2.,3. 2.,3. 3.,0. 3.,0. 0.))
            ));
            let indexed = IntervalTreeMultiPolygon::new(&mp);

            // Half-steps land on edges and vertices; quarter-steps land off them.
            for i in -2..=14 {
                for j in -2..=14 {
                    let coord = coord! { x: f64::from(i) / 4., y: f64::from(j) / 4. };
                    let im = mp.relate(&Point::from(coord));

                    assert_eq!(
                        indexed.intersects(&coord),
                        im.is_intersects(),
                        "intersects disagrees with Relate at {coord:?}"
                    );
                    assert_eq!(
                        indexed.contains(&coord),
                        im.is_contains(),
                        "contains disagrees with Relate at {coord:?}"
                    );
                }
            }
        }
    }
}
