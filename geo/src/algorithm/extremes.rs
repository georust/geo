use crate::algorithm::coords_iter::CoordsIter;
use crate::{Coordinate, CoordinateType};

/// Find the extreme coordinates and indices of a geometry.
///
/// # Examples
///
/// ```
/// use geo::extremes::Extremes;
/// use geo::polygon;
///
/// // a diamond shape
/// let polygon = polygon![
///     (x: 1.0, y: 0.0),
///     (x: 2.0, y: 1.0),
///     (x: 1.0, y: 2.0),
///     (x: 0.0, y: 1.0),
///     (x: 1.0, y: 0.0),
/// ];
///
/// let extremes = polygon.extremes().unwrap();
///
/// assert_eq!(extremes.y_max.index, 2);
/// assert_eq!(extremes.y_max.coord.x, 1.);
/// assert_eq!(extremes.y_max.coord.x, 2.);
/// ```
pub trait Extremes<'a, T: CoordinateType> {
    fn extremes(&'a self) -> Option<Outcome<T>>;
}

pub struct Extreme<T: CoordinateType> {
    pub index: usize,
    pub coord: Coordinate<T>,
}

pub struct Outcome<T: CoordinateType> {
    pub x_min: Extreme<T>,
    pub y_min: Extreme<T>,
    pub x_max: Extreme<T>,
    pub y_max: Extreme<T>,
}

impl<'a, T, G> Extremes<'a, T> for G
where
    G: CoordsIter<'a, Scalar = T>,
    T: CoordinateType,
{
    fn extremes(&'a self) -> Option<Outcome<T>> {
        let mut iter = self.coords_iter().enumerate();

        let mut outcome = iter.next().map(|(index, coord)| Outcome {
            x_min: Extreme { index, coord },
            y_min: Extreme { index, coord },
            x_max: Extreme { index, coord },
            y_max: Extreme { index, coord },
        })?;

        for (i, coord) in iter {
            if coord.x < outcome.x_min.coord.x {
                outcome.x_min.index = i;
                outcome.x_min.coord = coord;
            }

            if coord.y < outcome.y_min.coord.y {
                outcome.y_min.index = i;
                outcome.y_min.coord = coord;
            }

            if coord.x > outcome.x_max.coord.x {
                outcome.x_max.index = i;
                outcome.x_max.coord = coord;
            }

            if coord.y > outcome.y_max.coord.y {
                outcome.y_max.index = i;
                outcome.y_max.coord = coord;
            }
        }

        Some(outcome)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{point, polygon};

    /*
    #[test]
    fn test_polygon_extreme_x() {
        // a diamond shape
        let poly1 = polygon![
            (x: 1.0, y: 0.0),
            (x: 2.0, y: 1.0),
            (x: 1.0, y: 2.0),
            (x: 0.0, y: 1.0),
            (x: 1.0, y: 0.0)
        ];
        let min_x = polymax_naive_indices(Coordinate { x: -1., y: 0. }, &poly1).unwrap();
        let correct = 3_usize;
        assert_eq!(min_x, correct);
    }
    #[test]
    #[should_panic]
    fn test_extreme_indices_bad_polygon() {
        // non-convex, with a bump on the top-right edge
        let poly1 = polygon![
            (x: 1.0, y: 0.0),
            (x: 1.3, y: 1.),
            (x: 2.0, y: 1.0),
            (x: 1.75, y: 1.75),
            (x: 1.0, y: 2.0),
            (x: 0.0, y: 1.0),
            (x: 1.0, y: 0.0)
        ];
        let extremes = find_extreme_indices(polymax_naive_indices, &poly1).unwrap();
        let correct = Extremes {
            ymin: 0,
            xmax: 1,
            ymax: 3,
            xmin: 4,
        };
        assert_eq!(extremes, correct);
    }
    #[test]
    fn test_extreme_indices_good_polygon() {
        // non-convex, with a bump on the top-right edge
        let poly1 = polygon![
            (x: 1.0, y: 0.0),
            (x: 1.3, y: 1.),
            (x: 2.0, y: 1.0),
            (x: 1.75, y: 1.75),
            (x: 1.0, y: 2.0),
            (x: 0.0, y: 1.0),
            (x: 1.0, y: 0.0)
        ];
        let extremes = find_extreme_indices(polymax_naive_indices, &poly1.convex_hull()).unwrap();
        let correct = Extremes {
            ymin: 0,
            xmax: 1,
            ymax: 3,
            xmin: 4,
        };
        assert_eq!(extremes, correct);
    }
    #[test]
    fn test_polygon_extreme_wrapper_convex() {
        // convex, with a bump on the top-right edge
        let poly1 = polygon![
            (x: 1.0, y: 0.0),
            (x: 2.0, y: 1.0),
            (x: 1.75, y: 1.75),
            (x: 1.0, y: 2.0),
            (x: 0.0, y: 1.0),
            (x: 1.0, y: 0.0)
        ];
        let extremes = find_extreme_indices(polymax_naive_indices, &poly1.convex_hull()).unwrap();
        let correct = Extremes {
            ymin: 0,
            xmax: 1,
            ymax: 3,
            xmin: 4,
        };
        assert_eq!(extremes, correct);
    }
    */

    /*
    #[test]
    fn test_polygon_extreme_point_x() {
        // a diamond shape
        let poly1 = polygon![
            (x: 1.0, y: 0.0),
            (x: 2.0, y: 1.0),
            (x: 1.0, y: 2.0),
            (x: 0.0, y: 1.0),
            (x: 1.0, y: 0.0)
        ];
        let extremes = poly1.extreme_points();
        let correct = point!(x: 0.0, y: 1.0);
        assert_eq!(extremes.xmin, correct);
    }
    */
}
