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
/// Each edge is taken the short way around in longitude, as [`GeodesicArea`] takes it, so an
/// edge from 179° to -179° crosses the antimeridian and is 2° wide. An edge spanning exactly 180°
/// keeps the direction it is written in, and one written from -180° to 180° spans nothing.
///
/// A ring that winds around a pole (such as a parallel of latitude, written east or west all the
/// way around) splits the Earth in two. Its signed area is that of the smaller side: positive when
/// the smaller side is on the ring's left, as for a counter-clockwise ring, and negative when it's
/// on the ring's right.
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
            total = total + (p3.x.to_radians() - p1.x.to_radians()) * p2.y.to_radians().sin();
        }

        // The sum above is the sum over the edges of each edge's change in longitude times the
        // sum of the sines of its ends' latitudes, with the changes in longitude as written. An
        // edge written across more than 180° is taken the short way around instead: its share is
        // corrected here, so a ring with no such edge gives exactly what it did.
        let pi = T::from(std::f64::consts::PI).unwrap();
        let two_pi = pi + pi;
        let mut turn = T::zero();
        for i in 0..coords_len {
            let a = coords[i];
            let b = coords[(i + 1) % coords_len];
            let written = b.x.to_radians() - a.x.to_radians();
            let short = if written.abs() > pi {
                written - two_pi * (written / two_pi).round()
            } else {
                written
            };
            if short != written {
                total =
                    total + (short - written) * (a.y.to_radians().sin() + b.y.to_radians().sin());
            }
            turn = turn + short;
        }

        total = total
            * T::from(EQUATORIAL_EARTH_RADIUS).unwrap()
            * T::from(EQUATORIAL_EARTH_RADIUS).unwrap()
            / T::from(-2).unwrap();

        // That is the area between the ring and the equator. A ring winding around a pole
        // encloses the area between it and the pole instead, on its left: 2πR² more for each
        // turn east (less for each turn west). Of the two sides it splits the Earth into, the
        // smaller is given, negative when it's on the ring's right.
        let winding = (turn / two_pi).round();
        if winding != T::zero() {
            let r2 = T::from(EQUATORIAL_EARTH_RADIUS).unwrap()
                * T::from(EQUATORIAL_EARTH_RADIUS).unwrap();
            let earth = two_pi * (r2 + r2);
            total = total + two_pi * r2 * winding;
            total = total - earth * (total / earth).round();
        }
    }
    total
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
        let half = polygon![
            (x: 0., y: 10.),
            (x: 180., y: 10.),
            (x: 180., y: 20.),
            (x: 0., y: 20.),
            (x: 0., y: 10.),
        ];
        let quarter = polygon![
            (x: 0., y: 10.),
            (x: 90., y: 10.),
            (x: 90., y: 20.),
            (x: 0., y: 20.),
            (x: 0., y: 10.),
        ];
        assert_relative_eq!(
            2. * quarter.chamberlain_duquette_signed_area(),
            half.chamberlain_duquette_signed_area(),
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
}
