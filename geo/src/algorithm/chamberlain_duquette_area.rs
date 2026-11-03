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
/// so an edge from 179° to -179° crosses the antimeridian and is 2° wide, and one written from
/// -180° to 180° spans nothing. An edge spanning exactly 180° (as stored) keeps the direction it
/// is written in, as a rhumb line does ([`GeodesicArea`] takes such an edge over a pole). So a
/// polygon with an edge wider than 180°, such as a box from -100° to 100° written with only its
/// corners, is the narrower box the other way round (from 100° to 260°), with the opposite sign;
/// with more vertices along its edges, it is the wider box. A [`Rect`] is always the box from its
/// minimum longitude east to its maximum.
///
/// A ring that winds around a pole (such as a parallel of latitude, written east or west all the
/// way around) splits the Earth in two. Its signed area is that of the smaller side, counted once
/// for each time the ring winds around: positive when the smaller side is on the ring's left, as
/// for a counter-clockwise ring, and negative when it's on the ring's right. A ring along the
/// equator gives half the Earth, positive. A ring that doesn't wind around a pole gives the side
/// with neither pole in it, which can be more than half the Earth. The unsigned area is the
/// absolute value of the signed area.
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
            // This sum takes each edge's change in longitude as written, so a ring with an edge
            // written across more than 180° (here, the edge from p2 to p3) is summed the short way
            // around instead. The span is compared in degrees, where an edge written across
            // exactly 180° is exactly 180° (in radians it can round to just over π).
            if (p3.x - p2.x).abs() > half_turn {
                return short_way_ring_area(&coords.0);
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

/// The area of a ring with an edge written across more than 180° of longitude, taking each edge
/// the short way around.
///
/// It is the sum in `ring_area` regrouped by edge (each edge's change in longitude times the sum
/// of the sines of its ends' latitudes), with each change in longitude taken the short way. Each
/// is reduced exactly, so the sum has no large terms to cancel, however far apart the ends are
/// written.
fn short_way_ring_area<T>(coords: &[Coord<T>]) -> T
where
    T: CoordFloat,
{
    let full_turn = T::from(360).unwrap();
    let r2 = T::from(EQUATORIAL_EARTH_RADIUS).unwrap() * T::from(EQUATORIAL_EARTH_RADIUS).unwrap();

    let mut total = T::zero();
    let mut turned = T::zero();
    let mut a = coords[coords.len() - 1];
    let mut sin_a = a.y.to_radians().sin();
    for &b in coords {
        let sin_b = b.y.to_radians().sin();
        let span = short_span(a.x, b.x);
        total = total + span.to_radians() * (sin_a + sin_b);
        turned = turned + span;
        a = b;
        sin_a = sin_b;
    }
    total = total * r2 / T::from(-2).unwrap();

    // That is the area between the ring and the equator. A ring winding around a pole encloses
    // the area between it and the pole instead, on its left: 2πR² more for each turn east (less
    // for each turn west). Of the two sides it splits the Earth into, the smaller is given,
    // negative when it's on the ring's right (and positive for halves, such as a ring along the
    // equator: the result is in (-earth/2, earth/2]).
    let winding = (turned / full_turn).round();
    if winding != T::zero() {
        let half_earth = T::from(std::f64::consts::TAU).unwrap() * r2;
        let earth = half_earth + half_earth;
        total = total + half_earth * winding;
        total = total - earth * (total / earth).round();
        if total <= -half_earth {
            total = total + earth;
        }
    }
    total
}

/// The change in longitude from `a` to `b` (degrees) taken the short way around, in
/// [-180°, 180°]. A change of exactly 180° (as stored), or of 180° plus whole turns, keeps the
/// direction it is written in.
fn short_span<T>(a: T, b: T) -> T
where
    T: CoordFloat,
{
    let half_turn = T::from(180).unwrap();
    let full_turn = half_turn + half_turn;
    let written = b - a;
    if written.abs() <= half_turn {
        return written;
    }
    // `%` is exact, so this is under 720° and as precise as `a` and `b` themselves, however many
    // turns apart they are written (even if `b - a` overflows).
    let span = b % full_turn - a % full_turn;
    let span = span - full_turn * (span / full_turn).round();
    if span.abs() == half_turn {
        half_turn * written.signum()
    } else {
        span
    }
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

/// A `Rect` spans from its minimum longitude east to its maximum, however wide, so it is measured
/// directly rather than as a polygon (whose edges are each taken the short way around).
impl<T> ChamberlainDuquetteArea<T> for Rect<T>
where
    T: CoordFloat,
{
    fn chamberlain_duquette_signed_area(&self) -> T {
        let r2 =
            T::from(EQUATORIAL_EARTH_RADIUS).unwrap() * T::from(EQUATORIAL_EARTH_RADIUS).unwrap();
        (self.max().x - self.min().x).to_radians()
            * (self.max().y.to_radians().sin() - self.min().y.to_radians().sin())
            * r2
    }

    fn chamberlain_duquette_unsigned_area(&self) -> T {
        self.chamberlain_duquette_signed_area().abs()
    }
}
sum_impl!(GeometryCollection);
sum_impl!(MultiPolygon);

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

    const EAST: [f64; 13] = [
        0., 30., 60., 90., 120., 150., 180., 210., 240., 270., 300., 330., 360.,
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
        for lons in [EAST, EAST_FOLDED] {
            let mut ring = parallel(80., &lons);
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
    fn test_cap_written_as_box() {
        // An edge from -180° to 180° spans nothing, so a cap written as a box encloses nothing
        // (as with `GeodesicArea`); with vertices no more than 180° apart along the parallel, it
        // is the cap.
        let as_box = polygon![
            (x: -180., y: 80.),
            (x: 180., y: 80.),
            (x: 180., y: 90.),
            (x: -180., y: 90.),
            (x: -180., y: 80.),
        ];
        assert_relative_eq!(0., as_box.chamberlain_duquette_signed_area(), epsilon = 1.);
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
        // Each change in longitude is reduced exactly, so a corner written whole turns further
        // round gives the same box.
        let near = corner_box(0., 10., 10., 20.).chamberlain_duquette_signed_area();
        for turns in [1., 1e3, 1e9] {
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
    fn test_extreme_longitudes_give_a_finite_area() {
        // The change in longitude written here overflows; reduced, it doesn't.
        let area = corner_box(-f64::MAX, f64::MAX, 10., 20.).chamberlain_duquette_signed_area();
        assert!(area.is_finite());
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
