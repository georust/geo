use crate::euclidean_length::EuclideanLength;
use crate::prelude::*;
use crate::{Closest, Coordinate, Line, LineString, MultiLineString, Point};
use num_traits::Float;
use std::iter::Sum;
use std::ops::{Add, Div};

pub trait PointLocator<F: Float + Add + Div + Sum, Rhs = Point<F>> {
    /// Return a float between 0 and 1 representing the location of the closest point
    /// on the line to the given point, as a percent of the total line length.
    ///
    /// # Examples
    /// ```
    /// use geo::{Point, LineString, Coordinate};
    /// use geo::algorithm::line_locate_point::PointLocator;
    ///
    /// let the_linestring = LineString(vec![
    ///     Coordinate { x: 0., y: 0. },
    ///     Coordinate { x: 1., y: 0. },
    ///     Coordinate { x: 2., y: 0. },
    /// ]);
    ///
    /// let pt = Point(Coordinate{x: 1.5, y: 1.5});
    ///
    /// println!("Percent Along Line: {}", the_linestring.locate_point(&pt).unwrap());
    ///
    /// ```
    fn locate_point(&self, p: &Rhs) -> Option<F>;
}

impl<F> PointLocator<F> for Line<F>
where
    F: Float + Add + Div + Sum,
{
    fn locate_point(&self, p: &Point<F>) -> Option<F> {
        match self.closest_point(p) {
            Closest::Intersection(pt) | Closest::SinglePoint(pt) => {
                Some((Point(self.start).euclidean_distance(&pt)) / self.euclidean_length())
            }
            _ => None,
        }
    }
}

impl<F> PointLocator<F> for LineString<F>
where
    F: Float + Add + Div + Sum,
{
    fn locate_point(&self, p: &Point<F>) -> Option<F> {
        let distance_along_line: F = self
            .lines()
            .map(|line| line.euclidean_length() * line.locate_point(p).unwrap())
            .sum();

        Some(distance_along_line / self.euclidean_length())
    }
}

//impl<F> PointLocator<F> for MultiLineString<F>
//where
//    F: Float + Add + Div + Sum,
//{
//    fn locate_point(&self, p: &Point<F>) -> Option<F> {}
//}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_line() {
        let the_line = Line::new(Coordinate { x: 0., y: 0. }, Coordinate { x: 2., y: 0. });

        let p1 = Point::new(0., 1.);
        assert_eq!(the_line.locate_point(&p1).unwrap(), 0.0);

        let p2 = Point::new(1., 1.);
        assert_eq!(the_line.locate_point(&p2).unwrap(), 0.5);

        let p3 = Point::new(1.5, 1.5);
        assert_eq!(the_line.locate_point(&p3).unwrap(), 0.75);

        let p4 = Point::new(2., 2.);
        assert_eq!(the_line.locate_point(&p4).unwrap(), 1.0);
    }

    #[test]
    fn test_linestring() {
        let the_linestring = LineString(vec![
            Coordinate { x: 0., y: 0. },
            Coordinate { x: 1., y: 0. },
            Coordinate { x: 2., y: 0. },
        ]);

        let p1 = Point::new(0., 1.);
        assert_eq!(the_linestring.locate_point(&p1).unwrap(), 0.0);

        let p2 = Point::new(1., 1.);
        assert_eq!(the_linestring.locate_point(&p2).unwrap(), 0.5);

        let p3 = Point::new(1.5, 1.5);
        assert_eq!(the_linestring.locate_point(&p3).unwrap(), 0.75);

        let p4 = Point::new(2., 2.);
        assert_eq!(the_linestring.locate_point(&p4).unwrap(), 1.0);
    }

}
