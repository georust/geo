use crate::{CoordinateType, LineString, Polygon, EQUATORIAL_EARTH_RADIUS};
use num_traits::Float;

/// Calculate the signed approximate geodesic area of a `Geometry`.
///
/// # Units
///
/// - return value: meters²
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
    T: Float + CoordinateType,
{
    fn chamberlain_duquette_signed_area(&self) -> T;

    fn chamberlain_duquette_unsigned_area(&self) -> T;
}

impl<T> ChamberlainDuquetteArea<T> for Polygon<T>
where
    T: Float + CoordinateType,
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
    T: Float + CoordinateType,
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

        total = total
            * T::from(EQUATORIAL_EARTH_RADIUS).unwrap()
            * T::from(EQUATORIAL_EARTH_RADIUS).unwrap()
            / T::from(-2).unwrap();
    }
    total
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
}
