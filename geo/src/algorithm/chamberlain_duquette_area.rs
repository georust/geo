use crate::geometry::*;
use crate::{CoordFloat, EQUATORIAL_EARTH_RADIUS};

/// Calculate the signed approximate geodesic area of a `Geometry`.
///
/// # Units
///
/// - return value: meters²
///
/// # Edges wider than 180°
///
/// If a ring has an edge spanning more than 180° of longitude that isn't along a parallel, as a
/// polygon crossing the antimeridian without being cut there does, its wide edges (except along a
/// pole) are taken the short way around, as [`GeodesicArea`] takes them. Otherwise the ring is
/// taken as written, as before: boxes and [`Rect`]s are as wide as they are written, and polygons
/// clipped along parallels are unchanged. So a box across the antimeridian written from 179° to
/// -179° is 358° wide; write it from 179° to 181° instead. An edge counts as along a parallel if
/// its ends' latitudes are within 1e-9° (about 4e-5° in `f32`).
///
/// A ring taken the short way can wind around a pole. Its polygon then measures the smaller of
/// the two sides it divides the Earth into, so one bigger than half the Earth measures its
/// complement. A pole is best written as an edge along it rather than as a single vertex.
///
/// Holes are subtracted with their signed area, so a hole wound opposite to its exterior is added
/// rather than subtracted: a known limitation.
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
        let (mut total, winding) = ring_area(self.exterior());
        // The polygon is reduced to its smaller side if its exterior winds around a pole, its
        // holes don't wind it back, and each ring's area is one its winding can account for.
        let mut net_winding = winding;
        let mut reducible = winding != 0 && accounted_for(total, winding);
        for hole in self.interiors() {
            let (area, hole_winding) = ring_area(hole);
            if winding != 0 {
                net_winding = net_winding.saturating_sub(hole_winding);
                reducible = reducible && accounted_for(area, hole_winding);
                total = total - area;
            } else if hole_winding != 0 && accounted_for(area, hole_winding) {
                // A hole winding around a pole in an exterior that doesn't cuts out the smaller of
                // its sides, whichever way it winds.
                total = total - total.signum() * smaller_side(area).abs();
            } else {
                total = total - area;
            }
        }
        if reducible && net_winding != 0 {
            smaller_side(total)
        } else {
            total
        }
    }

    fn chamberlain_duquette_unsigned_area(&self) -> T {
        self.chamberlain_duquette_signed_area().abs()
    }
}

/// The signed area of a ring in meters², and how many times it winds around a pole (east
/// positive). For a ring that winds, the area is that on its left, to within whole Earths.
fn ring_area<T>(coords: &LineString<T>) -> (T, i64)
where
    T: CoordFloat,
{
    let mut total = T::zero();
    let coords_len = coords.0.len();
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
            // p3) is summed the short way around instead, unless it is too far out.
            if !as_written && wider_than_half_turn(p2, p3) && !along_parallel(p2, p3) {
                match short_way_ring_area(&coords.0) {
                    Some(area_and_winding) => return area_and_winding,
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
    (total, 0)
}

/// `ring_area` for a ring with an edge written across more than 180° of longitude that isn't
/// along a parallel, taking each of its edges written across more than 180° the short way around
/// (except along a pole); or `None` if a longitude is too far out to be one, when the ring is
/// taken as written.
///
/// It is the sum in `ring_area` regrouped by edge: each edge's change in longitude times the sum
/// of the sines of its ends' latitudes. The sines are taken relative to the first vertex's, which
/// keeps the terms as small as the ring is wherever it lies; what that takes off is the first
/// vertex's sine times the changes in longitude, which add up to whole turns, so it goes back on
/// exactly.
fn short_way_ring_area<T>(coords: &[Coord<T>]) -> Option<(T, i64)>
where
    T: CoordFloat,
{
    // Further out than this isn't a longitude: the ring is taken as written, as it always was,
    // rather than reduced to a plausible area. (Written as `all(.. <= ..)` so that a NaN is
    // turned back here too, and `short_span` only sees longitudes within it.)
    let limit = T::from(720).unwrap();
    if !coords.iter().all(|coord| coord.x.abs() <= limit) {
        return None;
    }

    let sin_0 = coords[0].y.to_radians().sin();
    let mut total = T::zero();
    let mut turns_taken_off = 0_i64;
    let mut a = coords[coords.len() - 1];
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
    // for each turn west).
    Some((total + tau * r2 * winding, -turns_taken_off))
}

/// The change in longitude from `a` to `b` (degrees), within ±720°, taken the short way around if
/// it is more than 180° and the edge isn't along a pole, and the whole turns taken off the change
/// as written to get it. A change of 180° plus whole turns keeps the direction it is written in.
fn short_span<T>(a: Coord<T>, b: Coord<T>) -> Option<(T, i64)>
where
    T: CoordFloat,
{
    let half_turn = T::from(180).unwrap();
    let full_turn = half_turn + half_turn;
    let written = b.x - a.x;
    if !wider_than_half_turn(a, b) || along_pole(a, b) {
        return Some((written, 0));
    }
    let mut turns = (written / full_turn).round();
    let rest = written - turns * full_turn;
    if (rest.abs() - half_turn).abs() <= half_turn_tolerance(a, b)
        && rest.signum() != written.signum()
    {
        turns = turns - written.signum();
    }
    // Taken from the ends rather than from `written`, which has rounded to the precision of a
    // value near a whole turn.
    let span = b.x - (a.x + turns * full_turn);
    Some((span, turns.to_i64()?))
}

/// The tolerance in degrees within which an edge runs along a parallel or a pole: 1e-9°, or the
/// precision of the type near 360° if coarser (in `f32`, about 4e-5°).
fn tolerance<T>() -> T
where
    T: CoordFloat,
{
    T::from(1e-9)
        .unwrap()
        .max(T::from(360).unwrap() * T::epsilon())
}

/// The tolerance in degrees within which the change in longitude from `a` to `b` is 180° (plus
/// whole turns): `tolerance()`, or the precision of the longitudes if coarser.
fn half_turn_tolerance<T>(a: Coord<T>, b: Coord<T>) -> T
where
    T: CoordFloat,
{
    let precision = T::from(4).unwrap() * T::epsilon() * a.x.abs().max(b.x.abs());
    tolerance::<T>().max(precision)
}

/// Whether the change in longitude from `a` to `b` is more than 180°, beyond the tolerance.
fn wider_than_half_turn<T>(a: Coord<T>, b: Coord<T>) -> bool
where
    T: CoordFloat,
{
    (b.x - a.x).abs() > T::from(180).unwrap() + half_turn_tolerance(a, b)
}

/// Whether an edge runs along a parallel of latitude, its ends' latitudes within the tolerance,
/// or along a pole.
fn along_parallel<T>(a: Coord<T>, b: Coord<T>) -> bool
where
    T: CoordFloat,
{
    (a.y - b.y).abs() <= tolerance() || along_pole(a, b)
}

/// Whether an edge runs along a pole, both ends within the tolerance of it: wherever their
/// longitudes, the edge goes nowhere.
fn along_pole<T>(a: Coord<T>, b: Coord<T>) -> bool
where
    T: CoordFloat,
{
    let quarter_turn = T::from(90).unwrap();
    a.y.signum() == b.y.signum()
        && quarter_turn - a.y.abs() <= tolerance()
        && quarter_turn - b.y.abs() <= tolerance()
}

/// Whether a ring's area is one its winding around a pole can account for, at most a whole Earth
/// for each turn and one more. A ring taken as written can come to more (or to NaN), which is left
/// as it is rather than reduced to a plausible area.
fn accounted_for<T>(area: T, winding: i64) -> bool
where
    T: CoordFloat,
{
    let earth = T::from(2. * std::f64::consts::TAU).unwrap() * radius_squared();
    match T::from(winding.unsigned_abs().saturating_add(1)) {
        Some(turns) => area.abs() <= earth * turns,
        None => false,
    }
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
            max_relative = 5e-4
        );
    }

    #[test]
    fn test_triangle_across_antimeridian() {
        // `Triangle::new` would order these by their winding in the plane, which across the
        // antimeridian isn't their winding on the sphere.
        let across =
            Triangle::unchecked_winding((170., 0.).into(), (-170., 10.).into(), (175., 20.).into());
        let past =
            Triangle::unchecked_winding((170., 0.).into(), (190., 10.).into(), (175., 20.).into());
        assert_relative_eq!(
            past.chamberlain_duquette_signed_area(),
            across.chamberlain_duquette_signed_area(),
            max_relative = 1e-12
        );
    }

    #[test]
    fn test_crossings_along_and_off_parallels() {
        // A ring that crosses the antimeridian once along a parallel and once off one takes both
        // crossings the short way, as written past 180° (where a crossing written through
        // vertices at 180° and -180° goes nowhere).
        let rings = [
            (
                polygon![
                    (x: 179.8, y: -16.8),
                    (x: -179.9, y: -16.8),
                    (x: -179.8, y: -16.5),
                    (x: 179.9, y: -16.4),
                ],
                polygon![
                    (x: 179.8, y: -16.8),
                    (x: 180.1, y: -16.8),
                    (x: 180.2, y: -16.5),
                    (x: 179.9, y: -16.4),
                ],
            ),
            (
                polygon![
                    (x: 179., y: 0.),
                    (x: -179., y: 0.),
                    (x: -179., y: 1.),
                    (x: 179., y: 1.5),
                ],
                polygon![
                    (x: 179., y: 0.),
                    (x: 181., y: 0.),
                    (x: 181., y: 1.),
                    (x: 179., y: 1.5),
                ],
            ),
            (
                polygon![
                    (x: 179., y: 60.),
                    (x: 180., y: 60.5),
                    (x: -180., y: 60.5),
                    (x: -179., y: 61.),
                    (x: -179., y: 62.),
                    (x: 179., y: 62.5),
                ],
                polygon![
                    (x: 179., y: 60.),
                    (x: 180., y: 60.5),
                    (x: 181., y: 61.),
                    (x: 181., y: 62.),
                    (x: 179., y: 62.5),
                ],
            ),
        ];
        for (across, past) in rings {
            assert_relative_eq!(
                past.chamberlain_duquette_signed_area(),
                across.chamberlain_duquette_signed_area(),
                max_relative = 1e-9
            );
        }
        // In f32, where a crossing 2 m off a parallel, as here, counts as along it, so the ring
        // is taken the short way by its other crossing.
        let across: Polygon<f32> = polygon![
            (x: 179.99, y: 45.),
            (x: -179.99, y: 45.00002),
            (x: -179.99, y: 45.01),
            (x: 179.99, y: 45.015),
        ];
        let past: Polygon<f32> = polygon![
            (x: 179.99, y: 45.),
            (x: 180.01, y: 45.00002),
            (x: 180.01, y: 45.01),
            (x: 179.99, y: 45.015),
        ];
        assert_relative_eq!(
            past.chamberlain_duquette_signed_area(),
            across.chamberlain_duquette_signed_area(),
            max_relative = 5e-4
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
        // corners are a few units in the last place off, or computed a little differently.
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
        let at_equator = polygon![
            (x: -180., y: 0.),
            (x: 180., y: -1e-17),
            (x: 180., y: 10.),
            (x: -180., y: 10.),
        ];
        assert_relative_eq!(
            band_area(0., 10., 360.),
            at_equator.chamberlain_duquette_signed_area(),
            max_relative = 1e-9
        );
        // One corner through a round trip to Web Mercator and back, the other not.
        let mercator = polygon![
            (x: -180., y: 0.49824634209292074),
            (x: 180., y: 0.4982463420929226),
            (x: 180., y: 10.),
            (x: -180., y: 10.),
        ];
        assert_relative_eq!(
            band_area(0.49824634209292074, 10., 360.),
            mercator.chamberlain_duquette_signed_area(),
            max_relative = 1e-9
        );
        // So in f32, where such a round trip is off by up to about 1e-5°.
        let mercator_f32: Polygon<f32> = polygon![
            (x: -180., y: 0.6817),
            (x: 180., y: 0.6817007),
            (x: 180., y: 10.6817),
            (x: -180., y: 10.6817),
        ];
        assert_relative_eq!(
            band_area(0.6817, 10.6817, 360.) as f32,
            mercator_f32.chamberlain_duquette_signed_area(),
            max_relative = 1e-4
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
        // So it is with the pole a little off, where longitude makes no difference.
        let mut nearly = north_of_60s.exterior().0.clone();
        nearly[6].y = 90. - 1e-10;
        assert_relative_eq!(
            cap_area(-60.),
            Polygon::new(LineString::new(nearly), vec![]).chamberlain_duquette_signed_area(),
            max_relative = 1e-9
        );
        // And with a dent across the antimeridian, which takes the ring the short way: its edge
        // along the North Pole is still taken as written, so the ring still encloses the pole.
        let dented = polygon![
            (x: -180., y: -60.),
            (x: -90., y: -60.),
            (x: 0., y: -60.),
            (x: 90., y: -60.),
            (x: 170., y: -60.),
            (x: -179., y: -61.),
            (x: 179., y: -62.),
            (x: 180., y: -60.),
            (x: 180., y: 90.),
            (x: -180., y: 90.),
        ];
        let dented_past = polygon![
            (x: -180., y: -60.),
            (x: -90., y: -60.),
            (x: 0., y: -60.),
            (x: 90., y: -60.),
            (x: 170., y: -60.),
            (x: 181., y: -61.),
            (x: 179., y: -62.),
            (x: 180., y: -60.),
            (x: 180., y: 90.),
            (x: -180., y: 90.),
        ];
        assert_relative_eq!(
            dented_past.chamberlain_duquette_signed_area(),
            dented.chamberlain_duquette_signed_area(),
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
        // Along parallels, from each of these: in radians, the span from all but the first rounds
        // to just over π.
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

    /// The area of a quadrilateral from 10° east 180° to 11°, up to 20°, and back west 180° to 19°,
    /// in meters².
    fn half_band() -> f64 {
        let sin = |lat: f64| lat.to_radians().sin();
        std::f64::consts::PI / 2.
            * EQUATORIAL_EARTH_RADIUS
            * EQUATORIAL_EARTH_RADIUS
            * (sin(20.) + sin(19.) - sin(10.) - sin(11.))
    }

    #[test]
    fn test_edges_off_parallels_spanning_180_degrees_keep_their_direction() {
        // Written across 180°, or 540°, east and then west: the way they are written, rather than
        // the other way round. In 0°–360°, 256.2884 - 76.2884 computes to 180.00000000000003.
        for west in [0., 76.2884, -180.] {
            let east = if west < 0. { 360. } else { west + 180. };
            let ring = polygon![
                (x: west, y: 10.),
                (x: east, y: 11.),
                (x: east, y: 20.),
                (x: west, y: 19.),
                (x: west, y: 10.),
            ];
            assert_relative_eq!(
                half_band(),
                ring.chamberlain_duquette_signed_area(),
                max_relative = 1e-12
            );
        }
        // In f32, 256.0002 - 76.0002 computes to 180.00002.
        let ring: Polygon<f32> = polygon![
            (x: 76.0002, y: 10.),
            (x: 256.0002, y: 11.),
            (x: 256.0002, y: 20.),
            (x: 76.0002, y: 19.),
        ];
        assert_relative_eq!(
            half_band() as f32,
            ring.chamberlain_duquette_signed_area(),
            max_relative = 1e-4
        );
    }

    #[test]
    fn test_ends_written_whole_turns_apart() {
        // A corner written a turn further round gives the same polygon.
        let near = polygon![
            (x: 0., y: 10.),
            (x: 10., y: 11.),
            (x: 10., y: 20.),
            (x: 0., y: 19.),
            (x: 0., y: 10.),
        ];
        let far = polygon![
            (x: 0., y: 10.),
            (x: 370., y: 11.),
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
        // Nor is such an area reduced by a ring winding around a pole in the same polygon.
        let with_winding_hole = Polygon::new(far.exterior().clone(), vec![zigzag(80., 1)]);
        assert!(with_winding_hole.chamberlain_duquette_signed_area().abs() > earth());
        let in_winding_exterior = Polygon::new(zigzag(80., 1), vec![polar.exterior().clone()]);
        assert!(in_winding_exterior.chamberlain_duquette_signed_area().abs() > earth());
        // Nor by many turns of the exterior: a hole taken as written covers two Earths.
        let twice: Polygon<f64> = polygon![
            (x: -360., y: -90.),
            (x: 360., y: -90.),
            (x: 360., y: 90.),
            (x: -360., y: 90.),
        ];
        let many_turns = Polygon::new(zigzag(80., 2), vec![twice.exterior().clone()]);
        assert!(many_turns.chamberlain_duquette_signed_area().abs() > earth());
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
            max_relative = 5e-5
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
        let ring_f32: Vec<(f32, f32)> = zigzag(-80., 1)
            .coords()
            .map(|coord| (coord.x as f32, coord.y as f32))
            .collect();
        let ring_f32 = Polygon::new(LineString::from(ring_f32), vec![]);
        assert_relative_eq!(
            (north - earth()) as f32,
            ring_f32.chamberlain_duquette_signed_area(),
            max_relative = 5e-5
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
        // The world but Antarctica: a hole winding around the South Pole in an exterior that
        // doesn't wind cuts out the smaller of its sides, whichever way it winds.
        let world = Rect::new((-180., -90.), (180., 90.)).to_polygon();
        let east = zigzag(-70., 1);
        let mut west = east.clone();
        west.0.reverse();
        for antarctica in [east, west] {
            let world_but_antarctica = Polygon::new(world.exterior().clone(), vec![antarctica]);
            assert_relative_eq!(
                north_of_zigzag(-70.),
                world_but_antarctica.chamberlain_duquette_signed_area(),
                max_relative = 1e-9
            );
        }
        // A band of more than half the Earth: its holes wind it back, so it isn't reduced.
        let wide_band = Polygon::new(zigzag(-60., 1), vec![zigzag(60., 1)]);
        assert_relative_eq!(
            north_of_zigzag(-60.) - north_of_zigzag(60.),
            wide_band.chamberlain_duquette_signed_area(),
            max_relative = 1e-9
        );
    }
}
