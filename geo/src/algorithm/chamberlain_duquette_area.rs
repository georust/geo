use crate::{CoordinateType, LineString, Polygon, EQUATORIAL_EARTH_RADIUS};
use num_traits::Float;

/// Signed approximate geodesic area of a geometry.
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
/// use geo::{polygon, prelude::*};
///
/// // The O2 in London
/// let p = polygon![
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
/// assert_eq!(
///     78478.08613616147, // 78,478 meters²
///     p.chamberlain_duquette_area(),
/// );
/// ```
pub trait ChamberlainDuquetteArea<T>
where
    T: Float + CoordinateType,
{
    fn chamberlain_duquette_area(&self) -> T;
}

impl<T> ChamberlainDuquetteArea<T> for Polygon<T>
where
    T: Float + CoordinateType,
{
    fn chamberlain_duquette_area(&self) -> T {
        self.interiors()
            .iter()
            .fold(ring_area(self.exterior()), |total, next| {
                total - ring_area(next)
            })
    }
}

fn ring_area<T>(coords: &LineString<T>) -> T
where
    T: Float + CoordinateType,
{
    let mut p1;
    let mut p2;
    let mut p3;
    let mut lower_index;
    let mut middle_index;
    let mut upper_index;
    let mut total = T::zero();
    let coords_len = coords.0.len();

    if coords_len > 2 {
        for i in 0..coords_len {
            if i == coords_len - 2 {
                // i = N-2
                lower_index = coords_len - 2;
                middle_index = coords_len - 1;
                upper_index = 0;
            } else if i == coords_len - 1 {
                // i = N-1
                lower_index = coords_len - 1;
                middle_index = 0;
                upper_index = 1;
            } else {
                // i = 0 to N-3
                lower_index = i;
                middle_index = i + 1;
                upper_index = i + 2;
            }
            p1 = coords[lower_index];
            p2 = coords[middle_index];
            p3 = coords[upper_index];
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
        assert_eq!(-7766240997209.013, polygon.chamberlain_duquette_area());
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
        assert_eq!(7766240997209.013, polygon.chamberlain_duquette_area());
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
        assert_eq!(1208198651182.4727, poly.chamberlain_duquette_area());
    }
}
