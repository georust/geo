use num::Float;
use types::{Point, LineString};
use algorithm::distance::Distance;

// perpendicular distance from a point to a line
fn point_line_distance<T>(point: &Point<T>, start: &Point<T>, end: &Point<T>) -> T
    where T: Float
{
    if start == end {
        point.distance(start)
    } else {
        let numerator = ((end.x() - start.x()) * (start.y() - point.y()) -
                         (start.x() - point.x()) * (end.y() - start.y()))
            .abs();
        let denominator = start.distance(end);
        numerator / denominator
    }
}

// Ramer–Douglas-Peucker line simplification algorithm
fn rdp<T>(points: &[Point<T>], epsilon: &T) -> Vec<Point<T>>
    where T: Float
{
    if points.is_empty() {
        return points.to_vec();
    }
    let mut dmax = T::zero();
    let mut index: usize = 0;
    let mut distance: T;

    for (i, _) in points.iter().enumerate().take(points.len() - 1).skip(1) {
        distance = point_line_distance(&points[i],
                                       &points[0],
                                       &*points.last().unwrap());
        if distance > dmax {
            index = i;
            dmax = distance;
        }
    }
    if dmax > *epsilon {
        let mut intermediate = rdp(&points[..index + 1], &*epsilon);
        intermediate.pop();
        intermediate.extend_from_slice(&rdp(&points[index..], &*epsilon));
        intermediate
    } else {
        vec![*points.first().unwrap(), *points.last().unwrap()]
    }
}

pub trait Simplify<T, Epsilon = T> {
    /// Returns the simplified representation of a LineString, using the [Ramer–Douglas–Peucker](https://en.wikipedia.org/wiki/Ramer–Douglas–Peucker_algorithm) algorithm
    ///
    /// ```
    /// use geo::{Point, LineString};
    /// use geo::algorithm::simplify::{Simplify};
    ///
    /// let mut vec = Vec::new();
    /// vec.push(Point::new(0.0, 0.0));
    /// vec.push(Point::new(5.0, 4.0));
    /// vec.push(Point::new(11.0, 5.5));
    /// vec.push(Point::new(17.3, 3.2));
    /// vec.push(Point::new(27.8, 0.1));
    /// let linestring = LineString(vec);
    /// let mut compare = Vec::new();
    /// compare.push(Point::new(0.0, 0.0));
    /// compare.push(Point::new(5.0, 4.0));
    /// compare.push(Point::new(11.0, 5.5));
    /// compare.push(Point::new(27.8, 0.1));
    /// let ls_compare = LineString(compare);
    /// let simplified = linestring.simplify(&1.0);
    /// assert_eq!(simplified, ls_compare)
    /// ```
    fn simplify(&self, epsilon: &T) -> Self where T: Float;
}

impl<T> Simplify<T> for LineString<T>
    where T: Float
{
    fn simplify(&self, epsilon: &T) -> LineString<T> {
        LineString(rdp(&self.0, epsilon))
    }
}

mod test {
    use types::{Point, LineString};
    use super::{point_line_distance, rdp};
    #[test]
    fn perpdistance_test() {
        let start = Point::new(1.0, 2.0);
        let end = Point::new(3.0, 4.0);
        let p = Point::new(1.0, 1.0);
        let dist = point_line_distance(&p, &start, &end);
        assert_eq!(dist, 0.7071067811865475);
    }
    #[test]
    fn rdp_test() {
        let mut vec = Vec::new();
        vec.push(Point::new(0.0, 0.0));
        vec.push(Point::new(5.0, 4.0));
        vec.push(Point::new(11.0, 5.5));
        vec.push(Point::new(17.3, 3.2));
        vec.push(Point::new(27.8, 0.1));
        let mut compare = Vec::new();
        compare.push(Point::new(0.0, 0.0));
        compare.push(Point::new(5.0, 4.0));
        compare.push(Point::new(11.0, 5.5));
        compare.push(Point::new(27.8, 0.1));
        let simplified = rdp(&vec, &1.0);
        assert_eq!(simplified, compare);
    }
    #[test]
    fn rdp_test_empty_linestring() {
        let mut vec = Vec::new();
        let mut compare = Vec::new();
        let simplified = rdp(&vec, &1.0);
        assert_eq!(simplified, compare);
    }
    #[test]
    fn rdp_test_two_point_linestring() {
        let mut vec = Vec::new();
        vec.push(Point::new(0.0, 0.0));
        vec.push(Point::new(27.8, 0.1));
        let mut compare = Vec::new();
        compare.push(Point::new(0.0, 0.0));
        compare.push(Point::new(27.8, 0.1));
        let simplified = rdp(&vec, &1.0);
        assert_eq!(simplified, compare);
    }
}
