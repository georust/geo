use std::ops::Mul;

use num_traits::FromPrimitive;

use crate::{
    Coord, CoordFloat, Geometry, LineString, MultiLineString, MultiPolygon, Polygon, coord,
};

/// Smoothen `LineString`, `Polygon`, `MultiLineString` and `MultiPolygon` using Chaikins algorithm.
///
/// [Chaikins smoothing algorithm](http://www.idav.ucdavis.edu/education/CAGDNotes/Chaikins-Algorithm/Chaikins-Algorithm.html)
///
/// Each iteration of the smoothing doubles the number of vertices of the geometry, so in some
/// cases it may make sense to apply a simplification afterwards to remove insignificant
/// coordinates.
///
/// This implementation preserves the start and end vertices of an open linestring and
/// smoothes the corner between start and end of a closed linestring.
pub trait ChaikinSmoothing<T>
where
    T: CoordFloat + FromPrimitive,
{
    /// create a new geometry with the Chaikin smoothing being
    /// applied `n_iterations` times.
    fn chaikin_smoothing(&self, n_iterations: usize) -> Self;
}

impl<T> ChaikinSmoothing<T> for LineString<T>
where
    T: CoordFloat + FromPrimitive,
{
    fn chaikin_smoothing(&self, n_iterations: usize) -> Self {
        if n_iterations == 0 {
            self.clone()
        } else {
            let mut smooth = smoothen_linestring(self);
            for _ in 0..(n_iterations - 1) {
                smooth = smoothen_linestring(&smooth);
            }
            smooth
        }
    }
}

impl<T> ChaikinSmoothing<T> for MultiLineString<T>
where
    T: CoordFloat + FromPrimitive,
{
    fn chaikin_smoothing(&self, n_iterations: usize) -> Self {
        MultiLineString::new(
            self.0
                .iter()
                .map(|ls| ls.chaikin_smoothing(n_iterations))
                .collect(),
        )
    }
}

impl<T> ChaikinSmoothing<T> for Polygon<T>
where
    T: CoordFloat + FromPrimitive,
{
    fn chaikin_smoothing(&self, n_iterations: usize) -> Self {
        Polygon::new(
            self.exterior().chaikin_smoothing(n_iterations),
            self.interiors()
                .iter()
                .map(|ls| ls.chaikin_smoothing(n_iterations))
                .collect(),
        )
    }
}

impl<T> ChaikinSmoothing<T> for MultiPolygon<T>
where
    T: CoordFloat + FromPrimitive,
{
    fn chaikin_smoothing(&self, n_iterations: usize) -> Self {
        MultiPolygon::new(
            self.0
                .iter()
                .map(|poly| poly.chaikin_smoothing(n_iterations))
                .collect(),
        )
    }
}

macro_rules! blanket_run_chaikin_smoothing {
    ($geo:expr, $n_iter:expr) => {{
        let smooth = $geo.chaikin_smoothing($n_iter);
        let geo: Geometry<T> = smooth.into();
        geo
    }};
}

impl<T> ChaikinSmoothing<T> for Geometry<T>
where
    T: CoordFloat + FromPrimitive,
{
    fn chaikin_smoothing(&self, n_iterations: usize) -> Geometry<T> {
        match self {
            Geometry::LineString(child) => blanket_run_chaikin_smoothing!(child, n_iterations),
            Geometry::MultiLineString(child) => blanket_run_chaikin_smoothing!(child, n_iterations),
            Geometry::Polygon(child) => blanket_run_chaikin_smoothing!(child, n_iterations),
            Geometry::MultiPolygon(child) => blanket_run_chaikin_smoothing!(child, n_iterations),
            _ => self.clone(),
        }
    }
}

fn smoothen_linestring<T>(linestring: &LineString<T>) -> LineString<T>
where
    T: CoordFloat + Mul<T> + FromPrimitive,
{
    let mut out_coords: Vec<_> = Vec::with_capacity(linestring.0.len() * 2);

    if let (Some(first), Some(last)) = (linestring.0.first(), linestring.0.last())
        && first != last
    {
        // preserve start coordinate when the linestring is open
        out_coords.push(*first);
    }
    for window_coordinates in linestring.0.windows(2) {
        let (q, r) = smoothen_coordinates(window_coordinates[0], window_coordinates[1]);
        out_coords.push(q);
        out_coords.push(r);
    }

    if let (Some(first), Some(last)) = (linestring.0.first(), linestring.0.last()) {
        if first != last {
            // preserve the last coordinate of an open linestring
            out_coords.push(*last);
        } else {
            // smoothen the edge between the beginning and the end in closed
            // linestrings while keeping the linestring closed.
            if let Some(out_first) = out_coords.first().copied() {
                out_coords.push(out_first);
            }
        }
    }
    out_coords.into()
}

fn smoothen_coordinates<T>(c0: Coord<T>, c1: Coord<T>) -> (Coord<T>, Coord<T>)
where
    T: CoordFloat + Mul<T> + FromPrimitive,
{
    let q = coord! {
        x: (T::from(0.75).unwrap() * c0.x) + (T::from(0.25).unwrap() * c1.x),
        y: (T::from(0.75).unwrap() * c0.y) + (T::from(0.25).unwrap() * c1.y),
    };
    let r = coord! {
        x: (T::from(0.25).unwrap() * c0.x) + (T::from(0.75).unwrap() * c1.x),
        y: (T::from(0.25).unwrap() * c0.y) + (T::from(0.75).unwrap() * c1.y),
    };
    (q, r)
}

#[cfg(test)]
mod test {
    use crate::ChaikinSmoothing;
    use crate::{Geometry, LineString, Point, Polygon};

    #[test]
    fn geometry() {
        // Test implemented geometry
        let ls = LineString::from(vec![(3.0, 0.0), (6.0, 3.0), (3.0, 6.0), (0.0, 3.0)]);
        let ls_geo: Geometry = ls.into();
        let ls_geo_out = ls_geo.chaikin_smoothing(1);
        let ls_out: LineString = ls_geo_out.try_into().unwrap();
        assert_eq!(
            ls_out,
            LineString::from(vec![
                (3.0, 0.0),
                (3.75, 0.75),
                (5.25, 2.25),
                (5.25, 3.75),
                (3.75, 5.25),
                (2.25, 5.25),
                (0.75, 3.75),
                (0.0, 3.0),
            ])
        );

        // Test unimplemented geometry
        let pt = Point::from((3.0, 0.0));
        let pt_geo: Geometry = pt.into();
        let pt_geo_out = pt_geo.chaikin_smoothing(1);
        let pt_out: Point = pt_geo_out.try_into().unwrap();
        assert_eq!(pt_out, Point::from((3.0, 0.0)));
    }

    #[test]
    fn linestring_open() {
        let ls = LineString::from(vec![(3.0, 0.0), (6.0, 3.0), (3.0, 6.0), (0.0, 3.0)]);
        let ls_out = ls.chaikin_smoothing(1);
        assert_eq!(
            ls_out,
            LineString::from(vec![
                (3.0, 0.0),
                (3.75, 0.75),
                (5.25, 2.25),
                (5.25, 3.75),
                (3.75, 5.25),
                (2.25, 5.25),
                (0.75, 3.75),
                (0.0, 3.0),
            ])
        );
    }

    #[test]
    fn linestring_closed() {
        let ls = LineString::from(vec![
            (3.0, 0.0),
            (6.0, 3.0),
            (3.0, 6.0),
            (0.0, 3.0),
            (3.0, 0.0),
        ]);
        let ls_out = ls.chaikin_smoothing(1);
        assert_eq!(
            ls_out,
            LineString::from(vec![
                (3.75, 0.75),
                (5.25, 2.25),
                (5.25, 3.75),
                (3.75, 5.25),
                (2.25, 5.25),
                (0.75, 3.75),
                (0.75, 2.25),
                (2.25, 0.75),
                (3.75, 0.75)
            ])
        );
    }

    #[test]
    fn polygon() {
        let poly = Polygon::new(
            LineString::from(vec![
                (3.0, 0.0),
                (6.0, 3.0),
                (3.0, 6.0),
                (0.0, 3.0),
                (3.0, 0.0),
            ]),
            vec![],
        );
        let poly_out = poly.chaikin_smoothing(1);
        assert_eq!(
            poly_out.exterior(),
            &LineString::from(vec![
                (3.75, 0.75),
                (5.25, 2.25),
                (5.25, 3.75),
                (3.75, 5.25),
                (2.25, 5.25),
                (0.75, 3.75),
                (0.75, 2.25),
                (2.25, 0.75),
                (3.75, 0.75)
            ])
        );
    }
}

#[cfg(test)]
mod hegel_props {
    use super::ChaikinSmoothing;
    use crate::utils::pbt_gens::{monotone_line_strings, star_polygons};
    use crate::{Euclidean, Length};
    use hegel::generators;

    fn iterations(tc: &hegel::TestCase) -> usize {
        tc.draw(generators::integers::<usize>().min_value(1).max_value(4))
    }

    // "This implementation preserves the start and end vertices of an open
    // linestring". The generator's x values strictly increase, so the string is
    // open.
    #[hegel::test]
    fn smoothing_preserves_the_ends_of_an_open_line_string(tc: hegel::TestCase) {
        let line_string = tc.draw(monotone_line_strings(1e3, 12));
        let smoothed = line_string.chaikin_smoothing(iterations(&tc));
        assert_eq!(smoothed.0.first(), line_string.0.first());
        assert_eq!(smoothed.0.last(), line_string.0.last());
    }

    // "Each iteration of the smoothing doubles the number of vertices of the
    // geometry": an open string of n coordinates yields the 2(n-1) midpoints
    // plus its two retained ends.
    #[hegel::test]
    fn each_iteration_doubles_an_open_line_strings_vertices(tc: hegel::TestCase) {
        let line_string = tc.draw(monotone_line_strings(1e3, 12));
        let n = line_string.0.len();
        assert_eq!(line_string.chaikin_smoothing(1).0.len(), 2 * n);
    }

    // Every new vertex is a convex combination of two neighbours, so smoothing
    // cannot leave the convex hull of the input — in particular it cannot
    // lengthen a closed ring's perimeter.
    #[hegel::test]
    fn smoothing_a_ring_never_lengthens_it(tc: hegel::TestCase) {
        let ring = tc.draw(star_polygons()).exterior().clone();
        let smoothed = ring.chaikin_smoothing(iterations(&tc));
        let before = Euclidean.length(&ring);
        let after = Euclidean.length(&smoothed);
        assert!(
            after <= before * (1.0 + 1e-9),
            "smoothing lengthened the ring: {before} -> {after}"
        );
    }

    // Smoothing "smoothes the corner between start and end of a closed
    // linestring", so a closed ring comes back closed.
    #[hegel::test]
    fn smoothing_keeps_a_closed_ring_closed(tc: hegel::TestCase) {
        let ring = tc.draw(star_polygons()).exterior().clone();
        assert!(ring.chaikin_smoothing(iterations(&tc)).is_closed());
    }

    // Zero iterations is no smoothing at all.
    #[hegel::test]
    fn zero_iterations_leaves_the_geometry_alone(tc: hegel::TestCase) {
        let line_string = tc.draw(monotone_line_strings(1e3, 12));
        assert_eq!(line_string.chaikin_smoothing(0), line_string);
    }
}
