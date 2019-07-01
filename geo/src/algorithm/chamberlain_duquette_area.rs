use crate::{CoordinateType, EQUATORIAL_EARTH_RADIUS, Polygon, LineString};
use num_traits::Float;

trait ChamberlainDuquetteArea<T>
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
        // TODO: holes
        ring_area(self.exterior())
    }
}

fn ring_area<T>(coords: &LineString<T>) -> T
where
    T: Float + CoordinateType,
{
    let mut p1;
    let mut p2;
    let mut p3;
    let mut lowerIndex;
    let mut middleIndex;
    let mut upperIndex;
    let mut total = T::zero();
    let coordsLength = coords.0.len();

    if coordsLength > 2 {
        for i in 0..coordsLength {
            if i == coordsLength - 2 { // i = N-2
                lowerIndex = coordsLength - 2;
                middleIndex = coordsLength - 1;
                upperIndex = 0;
            } else if i == coordsLength - 1 { // i = N-1
                lowerIndex = coordsLength - 1;
                middleIndex = 0;
                upperIndex = 1;
            } else { // i = 0 to N-3
                lowerIndex = i;
                middleIndex = i + 1;
                upperIndex = i + 2;
            }
            p1 = coords[lowerIndex];
            p2 = coords[middleIndex];
            p3 = coords[upperIndex];
            total = total + (p3.x.to_radians() - p1.x.to_radians()) * p2.y.to_radians().sin();
        }

        total = total * T::from(EQUATORIAL_EARTH_RADIUS).unwrap() * T::from(EQUATORIAL_EARTH_RADIUS).unwrap() / T::from(2).unwrap();
    }
    total
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::polygon;

    #[test]
    fn test_no_holes() {
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
        assert_eq!(-7766240997209.013, polygon.chamberlain_duquette_area());
    }
}