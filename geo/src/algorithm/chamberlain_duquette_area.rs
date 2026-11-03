use crate::geometry::*;
use crate::{CoordFloat, EQUATORIAL_EARTH_RADIUS};

/// Calculate the signed approximate geodesic area of a `Geometry`.
///
/// # Units
///
/// - return value: meters²
///
/// # Edges
///
/// Each edge of a ring is taken the short way around in longitude, as [`GeodesicArea`] takes it,
/// so an edge from 179° to -179° crosses the antimeridian and is 2° wide. An edge whose change in
/// longitude computes to exactly 180° keeps the direction it is written in, as a rhumb line does
/// ([`GeodesicArea`] takes such an edge over a pole). An edge is taken as written if it runs along
/// a parallel by a whole number of turns, such as one from -180° to 180°, all the way round it (so
/// a box from -180° to 180° is the whole band of latitude); if it runs along a pole; or if an end
/// is more than a million degrees out, which isn't a longitude.
///
/// So a polygon with an edge wider than 180°, such as a box from -100° to 100° written with only
/// its corners, is the narrower box the other way round (from 100° to 260°), with the opposite
/// sign; with more vertices along its edges, it is the wider box. A [`Rect`] is always the box
/// from its minimum longitude east to its maximum, so one between 180° and 360° wide measures
/// differently from its `to_polygon()`.
///
/// A ring that winds around a pole splits the Earth in two: a parallel of latitude written east
/// or west all the way around, say, or a triangle with one edge wider than 180°. Its signed area
/// is that of the smaller side, positive when that side is on the ring's left (as for a
/// counter-clockwise ring) and negative when it's on the ring's right, so the result is in
/// (-earth/2, earth/2]: a ring along the equator gives half the Earth, positive, and a ring
/// winding around more than once gives the total for its turns reduced into that range. A ring
/// that doesn't wind around a pole gives the side with neither pole in it, which can be more than
/// half the Earth. A pole is best written as an edge along it rather than as a single vertex.
///
/// The unsigned area is the absolute value of the signed area (unlike [`GeodesicArea`]'s, which
/// is the area on the ring's left). Holes are subtracted with their signed area, so they should
/// wind the same way as their exterior: a hole wound the other way (as in OGC Simple Features and
/// RFC 7946) is added instead.
///
/// [`GeodesicArea`]: crate::GeodesicArea
///
/// # References
///
/// * Robert. G. Chamberlain and William H. Duquette, "Some Algorithms for Polygons on a Sphere",
///
///   JPL Publication 07-03, Jet Propulsion Laboratory, Pasadena, CA, June 2007 <https://trs.jpl.nasa.gov/handle/2014/41271>
///
/// # Examples
///
/// ```
/// use geo::{polygon, Polygon};
/// use geo::chamberlain_duquette_area::ChamberlainDuquetteArea;
///
/// // The O2 in London
/// let mut polygon: Polygon<f64> = polygon![
///     (x: 0.00388383, y: 51.501574),
///     (x: 0.00538587, y: 51.502278),
///     (x: 0.00553607, y: 51.503299),
///     (x: 0.00467777, y: 51.504181),
///     (x: 0.00327229, y: 51.504435),
///     (x: 0.00187754, y: 51.504168),
///     (x: 0.00087976, y: 51.503380),
///     (x: 0.00107288, y: 51.502324),
///     (x: 0.00185608, y: 51.501770),
///     (x: 0.00388383, y: 51.501574),
/// ];
///
/// // 78,478 meters²
/// assert_eq!(78_478., polygon.chamberlain_duquette_unsigned_area().round());
/// assert_eq!(78_478., polygon.chamberlain_duquette_signed_area().round());
///
/// polygon.exterior_mut(|line_string| {
///     line_string.0.reverse();
/// });
///
/// assert_eq!(78_478., polygon.chamberlain_duquette_unsigned_area().round());
/// assert_eq!(-78_478., polygon.chamberlain_duquette_signed_area().round());
/// ```
pub trait ChamberlainDuquetteArea<T>
where
    T: CoordFloat,
{
    fn chamberlain_duquette_signed_area(&self) -> T;

    fn chamberlain_duquette_unsigned_area(&self) -> T;
}

impl<T> ChamberlainDuquetteArea<T> for Polygon<T>
where
    T: CoordFloat,
{
    fn chamberlain_duquette_signed_area(&self) -> T {
        self.interiors()
            .iter()
            .fold(ring_area(self.exterior()), |total, next| {
                total - ring_area(next)
            })
    }

    fn chamberlain_duquette_unsigned_area(&self) -> T {
        self.chamberlain_duquette_signed_area().abs()
    }
}

fn ring_area<T>(coords: &LineString<T>) -> T
where
    T: CoordFloat,
{
    let mut total = T::zero();
    let coords_len = coords.0.len();
    let half_turn = T::from(180).unwrap();
    let mut as_written = false;

    if coords_len > 2 {
        for i in 0..coords_len {
            let (lower_index, middle_index, upper_index) = if i == coords_len - 2 {
                // i = N-2
                (coords_len - 2, coords_len - 1, 0)
            } else if i == coords_len - 1 {
                // i = N-1
                (coords_len - 1, 0, 1)
            } else {
                // i = 0 to N-3
                (i, i + 1, i + 2)
            };
            let p1 = coords[lower_index];
            let p2 = coords[middle_index];
            let p3 = coords[upper_index];
            // This sum takes each edge's change in longitude as written. A ring with an edge
            // written across more than 180° (here, the edge from p2 to p3) may need edges taken
            // the short way around instead; if not, the sum goes on as written. The span is
            // compared in degrees, where an edge written across exactly 180° is exactly 180° (in
            // radians it can round to just over π).
            if !as_written && (p3.x - p2.x).abs() > half_turn {
                match short_way_ring_area(&coords.0) {
                    Some(area) => return area,
                    None => as_written = true,
                }
            }
            total = total + (p3.x.to_radians() - p1.x.to_radians()) * p2.y.to_radians().sin();
        }

        total = total
            * T::from(EQUATORIAL_EARTH_RADIUS).unwrap()
            * T::from(EQUATORIAL_EARTH_RADIUS).unwrap()
            / T::from(-2).unwrap();
    }
    total
}

/// The area of a ring with an edge written across more than 180° of longitude, taking edges the
/// short way around; or `None` if none is to be (each such edge spans whole turns or runs along a
/// pole) or if one can't be (an end is too far out to be a longitude), when the ring is taken as
/// written.
///
/// It is the sum in `ring_area` regrouped by edge: each edge's change in longitude times the sum
/// of the sines of its ends' latitudes. The sines are taken relative to the first vertex's, which
/// keeps the terms as small as the ring is wherever it lies; what that takes off is the first
/// vertex's sine times the changes in longitude, which add up to whole turns, so it goes back on
/// exactly.
fn short_way_ring_area<T>(coords: &[Coord<T>]) -> Option<T>
where
    T: CoordFloat,
{
    let last = *coords.last()?;
    let mut a = last;
    let mut any_reduced = false;
    for &b in coords {
        any_reduced |= short_span(a, b)?.1 != 0;
        a = b;
    }
    if !any_reduced {
        return None;
    }

    let sin_0 = coords[0].y.to_radians().sin();
    let mut total = T::zero();
    let mut turns_taken_off = 0_i64;
    let mut a = last;
    let mut sin_a = a.y.to_radians().sin() - sin_0;
    for &b in coords {
        let sin_b = b.y.to_radians().sin() - sin_0;
        let (span, turns) = short_span(a, b)?;
        total = total + span.to_radians() * (sin_a + sin_b);
        turns_taken_off += turns;
        a = b;
        sin_a = sin_b;
    }
    // The changes in longitude as written add up to nothing, so the ring winds around by the
    // turns taken off them, the other way.
    let winding = T::from(-turns_taken_off)?;
    let tau = T::from(std::f64::consts::TAU).unwrap();
    total = total + (sin_0 + sin_0) * tau * winding;
    let r2 = radius_squared::<T>();
    total = total * r2 / T::from(-2).unwrap();

    // That is the area between the ring and the equator. A ring winding around a pole encloses
    // the area between it and the pole instead, on its left: 2πR² more for each turn east (less
    // for each turn west). Of the two sides it splits the Earth into, the smaller is given,
    // negative when it's on the ring's right (and positive for halves, such as a ring along the
    // equator: the result is in (-earth/2, earth/2]).
    if winding != T::zero() {
        let half_earth = tau * r2;
        let earth = half_earth + half_earth;
        total = total + half_earth * winding;
        total = total - earth * (total / earth).round();
        if total <= -half_earth {
            total = total + earth;
        }
    }
    Some(total)
}

/// The change in longitude from `a` to `b` (degrees) taken the short way around, with the whole
/// turns taken off the change as written to get it; or `None` if an end is too far out to be a
/// longitude.
///
/// A change of 180° plus whole turns keeps the direction it is written in. One along a parallel
/// by whole turns (such as from -180° to 180°, all the way round it) or along a pole (where any
/// change in longitude goes nowhere) is kept as written.
fn short_span<T>(a: Coord<T>, b: Coord<T>) -> Option<(T, i64)>
where
    T: CoordFloat,
{
    let half_turn = T::from(180).unwrap();
    let full_turn = half_turn + half_turn;
    let written = b.x - a.x;
    if written.abs() <= half_turn || (a.y == b.y && a.y.abs() == T::from(90).unwrap()) {
        return Some((written, 0));
    }
    // Ends further out than this aren't longitudes: the ring is taken as written, as it always
    // was, rather than reduced to a plausible area.
    let limit = T::from(1e6).unwrap();
    if !(a.x.abs() <= limit && b.x.abs() <= limit) {
        return None;
    }
    let mut span = written - full_turn * (written / full_turn).round();
    if span == T::zero() && a.y == b.y {
        return Some((written, 0));
    }
    if span.abs() == half_turn {
        span = half_turn * written.signum();
    }
    let turns = ((written - span) / full_turn).round().to_i64()?;
    Some((span, turns))
}

/// The square of the Earth's equatorial radius, in meters².
fn radius_squared<T>() -> T
where
    T: CoordFloat,
{
    let radius = T::from(EQUATORIAL_EARTH_RADIUS).unwrap();
    radius * radius
}

/// Generate a `ChamberlainDuquetteArea` implementation where the result is zero.
macro_rules! zero_impl {
    ($type:ident) => {
        impl<T> ChamberlainDuquetteArea<T> for $type<T>
        where
            T: CoordFloat,
        {
            fn chamberlain_duquette_signed_area(&self) -> T {
                T::zero()
            }

            fn chamberlain_duquette_unsigned_area(&self) -> T {
                T::zero()
            }
        }
    };
}

/// Generate a `ChamberlainDuquetteArea` implementation which delegates to the `Polygon`
/// implementation.
macro_rules! to_polygon_impl {
    ($type:ident) => {
        impl<T> ChamberlainDuquetteArea<T> for $type<T>
        where
            T: CoordFloat,
        {
            fn chamberlain_duquette_signed_area(&self) -> T {
                self.to_polygon().chamberlain_duquette_signed_area()
            }

            fn chamberlain_duquette_unsigned_area(&self) -> T {
                self.to_polygon().chamberlain_duquette_unsigned_area()
            }
        }
    };
}

/// Generate a `ChamberlainDuquetteArea` implementation which calculates the area for each of its
/// sub-components and sums them up.
macro_rules! sum_impl {
    ($type:ident) => {
        impl<T> ChamberlainDuquetteArea<T> for $type<T>
        where
            T: CoordFloat,
        {
            fn chamberlain_duquette_signed_area(&self) -> T {
                self.iter().fold(T::zero(), |total, next| {
                    total + next.chamberlain_duquette_signed_area()
                })
            }

            fn chamberlain_duquette_unsigned_area(&self) -> T {
                self.iter().fold(T::zero(), |total, next| {
                    total + next.chamberlain_duquette_unsigned_area()
                })
            }
        }
    };
}

zero_impl!(Point);
zero_impl!(Line);
zero_impl!(LineString);
zero_impl!(MultiPoint);
zero_impl!(MultiLineString);
to_polygon_impl!(Triangle);
sum_impl!(GeometryCollection);
sum_impl!(MultiPolygon);

/// A `Rect` spans from its minimum longitude east to its maximum, however wide, so it is measured
/// directly rather than as a polygon (whose edges wider than 180° are taken the short way around).
impl<T> ChamberlainDuquetteArea<T> for Rect<T>
where
    T: CoordFloat,
{
    fn chamberlain_duquette_signed_area(&self) -> T {
        self.width().to_radians()
            * (self.max().y.to_radians().sin() - self.min().y.to_radians().sin())
            * radius_squared()
    }

    fn chamberlain_duquette_unsigned_area(&self) -> T {
        self.chamberlain_duquette_signed_area().abs()
    }
}

impl<T> ChamberlainDuquetteArea<T> for Geometry<T>
where
    T: CoordFloat,
{
    crate::geometry_delegate_impl! {
        fn chamberlain_duquette_signed_area(&self) -> T;
        fn chamberlain_duquette_unsigned_area(&self) -> T;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::polygon;

    #[test]
    fn test_negative() {
        let polygon = polygon![
            (x: 125., y: -15.),
            (x: 144., y: -15.),
            (x: 154., y: -27.),
            (x: 148., y: -39.),
            (x: 130., y: -33.),
            (x: 117., y: -37.),
            (x: 113., y: -22.),
            (x: 125., y: -15.),
        ];
        assert_relative_eq!(
            -7766240997209.013,
            polygon.chamberlain_duquette_signed_area()
        );
    }

    #[test]
    fn test_positive() {
        let polygon = polygon![
            (x: 125., y: -15.),
            (x: 113., y: -22.),
            (x: 117., y: -37.),
            (x: 130., y: -33.),
            (x: 148., y: -39.),
            (x: 154., y: -27.),
            (x: 144., y: -15.),
            (x: 125., y: -15.),
        ];
        assert_relative_eq!(
            7766240997209.013,
            polygon.chamberlain_duquette_signed_area()
        );
    }

    #[test]
    fn test_holes() {
        let poly = polygon![
            exterior: [
                (x: 0., y: 0.),
                (x: 10., y: 0.),
                (x: 10., y: 10.),
                (x: 0., y: 10.),
                (x: 0., y: 0.)
            ],
            interiors: [
                [
                    (x: 1., y: 1.),
                    (x: 2., y: 1.),
                    (x: 2., y: 2.),
                    (x: 1., y: 2.),
                    (x: 1., y: 1.),
                ],
                [
                    (x: 5., y: 5.),
                    (x: 6., y: 5.),
                    (x: 6., y: 6.),
                    (x: 5., y: 6.),
                    (x: 5., y: 5.)
                ],
            ],
        ];
        assert_relative_eq!(1208198651182.4727, poly.chamberlain_duquette_signed_area());
    }

    /// The area of the cap of the sphere north of `lat` (degrees), in meters².
    fn cap_area(lat: f64) -> f64 {
        2. * std::f64::consts::PI
            * EQUATORIAL_EARTH_RADIUS
            * EQUATORIAL_EARTH_RADIUS
            * (1. - lat.to_radians().sin())
    }

    /// A ring along the parallel `lat` through the longitudes `lons`.
    fn parallel(lat: f64, lons: &[f64]) -> Polygon<f64> {
        let ring: Vec<(f64, f64)> = lons.iter().map(|&lon| (lon, lat)).collect();
        Polygon::new(LineString::from(ring), vec![])
    }

    // Closed from 330° back to 0°: 30° east.
    const EAST: [f64; 12] = [
        0., 30., 60., 90., 120., 150., 180., 210., 240., 270., 300., 330.,
    ];
    const EAST_FOLDED: [f64; 13] = [
        0., 30., 60., 90., 120., 150., 180., -150., -120., -90., -60., -30., 0.,
    ];

    #[test]
    fn test_antimeridian() {
        let across = polygon![
            (x: 179., y: 0.),
            (x: -179., y: 0.),
            (x: -179., y: 1.),
            (x: 179., y: 1.),
            (x: 179., y: 0.),
        ];
        let centered = polygon![
            (x: -1., y: 0.),
            (x: 1., y: 0.),
            (x: 1., y: 1.),
            (x: -1., y: 1.),
            (x: -1., y: 0.),
        ];
        let area = centered.chamberlain_duquette_signed_area();
        assert!(area > 0.);
        assert_relative_eq!(
            area,
            across.chamberlain_duquette_signed_area(),
            max_relative = 1e-9
        );
    }

    #[test]
    fn test_edge_spanning_180_degrees_keeps_its_direction() {
        let quarter = polygon![
            (x: 0., y: 10.),
            (x: 90., y: 10.),
            (x: 90., y: 20.),
            (x: 0., y: 20.),
            (x: 0., y: 10.),
        ];
        // From all but the first of these, a span of 180° in radians rounds to just over π.
        for west in [0., 10., -172., -8.] {
            let east = west + 180.;
            let half = polygon![
                (x: west, y: 10.),
                (x: east, y: 10.),
                (x: east, y: 20.),
                (x: west, y: 20.),
                (x: west, y: 10.),
            ];
            assert_relative_eq!(
                2. * quarter.chamberlain_duquette_signed_area(),
                half.chamberlain_duquette_signed_area(),
                max_relative = 1e-9
            );
        }
    }

    #[test]
    fn test_ring_along_equator() {
        // Either way round, it splits the Earth in halves: half the Earth, positive.
        let half_earth =
            2. * std::f64::consts::PI * EQUATORIAL_EARTH_RADIUS * EQUATORIAL_EARTH_RADIUS;
        let mut ring = parallel(0., &EAST);
        assert_relative_eq!(
            half_earth,
            ring.chamberlain_duquette_signed_area(),
            max_relative = 1e-9
        );
        ring.exterior_mut(|line_string| line_string.0.reverse());
        assert_relative_eq!(
            half_earth,
            ring.chamberlain_duquette_signed_area(),
            max_relative = 1e-9
        );
    }

    #[test]
    fn test_ring_around_north_pole() {
        for lons in [&EAST[..], &EAST_FOLDED[..]] {
            let mut ring = parallel(80., lons);
            assert_relative_eq!(
                cap_area(80.),
                ring.chamberlain_duquette_signed_area(),
                max_relative = 1e-9
            );
            ring.exterior_mut(|line_string| line_string.0.reverse());
            assert_relative_eq!(
                -cap_area(80.),
                ring.chamberlain_duquette_signed_area(),
                max_relative = 1e-9
            );
            assert_relative_eq!(
                cap_area(80.),
                ring.chamberlain_duquette_unsigned_area(),
                max_relative = 1e-9
            );
        }
    }

    #[test]
    fn test_ring_around_south_pole() {
        // Going east around the South Pole, the smaller side (the cap south of 80°S) is on the
        // ring's right.
        let mut ring = parallel(-80., &EAST);
        assert_relative_eq!(
            -cap_area(80.),
            ring.chamberlain_duquette_signed_area(),
            max_relative = 1e-9
        );
        ring.exterior_mut(|line_string| line_string.0.reverse());
        assert_relative_eq!(
            cap_area(80.),
            ring.chamberlain_duquette_signed_area(),
            max_relative = 1e-9
        );
    }

    #[test]
    fn test_ring_around_pole_with_hole() {
        let hole = polygon![
            (x: 0., y: 85.),
            (x: 10., y: 85.),
            (x: 10., y: 86.),
            (x: 0., y: 86.),
            (x: 0., y: 85.),
        ];
        let cap = Polygon::new(
            parallel(80., &EAST).exterior().clone(),
            vec![hole.exterior().clone()],
        );
        assert_relative_eq!(
            cap_area(80.) - hole.chamberlain_duquette_signed_area(),
            cap.chamberlain_duquette_signed_area(),
            max_relative = 1e-9
        );
    }

    #[test]
    fn test_ring_winding_twice() {
        // Twice the cap, reduced into (-earth/2, earth/2].
        let earth = 4. * std::f64::consts::PI * EQUATORIAL_EARTH_RADIUS * EQUATORIAL_EARTH_RADIUS;
        let lons: Vec<f64> = (0..24).map(|i| f64::from(i) * 30.).collect();
        assert_relative_eq!(
            2. * cap_area(10.) - earth,
            parallel(10., &lons).chamberlain_duquette_signed_area(),
            max_relative = 1e-9
        );
    }

    #[test]
    fn test_whole_turns_and_pole_edges_taken_as_written() {
        // An edge from -180° to 180° runs all the way round its parallel, and one along a pole
        // goes nowhere whatever its change in longitude: so a box from -180° to 180° is the whole
        // band of latitude, here a cap, written with only its corners or not.
        let as_box = corner_box(-180., 180., 80., 90.);
        assert_relative_eq!(
            cap_area(80.),
            as_box.chamberlain_duquette_signed_area(),
            max_relative = 1e-9
        );
        let world = Rect::new((-180., -90.), (180., 90.));
        assert_relative_eq!(
            world.chamberlain_duquette_signed_area(),
            world.to_polygon().chamberlain_duquette_signed_area(),
            max_relative = 1e-12
        );
        // Everything north of 60°S, closed along the North Pole.
        let north_of_60s = polygon![
            (x: -180., y: -60.),
            (x: -90., y: -60.),
            (x: 0., y: -60.),
            (x: 90., y: -60.),
            (x: 180., y: -60.),
            (x: 180., y: 90.),
            (x: -180., y: 90.),
            (x: -180., y: -60.),
        ];
        assert_relative_eq!(
            cap_area(-60.),
            north_of_60s.chamberlain_duquette_signed_area(),
            max_relative = 1e-9
        );
        // A band closed along a parallel, as when clipped to Web Mercator's limit.
        let clipped = polygon![
            (x: -180., y: -60.),
            (x: -90., y: -60.),
            (x: 0., y: -60.),
            (x: 90., y: -60.),
            (x: 180., y: -60.),
            (x: 180., y: -85.),
            (x: -180., y: -85.),
            (x: -180., y: -60.),
        ];
        assert_relative_eq!(
            cap_area(-60.) - cap_area(-85.),
            clipped.chamberlain_duquette_signed_area(),
            max_relative = 1e-9
        );
        let along = polygon![
            (x: -180., y: 80.),
            (x: -90., y: 80.),
            (x: 0., y: 80.),
            (x: 90., y: 80.),
            (x: 180., y: 80.),
            (x: 180., y: 90.),
            (x: -180., y: 90.),
            (x: -180., y: 80.),
        ];
        assert_relative_eq!(
            cap_area(80.),
            along.chamberlain_duquette_signed_area(),
            max_relative = 1e-9
        );
    }

    /// A box from `west` east to `east` and `south` to `north`, written with only its corners.
    fn corner_box<T: CoordFloat>(west: T, east: T, south: T, north: T) -> Polygon<T> {
        let ring = vec![
            (west, south),
            (east, south),
            (east, north),
            (west, north),
            (west, south),
        ];
        Polygon::new(LineString::from(ring), vec![])
    }

    #[test]
    fn test_rect() {
        // A Rect spans east from its minimum longitude, however wide it is.
        let earth = 4. * std::f64::consts::PI * EQUATORIAL_EARTH_RADIUS * EQUATORIAL_EARTH_RADIUS;
        let world = Rect::new((-180., -90.), (180., 90.));
        assert_relative_eq!(
            earth,
            world.chamberlain_duquette_signed_area(),
            max_relative = 1e-12
        );
        let arctic = Rect::new((-180., 60.), (180., 90.));
        assert_relative_eq!(
            cap_area(60.),
            arctic.chamberlain_duquette_signed_area(),
            max_relative = 1e-12
        );
        let wide = Rect::new((-100., 0.), (100., 10.));
        let half = Rect::new((0., 0.), (100., 10.));
        assert_relative_eq!(
            2. * half.chamberlain_duquette_signed_area(),
            wide.chamberlain_duquette_signed_area(),
            max_relative = 1e-12
        );
        // The same as the polygon it describes, when no edge is wider than 180°.
        assert_relative_eq!(
            half.to_polygon().chamberlain_duquette_signed_area(),
            half.chamberlain_duquette_signed_area(),
            max_relative = 1e-12
        );
    }

    #[test]
    fn test_edge_spanning_540_degrees_keeps_its_direction() {
        // 540° east is taken as 180° east, the way it is written, rather than 180° west.
        assert_relative_eq!(
            corner_box(0., 180., 10., 20.).chamberlain_duquette_signed_area(),
            corner_box(-180., 360., 10., 20.).chamberlain_duquette_signed_area(),
            max_relative = 1e-12
        );
    }

    #[test]
    fn test_ends_written_whole_turns_apart() {
        // A corner written whole turns further round gives the same box.
        let near = corner_box(0., 10., 10., 20.).chamberlain_duquette_signed_area();
        for turns in [1., 1e3] {
            let far = polygon![
                (x: 0., y: 10.),
                (x: 10. + 360. * turns, y: 10.),
                (x: 10., y: 20.),
                (x: 0., y: 20.),
                (x: 0., y: 10.),
            ];
            assert_relative_eq!(
                near,
                far.chamberlain_duquette_signed_area(),
                max_relative = 1e-12
            );
        }
    }

    #[test]
    fn test_ends_too_far_out_taken_as_written() {
        // These aren't longitudes, so the ring is taken as written, as before, rather than reduced
        // to a plausible area.
        let earth = 4. * std::f64::consts::PI * EQUATORIAL_EARTH_RADIUS * EQUATORIAL_EARTH_RADIUS;
        let far: Polygon<f64> = polygon![
            (x: 0., y: 10.),
            (x: 10. + 360. * 1e9, y: 10.),
            (x: 10., y: 20.),
            (x: 0., y: 20.),
            (x: 0., y: 10.),
        ];
        assert!(far.chamberlain_duquette_signed_area().abs() > earth);
        let extreme = corner_box(-f64::MAX, f64::MAX, 10., 20.);
        let area = extreme.chamberlain_duquette_signed_area();
        assert!(area.is_nan() || area.abs() > earth);
    }

    #[test]
    fn test_triangle_with_wide_edge() {
        // Its edge from -100° to 100° is taken the short way, through the antimeridian, so it
        // winds around a pole, as the same ring written through 180° does.
        let triangle = Triangle::new((-100., 0.).into(), (100., 0.).into(), (0., 10.).into());
        let through = polygon![
            (x: -100., y: 0.),
            (x: -180., y: 0.),
            (x: 100., y: 0.),
            (x: 0., y: 10.),
            (x: -100., y: 0.),
        ];
        assert_relative_eq!(
            through.chamberlain_duquette_signed_area(),
            triangle.chamberlain_duquette_signed_area(),
            max_relative = 1e-9
        );
    }

    /// An irregular pentagon about 2 km across at 45°N with its middle on the antimeridian:
    /// across it if `wrap`, or written past 180° if not.
    fn pentagon<T: CoordFloat>(wrap: bool) -> Polygon<T> {
        let offsets: [(f64, f64); 5] = [
            (-0.01, 0.),
            (0.008, -0.006),
            (0.012, 0.004),
            (0.002, 0.01),
            (-0.008, 0.006),
        ];
        let ring: Vec<(T, T)> = offsets
            .iter()
            .map(|&(dx, dy)| {
                let x = if wrap && dx > 0. {
                    dx - 180.
                } else {
                    180. + dx
                };
                (T::from(x).unwrap(), T::from(45. + dy).unwrap())
            })
            .collect();
        Polygon::new(LineString::from(ring), vec![])
    }

    #[test]
    fn test_small_polygon_across_antimeridian() {
        // As precise as the same polygon written without crossing the antimeridian.
        assert_relative_eq!(
            pentagon::<f64>(false).chamberlain_duquette_signed_area(),
            pentagon::<f64>(true).chamberlain_duquette_signed_area(),
            max_relative = 1e-9
        );
        assert_relative_eq!(
            pentagon::<f32>(false).chamberlain_duquette_signed_area(),
            pentagon::<f32>(true).chamberlain_duquette_signed_area(),
            max_relative = 1e-2
        );
    }

    #[test]
    fn test_small_box_across_antimeridian() {
        // As precise as the same box written without crossing the antimeridian.
        let across = corner_box(179.9995, -179.9995, 45., 45.001);
        let unwrapped = corner_box(179.9995, 180.0005, 45., 45.001);
        assert_relative_eq!(
            unwrapped.chamberlain_duquette_signed_area(),
            across.chamberlain_duquette_signed_area(),
            max_relative = 1e-9
        );
        let across = corner_box(179.95f32, -179.95, 45., 45.1);
        let unwrapped = corner_box(179.95f32, 180.05, 45., 45.1);
        assert_relative_eq!(
            unwrapped.chamberlain_duquette_signed_area(),
            across.chamberlain_duquette_signed_area(),
            max_relative = 1e-3
        );
    }
}
