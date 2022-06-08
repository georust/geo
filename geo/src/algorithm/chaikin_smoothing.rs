use std::ops::Mul;

use num_traits::FromPrimitive;

use crate::{coord, CoordFloat, Coordinate, LineString, MultiLineString, MultiPolygon, Polygon};

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

fn smoothen_linestring<T>(linestring: &LineString<T>) -> LineString<T>
where
    T: CoordFloat + Mul<T> + FromPrimitive,
{
    let mut out_coords: Vec<_> = Vec::with_capacity(linestring.0.len() * 2);

    if let (Some(first), Some(last)) = (linestring.0.first(), linestring.0.last()) {
        if first != last {
            // preserve start coordinate when the linestring is open
            out_coords.push(*first);
        }
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

fn smoothen_coordinates<T>(c0: Coordinate<T>, c1: Coordinate<T>) -> (Coordinate<T>, Coordinate<T>)
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
    use crate::{LineString, Polygon};

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
