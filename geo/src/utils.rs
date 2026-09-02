//! Internal utility functions, types, and data structures.

use crate::GeoNum;
use geo_types::{Coord, CoordFloat};
use num_traits::FromPrimitive;

/// Partition a mutable slice in-place so that it contains all elements for
/// which `predicate(e)` is `true`, followed by all elements for which
/// `predicate(e)` is `false`. Returns sub-slices to all predicated and
/// non-predicated elements, respectively.
///
/// https://github.com/llogiq/partition/blob/master/src/lib.rs
pub fn partition_slice<T, P>(data: &mut [T], predicate: P) -> (&mut [T], &mut [T])
where
    P: Fn(&T) -> bool,
{
    let len = data.len();
    if len == 0 {
        return (&mut [], &mut []);
    }
    let (mut l, mut r) = (0, len - 1);
    loop {
        while l < len && predicate(&data[l]) {
            l += 1;
        }
        while r > 0 && !predicate(&data[r]) {
            r -= 1;
        }
        if l >= r {
            return data.split_at_mut(l);
        }
        data.swap(l, r);
    }
}

pub enum EitherIter<I1, I2> {
    A(I1),
    B(I2),
}

impl<I1, I2> ExactSizeIterator for EitherIter<I1, I2>
where
    I1: ExactSizeIterator,
    I2: ExactSizeIterator<Item = I1::Item>,
{
    #[inline]
    fn len(&self) -> usize {
        match self {
            EitherIter::A(i1) => i1.len(),
            EitherIter::B(i2) => i2.len(),
        }
    }
}

impl<T, I1, I2> Iterator for EitherIter<I1, I2>
where
    I1: Iterator<Item = T>,
    I2: Iterator<Item = T>,
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            EitherIter::A(iter) => iter.next(),
            EitherIter::B(iter) => iter.next(),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            EitherIter::A(iter) => iter.size_hint(),
            EitherIter::B(iter) => iter.size_hint(),
        }
    }
}

// The Rust standard library has `max` for `Ord`, but not for `PartialOrd`
pub fn partial_max<T: PartialOrd>(a: T, b: T) -> T {
    if a > b { a } else { b }
}

// The Rust standard library has `min` for `Ord`, but not for `PartialOrd`
pub fn partial_min<T: PartialOrd>(a: T, b: T) -> T {
    if a < b { a } else { b }
}

use std::cmp::Ordering;

/// Compare two coordinates lexicographically: first by the
/// x coordinate, and break ties with the y coordinate.
#[inline]
pub fn lex_cmp<T: GeoNum>(p: &Coord<T>, q: &Coord<T>) -> Ordering {
    p.x.total_cmp(&q.x).then(p.y.total_cmp(&q.y))
}

/// Compute index of the least point in slice. Comparison is
/// done using [`lex_cmp`].
///
/// Should only be called on a non-empty slice with no `nan`
/// coordinates.
pub fn least_index<T: GeoNum>(pts: &[Coord<T>]) -> usize {
    pts.iter()
        .enumerate()
        .min_by(|(_, p), (_, q)| lex_cmp(p, q))
        .unwrap()
        .0
}

/// Normalize a longitude to coordinate to ensure it's within [-180,180]
pub fn normalize_longitude<T: CoordFloat + FromPrimitive>(coord: T) -> T {
    let one_eighty = T::from(180.0f64).unwrap();
    let three_sixty = T::from(360.0f64).unwrap();
    let five_forty = T::from(540.0f64).unwrap();

    ((coord + five_forty) % three_sixty) - one_eighty
}

/// Generators shared by the hegel property tests throughout the crate.
///
/// Geometry types are foreign to hegel, so nothing here can implement
/// `PrettyPrintable`: each generator ends in `print_as_debug()`, and the draws
/// it makes internally are silent so that a counterexample reports the
/// assembled geometry rather than the floats it was built from.
#[cfg(test)]
pub(crate) mod pbt_gens {
    use crate::{
        Coord, Geometry, GeometryCollection, Line, LineString, MultiLineString, MultiPoint,
        MultiPolygon, Point, Polygon, Rect, Triangle, coord,
    };
    use hegel::compose;
    use hegel::generators::{self, Generator, PrintableGenerator};
    use std::f64::consts::TAU;

    fn float(min: f64, max: f64) -> impl PrintableGenerator<f64> {
        generators::floats::<f64>().min_value(min).max_value(max)
    }

    fn draw_coord(tc: &hegel::TestCase, max_coord: f64) -> Coord<f64> {
        let (x, y) = tc.draw_silent(generators::tuples!(
            float(-max_coord, max_coord),
            float(-max_coord, max_coord)
        ));
        coord! { x: x, y: y }
    }

    /// `n` angular gaps summing to a full turn, each strictly less than half a
    /// turn.
    ///
    /// Raw gaps come from `[0.5, 0.99]`. With `n >= 3` the gaps other than the
    /// largest sum to at least `1.0 > 0.99`, so after normalisation no gap
    /// reaches `pi`. Vertices at the cumulative angles are then in strictly
    /// increasing angular order with no gap spanning a half turn, which makes
    /// the ring star-shaped about its centre and hence simple.
    fn draw_angular_gaps(tc: &hegel::TestCase, n: usize) -> Vec<f64> {
        let raw: Vec<f64> =
            tc.draw_silent(generators::vecs(float(0.5, 0.99)).min_size(n).max_size(n));
        let total: f64 = raw.iter().sum();
        raw.into_iter().map(|gap| gap / total * TAU).collect()
    }

    fn ring(centre: Coord<f64>, gaps: &[f64], radii: &[f64]) -> LineString<f64> {
        let mut angle = 0.0_f64;
        LineString::new(
            gaps.iter()
                .zip(radii)
                .map(|(gap, radius)| {
                    angle += gap;
                    coord! {
                        x: centre.x + radius * angle.cos(),
                        y: centre.y + radius * angle.sin(),
                    }
                })
                .collect(),
        )
    }

    /// A simple star-shaped ring about `centre` with 3 to 24 vertices at radii
    /// in `[min_radius, max_radius]`.
    fn draw_star_ring(
        tc: &hegel::TestCase,
        centre: Coord<f64>,
        min_radius: f64,
        max_radius: f64,
    ) -> LineString<f64> {
        let n = tc.draw_silent(generators::integers::<usize>().min_value(3).max_value(24));
        let gaps = draw_angular_gaps(tc, n);
        let radii: Vec<f64> = tc.draw_silent(
            generators::vecs(float(min_radius, max_radius))
                .min_size(n)
                .max_size(n),
        );
        ring(centre, &gaps, &radii)
    }

    /// Simple polygons without holes.
    ///
    /// The centre (`±1e3`) and radius (`0.5..=1e3`) bounds keep areas and
    /// perimeters well clear of f64 rounding noise, which is what the
    /// tolerances in the area- and length-comparison properties are sized
    /// against; they are not claims about the library's domain.
    pub(crate) fn star_polygons() -> impl PrintableGenerator<Polygon<f64>> {
        compose!(|tc| {
            let centre = draw_coord(tc, 1e3);
            Polygon::new(draw_star_ring(tc, centre, 0.5, 1e3), vec![])
        })
        .print_as_debug()
    }

    /// The pieces of a cyclic polygon: the exterior ring and the radius of its
    /// inscribed circle.
    ///
    /// All vertices share one radius, so the vertices lie on a circle in
    /// angular order and the ring is convex. The inscribed circle then has
    /// radius `radius * min(cos(gap / 2))`, the least distance from the centre
    /// to a chord.
    fn draw_cyclic_ring(tc: &hegel::TestCase, centre: Coord<f64>) -> (LineString<f64>, f64) {
        let radius = tc.draw_silent(float(1.0, 1e3));
        let n = tc.draw_silent(generators::integers::<usize>().min_value(3).max_value(24));
        let gaps = draw_angular_gaps(tc, n);
        let inscribed = gaps
            .iter()
            .map(|gap| radius * (gap / 2.0).cos())
            .fold(f64::INFINITY, f64::min);
        (ring(centre, &gaps, &vec![radius; n]), inscribed)
    }

    /// Convex polygons without holes.
    pub(crate) fn convex_polygons() -> impl PrintableGenerator<Polygon<f64>> {
        compose!(|tc| {
            let centre = draw_coord(tc, 1e3);
            Polygon::new(draw_cyclic_ring(tc, centre).0, vec![])
        })
        .print_as_debug()
    }

    /// Polygons with 0 to 4 holes, valid by construction.
    ///
    /// The exterior is convex, and each hole is a star ring confined to its own
    /// cell of a 2x2 grid inscribed in the exterior's inscribed circle, so
    /// holes lie strictly inside the exterior and never meet one another —
    /// what `InvalidPolygon` requires of interior rings.
    pub(crate) fn polygons_with_holes() -> impl PrintableGenerator<Polygon<f64>> {
        compose!(|tc| {
            let centre = draw_coord(tc, 1e3);
            let (exterior, inscribed) = draw_cyclic_ring(tc, centre);
            // The largest axis-aligned square inside the inscribed circle has
            // side `inscribed * sqrt(2)`, so a 2x2 grid of `inscribed * 0.7`
            // cells fits with room to spare.
            let cell = inscribed * 0.7;
            let cells = tc.draw_silent(generators::subsequences(vec![
                (0, 0),
                (0, 1),
                (1, 0),
                (1, 1),
            ]));
            let interiors = cells
                .into_iter()
                .map(|(i, j)| {
                    let hole_centre = coord! {
                        x: centre.x + (i as f64 - 0.5) * cell,
                        y: centre.y + (j as f64 - 0.5) * cell,
                    };
                    // Radii below half the cell size keep each hole inside its
                    // own cell.
                    draw_star_ring(tc, hole_centre, cell * 0.05, cell * 0.4)
                })
                .collect();
            Polygon::new(exterior, interiors)
        })
        .print_as_debug()
    }

    /// Multi polygons of 1 to 6 pairwise disjoint star polygons.
    ///
    /// Members sit on a grid whose spacing is three times the maximum radius,
    /// so no two can touch — the non-overlap condition
    /// `InvalidMultiPolygon` checks.
    pub(crate) fn disjoint_multi_polygons() -> impl PrintableGenerator<MultiPolygon<f64>> {
        compose!(|tc| {
            let origin = draw_coord(tc, 1e3);
            let radius = tc.draw_silent(float(0.5, 1e2));
            let cells = tc.draw_silent(
                generators::subsequences(vec![(0, 0), (0, 1), (1, 0), (1, 1), (2, 0), (2, 1)])
                    .min_size(1),
            );
            MultiPolygon::new(
                cells
                    .into_iter()
                    .map(|(i, j)| {
                        let centre = coord! {
                            x: origin.x + i as f64 * 3.0 * radius,
                            y: origin.y + j as f64 * 3.0 * radius,
                        };
                        Polygon::new(draw_star_ring(tc, centre, radius * 0.2, radius), vec![])
                    })
                    .collect(),
            )
        })
        .print_as_debug()
    }

    /// Finite coordinates with both components in `[-max_coord, max_coord]`.
    pub(crate) fn coords(max_coord: f64) -> impl PrintableGenerator<Coord<f64>> {
        compose!(|tc| { draw_coord(tc, max_coord) }).print_as_debug()
    }

    /// Coordinates on the integer grid `[-64, 64]^2`.
    ///
    /// Such coordinates and every difference and product of two of them are
    /// exact in f64, so duplicate and exactly-collinear points are common and
    /// a property can compare against exact arithmetic.
    pub(crate) fn grid_coords() -> impl PrintableGenerator<Coord<f64>> {
        compose!(|tc| {
            let component = || generators::integers::<i8>().min_value(-64).max_value(64);
            let (x, y) = tc.draw_silent(generators::tuples!(component(), component()));
            coord! { x: x as f64, y: y as f64 }
        })
        .print_as_debug()
    }

    /// Line strings of up to `max_len` finite coordinates, possibly empty and
    /// possibly with repeated coordinates.
    pub(crate) fn line_strings(
        max_coord: f64,
        max_len: usize,
    ) -> impl PrintableGenerator<LineString<f64>> {
        compose!(|tc| {
            let n = tc.draw_silent(generators::integers::<usize>().max_value(max_len));
            LineString::new((0..n).map(|_| draw_coord(tc, max_coord)).collect())
        })
        .print_as_debug()
    }

    /// Line strings of 2 to `max_len` coordinates whose x values strictly
    /// increase.
    ///
    /// Consecutive segments therefore have disjoint x ranges apart from their
    /// shared endpoint, so the string is simple and never retraces itself —
    /// what set operations on 1-D geometry need to preserve length.
    pub(crate) fn monotone_line_strings(
        max_coord: f64,
        max_len: usize,
    ) -> impl PrintableGenerator<LineString<f64>> {
        compose!(|tc| {
            let n = tc.draw_silent(
                generators::integers::<usize>()
                    .min_value(2)
                    .max_value(max_len),
            );
            let mut x = tc.draw_silent(float(-max_coord, max_coord));
            let step = max_coord / max_len as f64;
            LineString::new(
                (0..n)
                    .map(|i| {
                        if i > 0 {
                            x += tc.draw_silent(float(step * 0.01, step));
                        }
                        coord! { x: x, y: tc.draw_silent(float(-max_coord, max_coord)) }
                    })
                    .collect(),
            )
        })
        .print_as_debug()
    }

    /// Any of the ten `Geometry` variants, with finite coordinates bounded by
    /// `max_coord`. Rings are arbitrary rather than simple, so this covers
    /// properties that hold for any geometry rather than only valid ones.
    pub(crate) fn geometries(max_coord: f64) -> impl PrintableGenerator<Geometry<f64>> {
        fn ring(tc: &hegel::TestCase, max_coord: f64) -> LineString<f64> {
            tc.draw_silent(line_strings(max_coord, 8))
        }
        fn point(tc: &hegel::TestCase, max_coord: f64) -> Point<f64> {
            Point::from(draw_coord(tc, max_coord))
        }
        fn count(tc: &hegel::TestCase) -> usize {
            tc.draw_silent(generators::integers::<usize>().max_value(3))
        }
        hegel::one_of!(
            compose!(|tc| { Geometry::Point(point(tc, max_coord)) }),
            compose!(|tc| {
                Geometry::Line(Line::new(
                    draw_coord(tc, max_coord),
                    draw_coord(tc, max_coord),
                ))
            }),
            compose!(|tc| { Geometry::LineString(tc.draw_silent(line_strings(max_coord, 12))) }),
            compose!(|tc| {
                Geometry::Polygon(Polygon::new(
                    ring(tc, max_coord),
                    (0..count(tc)).map(|_| ring(tc, max_coord)).collect(),
                ))
            }),
            compose!(|tc| {
                Geometry::MultiPoint(MultiPoint::new(
                    (0..count(tc)).map(|_| point(tc, max_coord)).collect(),
                ))
            }),
            compose!(|tc| {
                Geometry::MultiLineString(MultiLineString::new(
                    (0..count(tc)).map(|_| ring(tc, max_coord)).collect(),
                ))
            }),
            compose!(|tc| {
                Geometry::MultiPolygon(MultiPolygon::new(
                    (0..count(tc))
                        .map(|_| Polygon::new(ring(tc, max_coord), vec![]))
                        .collect(),
                ))
            }),
            compose!(|tc| {
                Geometry::Rect(Rect::new(
                    draw_coord(tc, max_coord),
                    draw_coord(tc, max_coord),
                ))
            }),
            compose!(|tc| {
                Geometry::Triangle(Triangle::new(
                    draw_coord(tc, max_coord),
                    draw_coord(tc, max_coord),
                    draw_coord(tc, max_coord),
                ))
            }),
            compose!(|tc| {
                Geometry::GeometryCollection(GeometryCollection::new_from(
                    (0..count(tc))
                        .map(|_| Geometry::Point(point(tc, max_coord)))
                        .collect(),
                ))
            }),
        )
        .print_as_debug()
    }
}

#[cfg(test)]
mod test {
    use super::{partial_max, partial_min};

    #[test]
    fn test_partial_max() {
        assert_eq!(5, partial_max(5, 4));
        assert_eq!(5, partial_max(5, 5));
    }

    #[test]
    fn test_partial_min() {
        assert_eq!(4, partial_min(5, 4));
        assert_eq!(4, partial_min(4, 4));
    }
}
