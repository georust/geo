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
/// An edge along a parallel of latitude (to within rounding) is taken as written, as a straight
/// line in longitude, so a box from -100° to 100° is 200° wide and one from -180° to 180° is the
/// whole band of latitude, as are the edges a clip along a parallel leaves. Any other edge written
/// across more than 180° of longitude is taken the short way around, as [`GeodesicArea`] takes it:
/// an edge from (179°, 0°) to (-179°, 1°) crosses the antimeridian and is 2° wide. (An edge whose
/// change in longitude computes to exactly 180° keeps the direction it is written in.) So a box
/// across the antimeridian is best written past 180°, from 179° to 181°, or as a [`Rect`]: written
/// from 179° to -179°, its edges along the parallels are 358° wide. A ring with a longitude more
/// than a million degrees out, which isn't a longitude, is taken as written throughout.
///
/// A ring that winds around a pole splits the Earth in two, as a coastline written all the way
/// around Antarctica does when it isn't closed along the pole. A polygon with such a ring measures
/// the smaller of the two sides its rings divide the Earth into: positive when that side is on the
/// exterior's left (as for a counter-clockwise exterior) and negative when it's on the right, so
/// its signed area is in (-earth/2, earth/2]. Any other polygon measures what it encloses as
/// written, which can be more than half the Earth. A pole is best written as an edge along it
/// rather than as a single vertex.
///
/// The unsigned area is the absolute value of the signed area (unlike [`GeodesicArea`]'s, which
/// is the area on the exterior's left). Holes are subtracted with their signed area, so they
/// should wind the same way as their exterior: a hole wound the other way (as in OGC Simple
/// Features and RFC 7946) is added instead.
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
        let (exterior, mut winds) = ring_area(self.exterior());
        let total = self.interiors().iter().fold(exterior, |total, next| {
            let (area, hole_winds) = ring_area(next);
            winds |= hole_winds;
            total - area
        });
        if winds { smaller_side(total) } else { total }
    }

    fn chamberlain_duquette_unsigned_area(&self) -> T {
        self.chamberlain_duquette_signed_area().abs()
    }
}

/// The signed area of a ring in meters², and whether it winds around a pole, when the area is
/// that on its left, to within whole Earths.
fn ring_area<T>(coords: &LineString<T>) -> (T, bool)
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
            // written across more than 180° that isn't along a parallel (here, the edge from p2 to
            // p3) takes such edges the short way around instead, unless it is too far out. The
            // span is compared in degrees, where an edge written across exactly 180° is exactly
            // 180° (in radians it can round to just over π).
            if !as_written && (p3.x - p2.x).abs() > half_turn && !along_parallel(p2, p3) {
                match short_way_ring_area(&coords.0) {
                    Some(area_and_winds) => return area_and_winds,
                    None => as_written = true,
                }
            }
            total = total + (p3.x.to_radians() - p1.x.to_radians()) * p2.y.to_radians().sin();
        }

        // R·R rather than `radius_squared()`, which rounds differently, so that this is what it
        // always was, bit for bit.
        total = total
            * T::from(EQUATORIAL_EARTH_RADIUS).unwrap()
            * T::from(EQUATORIAL_EARTH_RADIUS).unwrap()
            / T::from(-2).unwrap();
    }
    (total, false)
}

/// `ring_area` for a ring with an edge written across more than 180° of longitude that isn't
/// along a parallel, taking such edges the short way around; or `None` if a longitude is too far
/// out to be one, when the ring is taken as written.
///
/// It is the sum in `ring_area` regrouped by edge: each edge's change in longitude times the sum
/// of the sines of its ends' latitudes. The sines are taken relative to the first vertex's, which
/// keeps the terms as small as the ring is wherever it lies; what that takes off is the first
/// vertex's sine times the changes in longitude, which add up to whole turns, so it goes back on
/// exactly.
fn short_way_ring_area<T>(coords: &[Coord<T>]) -> Option<(T, bool)>
where
    T: CoordFloat,
{
    // Further out than this isn't a longitude: the ring is taken as written, as it always was,
    // rather than reduced to a plausible area.
    let limit = T::from(1e6).unwrap();
    if !coords.iter().all(|coord| coord.x.abs() <= limit) {
        return None;
    }

    let sin_0 = coords.first()?.y.to_radians().sin();
    let mut total = T::zero();
    let mut turns_taken_off = 0_i64;
    let mut a = *coords.last()?;
    let mut sin_a = a.y.to_radians().sin() - sin_0;
    for &b in coords {
        let sin_b = b.y.to_radians().sin() - sin_0;
        let (span, turns) = short_span(a, b);
        total = total + span.to_radians() * (sin_a + sin_b);
        turns_taken_off += turns;
        a = b;
        sin_a = sin_b;
    }
    // The changes in longitude as written add up to nothing, so the ring winds around by the
    // turns taken off them, the other way.
    let winding = T::from(-turns_taken_off).unwrap();
    let tau = T::from(std::f64::consts::TAU).unwrap();
    total = total + (sin_0 + sin_0) * tau * winding;
    let r2 = radius_squared::<T>();
    total = total * r2 / T::from(-2).unwrap();

    // That is the area between the ring and the equator. A ring winding around a pole encloses
    // the area between it and the pole instead, on its left: 2πR² more for each turn east (less
    // for each turn west).
    Some((total + tau * r2 * winding, winding != T::zero()))
}

/// The change in longitude from `a` to `b` (degrees), taken the short way around if it is more
/// than 180° and the edge isn't along a parallel, and the whole turns taken off the change as
/// written to get it. A change of 180° plus whole turns keeps the direction it is written in.
fn short_span<T>(a: Coord<T>, b: Coord<T>) -> (T, i64)
where
    T: CoordFloat,
{
    let half_turn = T::from(180).unwrap();
    let full_turn = half_turn + half_turn;
    let written = b.x - a.x;
    if written.abs() <= half_turn || along_parallel(a, b) {
        return (written, 0);
    }
    let mut turns = (written / full_turn).round();
    if (written - turns * full_turn).abs() == half_turn {
        turns = turns - written.signum();
    }
    // Taken from the ends rather than from `written`, which has rounded to the precision of a
    // value near a whole turn.
    let span = b.x - (a.x + turns * full_turn);
    (span, turns.to_i64().unwrap())
}

/// Whether an edge runs along a parallel of latitude (or a pole), to within a few units in the
/// last place.
fn along_parallel<T>(a: Coord<T>, b: Coord<T>) -> bool
where
    T: CoordFloat,
{
    (a.y - b.y).abs() <= T::from(360).unwrap() * T::epsilon()
}

/// An area on the left of a polygon winding around a pole, reduced by whole Earths into
/// (-earth/2, earth/2]: the smaller of the two sides it divides the Earth into, negative when
/// it's on the polygon's right (and positive for halves).
fn smaller_side<T>(area: T) -> T
where
    T: CoordFloat,
{
    let half_earth = T::from(std::f64::consts::TAU).unwrap() * radius_squared();
    let earth = half_earth + half_earth;
    let area = area - earth * (area / earth).round();
    if area <= -half_earth {
        area + earth
    } else {
        area
    }
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
to_polygon_impl!(Rect);
to_polygon_impl!(Triangle);
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

    /// The area of the Earth, in meters².
    fn earth() -> f64 {
        4. * std::f64::consts::PI * EQUATORIAL_EARTH_RADIUS * EQUATORIAL_EARTH_RADIUS
    }

    /// The area of the cap of the sphere north of `lat` (degrees), in meters².
    fn cap_area(lat: f64) -> f64 {
        2. * std::f64::consts::PI
            * EQUATORIAL_EARTH_RADIUS
            * EQUATORIAL_EARTH_RADIUS
            * (1. - lat.to_radians().sin())
    }

    /// The area of the band of latitude between `south` and `north`, `width` degrees wide.
    fn band_area(south: f64, north: f64, width: f64) -> f64 {
        (cap_area(south) - cap_area(north)) * width / 360.
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

    /// A ring zigzagging 1° either side of the parallel `lat` east through 0°, 30°, …, 330° and on
    /// to 0° again (an edge across 330° written, not along a parallel): it winds around the North
    /// Pole once for each of `turns`.
    fn zigzag(lat: f64, turns: i32) -> LineString<f64> {
        let ring: Vec<(f64, f64)> = (0..12 * turns)
            .map(|i| (f64::from(i) * 30., lat + if i % 2 == 0 { -1. } else { 1. }))
            .collect();
        LineString::from(ring)
    }

    /// What's north of `zigzag(lat, 1)`: the same ring written east to 360° and closed along the
    /// North Pole, measured as written.
    fn north_of_zigzag(lat: f64) -> f64 {
        let mut ring = zigzag(lat, 1).0;
        ring.extend([
            Coord::from((360., lat - 1.)),
            Coord::from((360., 90.)),
            Coord::from((0., 90.)),
        ]);
        Polygon::new(LineString::new(ring), vec![]).chamberlain_duquette_signed_area()
    }

    #[test]
    fn test_antimeridian() {
        // Edges across the antimeridian that aren't along a parallel are taken the short way, so
        // this is the same as the polygon written past 180°.
        let across = polygon![
            (x: 179., y: 0.),
            (x: -179., y: 0.5),
            (x: -179., y: 1.5),
            (x: 179., y: 1.),
            (x: 179., y: 0.),
        ];
        let past = polygon![
            (x: 179., y: 0.),
            (x: 181., y: 0.5),
            (x: 181., y: 1.5),
            (x: 179., y: 1.),
            (x: 179., y: 0.),
        ];
        let area = past.chamberlain_duquette_signed_area();
        assert!(area > 0.);
        assert_relative_eq!(
            area,
            across.chamberlain_duquette_signed_area(),
            max_relative = 1e-12
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
        // As precise as the same polygon written past 180°.
        assert_relative_eq!(
            pentagon::<f64>(false).chamberlain_duquette_signed_area(),
            pentagon::<f64>(true).chamberlain_duquette_signed_area(),
            max_relative = 1e-11
        );
        assert_relative_eq!(
            pentagon::<f32>(false).chamberlain_duquette_signed_area(),
            pentagon::<f32>(true).chamberlain_duquette_signed_area(),
            max_relative = 2e-3
        );
    }

    #[test]
    fn test_triangle_across_antimeridian() {
        // `Triangle::new` orders its vertices by their winding in the plane, which across the
        // antimeridian isn't their winding on the sphere, so only the unsigned areas agree.
        let across = Triangle::new((170., 0.).into(), (-170., 10.).into(), (175., 20.).into());
        let past = Triangle::new((170., 0.).into(), (190., 10.).into(), (175., 20.).into());
        assert_relative_eq!(
            past.chamberlain_duquette_unsigned_area(),
            across.chamberlain_duquette_unsigned_area(),
            max_relative = 1e-12
        );
    }

    #[test]
    fn test_edges_along_parallels_taken_as_written() {
        // A box is as wide as it is written, whether across the antimeridian or wider than 180°.
        assert_relative_eq!(
            -band_area(0., 1., 358.),
            corner_box(179., -179., 0., 1.).chamberlain_duquette_signed_area(),
            max_relative = 1e-9
        );
        assert_relative_eq!(
            band_area(0., 1., 2.),
            corner_box(179., 181., 0., 1.).chamberlain_duquette_signed_area(),
            max_relative = 1e-9
        );
        assert_relative_eq!(
            band_area(0., 10., 200.),
            corner_box(-100., 100., 0., 10.).chamberlain_duquette_signed_area(),
            max_relative = 1e-9
        );
        // From -180° to 180°, a box is the whole band of latitude, here a cap; so it is when the
        // corners are a few units in the last place off, as after a round trip through Web
        // Mercator.
        assert_relative_eq!(
            cap_area(80.),
            corner_box(-180., 180., 80., 90.).chamberlain_duquette_signed_area(),
            max_relative = 1e-9
        );
        let noisy = polygon![
            (x: -180., y: 10.),
            (x: 180., y: 10.000000000000002),
            (x: 180., y: 20.),
            (x: -180.00000000000003, y: 20.),
        ];
        assert_relative_eq!(
            band_area(10., 20., 360.),
            noisy.chamberlain_duquette_signed_area(),
            max_relative = 1e-9
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
        ];
        assert_relative_eq!(
            cap_area(-60.),
            north_of_60s.chamberlain_duquette_signed_area(),
            max_relative = 1e-9
        );
        // Strips clipped along parallels, as a clip leaves them: here from 57°W east to 140°E.
        let strip = polygon![
            (x: -57., y: -36.),
            (x: 140., y: -36.),
            (x: 140., y: -35.),
            (x: 100., y: -35.),
            (x: 40., y: -35.),
            (x: -20., y: -35.),
            (x: -57., y: -35.),
        ];
        assert_relative_eq!(
            band_area(-36., -35., 197.),
            strip.chamberlain_duquette_signed_area(),
            max_relative = 1e-9
        );
        let clipped = polygon![
            (x: -140., y: -85.),
            (x: 180., y: -85.),
            (x: 180., y: -80.),
            (x: 100., y: -80.),
            (x: 20., y: -80.),
            (x: -60., y: -80.),
            (x: -140., y: -80.),
        ];
        assert_relative_eq!(
            band_area(-85., -80., 320.),
            clipped.chamberlain_duquette_signed_area(),
            max_relative = 1e-9
        );
        // A ring along a parallel goes round and back again.
        let parallel = polygon![(x: 0., y: 80.), (x: 120., y: 80.), (x: 240., y: 80.)];
        assert_relative_eq!(
            0.,
            parallel.chamberlain_duquette_signed_area(),
            epsilon = 1.
        );
    }

    #[test]
    fn test_rect() {
        let world = Rect::new((-180., -90.), (180., 90.));
        assert_relative_eq!(
            earth(),
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
        assert_relative_eq!(
            band_area(0., 10., 200.),
            wide.chamberlain_duquette_signed_area(),
            max_relative = 1e-12
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
    fn test_edge_spanning_540_degrees_keeps_its_direction() {
        // 540° east is taken as 180° east, the way it is written, rather than 180° west.
        let short = polygon![
            (x: 0., y: 10.),
            (x: 180., y: 11.),
            (x: 180., y: 20.),
            (x: 0., y: 19.),
            (x: 0., y: 10.),
        ];
        let long = polygon![
            (x: -180., y: 10.),
            (x: 360., y: 11.),
            (x: 360., y: 20.),
            (x: -180., y: 19.),
            (x: -180., y: 10.),
        ];
        assert_relative_eq!(
            short.chamberlain_duquette_signed_area(),
            long.chamberlain_duquette_signed_area(),
            max_relative = 1e-12
        );
    }

    #[test]
    fn test_ends_written_whole_turns_apart() {
        // A corner written whole turns further round gives the same polygon.
        let near = polygon![
            (x: 0., y: 10.),
            (x: 10., y: 11.),
            (x: 10., y: 20.),
            (x: 0., y: 19.),
            (x: 0., y: 10.),
        ];
        for turns in [1., 1e3] {
            let far = polygon![
                (x: 0., y: 10.),
                (x: 10. + 360. * turns, y: 11.),
                (x: 10., y: 20.),
                (x: 0., y: 19.),
                (x: 0., y: 10.),
            ];
            assert_relative_eq!(
                near.chamberlain_duquette_signed_area(),
                far.chamberlain_duquette_signed_area(),
                max_relative = 1e-12
            );
        }
    }

    #[test]
    fn test_ends_too_far_out_taken_as_written() {
        // These aren't longitudes, so the ring is taken as written, as before, rather than reduced
        // to a plausible area.
        let far: Polygon<f64> = polygon![
            (x: 0., y: 10.),
            (x: 10. + 360. * 1e9, y: 11.),
            (x: 10., y: 20.),
            (x: 0., y: 19.),
            (x: 0., y: 10.),
        ];
        assert!(far.chamberlain_duquette_signed_area().abs() > earth());
        let polar: Polygon<f64> = polygon![
            (x: 0., y: -90.),
            (x: 1e20, y: -90.),
            (x: 1e20, y: 90.),
            (x: 400., y: 90.),
        ];
        assert!(polar.chamberlain_duquette_signed_area().abs() > earth());
    }

    #[test]
    fn test_ring_around_north_pole() {
        let north = north_of_zigzag(80.);
        assert!(north > 0.);
        let mut ring = Polygon::new(zigzag(80., 1), vec![]);
        assert_relative_eq!(
            north,
            ring.chamberlain_duquette_signed_area(),
            max_relative = 1e-9
        );
        ring.exterior_mut(|line_string| line_string.0.reverse());
        assert_relative_eq!(
            -north,
            ring.chamberlain_duquette_signed_area(),
            max_relative = 1e-9
        );
        assert_relative_eq!(
            north,
            ring.chamberlain_duquette_unsigned_area(),
            max_relative = 1e-9
        );
        let ring_f32: Vec<(f32, f32)> = zigzag(80., 1)
            .coords()
            .map(|coord| (coord.x as f32, coord.y as f32))
            .collect();
        let ring_f32 = Polygon::new(LineString::from(ring_f32), vec![]);
        assert_relative_eq!(
            north as f32,
            ring_f32.chamberlain_duquette_signed_area(),
            max_relative = 1e-4
        );
    }

    #[test]
    fn test_ring_around_south_pole() {
        // Going east around the South Pole, the smaller side (south of the ring) is on its right.
        let north = north_of_zigzag(-80.);
        let mut ring = Polygon::new(zigzag(-80., 1), vec![]);
        assert_relative_eq!(
            north - earth(),
            ring.chamberlain_duquette_signed_area(),
            max_relative = 1e-9
        );
        ring.exterior_mut(|line_string| line_string.0.reverse());
        assert_relative_eq!(
            earth() - north,
            ring.chamberlain_duquette_signed_area(),
            max_relative = 1e-9
        );
    }

    #[test]
    fn test_ring_winding_twice() {
        // Twice what's north of it, reduced into (-earth/2, earth/2].
        let ring = Polygon::new(zigzag(10., 2), vec![]);
        assert_relative_eq!(
            2. * north_of_zigzag(10.) - earth(),
            ring.chamberlain_duquette_signed_area(),
            max_relative = 1e-9
        );
    }

    #[test]
    fn test_ring_around_pole_with_holes() {
        let hole = polygon![
            (x: 0., y: 85.),
            (x: 10., y: 85.),
            (x: 10., y: 86.),
            (x: 0., y: 86.),
            (x: 0., y: 85.),
        ];
        let cap = Polygon::new(zigzag(80., 1), vec![hole.exterior().clone()]);
        assert_relative_eq!(
            north_of_zigzag(80.) - hole.chamberlain_duquette_signed_area(),
            cap.chamberlain_duquette_signed_area(),
            max_relative = 1e-9
        );
        // A band, going round below it and (as a hole) above it: more than half the Earth is
        // north of its exterior, but the band itself is less.
        let band = Polygon::new(zigzag(-10., 1), vec![zigzag(30., 1)]);
        assert_relative_eq!(
            north_of_zigzag(-10.) - north_of_zigzag(30.),
            band.chamberlain_duquette_signed_area(),
            max_relative = 1e-9
        );
    }
}
