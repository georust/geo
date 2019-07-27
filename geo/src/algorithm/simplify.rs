use crate::algorithm::euclidean_distance::EuclideanDistance;
use crate::{Line, LineString, MultiLineString, MultiPolygon, Point, Polygon};
use num_traits::Float;

// Ramer–Douglas-Peucker line simplification algorithm
fn rdp<T>(points: &[Point<T>], epsilon: &T) -> Vec<Point<T>>
where
    T: Float,
{
    if points.is_empty() {
        return points.to_vec();
    }
    let mut dmax = T::zero();
    let mut index: usize = 0;
    let mut distance: T;

    for (i, _) in points.iter().enumerate().take(points.len() - 1).skip(1) {
        distance = points[i].euclidean_distance(&Line::new(points[0].0, points.last().unwrap().0));
        if distance > dmax {
            index = i;
            dmax = distance;
        }
    }
    if dmax > *epsilon {
        let mut intermediate = rdp(&points[..=index], &*epsilon);
        intermediate.pop();
        intermediate.extend_from_slice(&rdp(&points[index..], &*epsilon));
        intermediate
    } else {
        vec![*points.first().unwrap(), *points.last().unwrap()]
    }
}

/// Simplifies a geometry.
///
/// The [Ramer–Douglas–Peucker
/// algorithm](https://en.wikipedia.org/wiki/Ramer–Douglas–Peucker_algorithm) simplifes a
/// linestring. Polygons are simplified by running the RDP algorithm on all their constituent
/// rings. This may result in invalid Polygons, and has no guarantee of preserving topology.
///
/// Multi* objects are simplified by simplifing all their constituent geometries individually.
pub trait Simplify<T, Epsilon = T> {
    /// Returns the simplified representation of a geometry, using the [Ramer–Douglas–Peucker](https://en.wikipedia.org/wiki/Ramer–Douglas–Peucker_algorithm) algorithm
    ///
    /// # Examples
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
    /// let linestring = LineString::from(vec);
    /// let mut compare = Vec::new();
    /// compare.push(Point::new(0.0, 0.0));
    /// compare.push(Point::new(5.0, 4.0));
    /// compare.push(Point::new(11.0, 5.5));
    /// compare.push(Point::new(27.8, 0.1));
    /// let ls_compare = LineString::from(compare);
    /// let simplified = linestring.simplify(&1.0);
    /// assert_eq!(simplified, ls_compare)
    /// ```
    fn simplify(&self, epsilon: &T) -> Self
    where
        T: Float;
}

impl<T> Simplify<T> for LineString<T>
where
    T: Float,
{
    fn simplify(&self, epsilon: &T) -> LineString<T> {
        LineString::from(rdp(&self.clone().into_points(), epsilon))
    }
}

impl<T> Simplify<T> for MultiLineString<T>
where
    T: Float,
{
    fn simplify(&self, epsilon: &T) -> MultiLineString<T> {
        MultiLineString(self.0.iter().map(|l| l.simplify(epsilon)).collect())
    }
}

impl<T> Simplify<T> for Polygon<T>
where
    T: Float,
{
    fn simplify(&self, epsilon: &T) -> Polygon<T> {
        Polygon::new(
            self.exterior().simplify(epsilon),
            self.interiors()
                .iter()
                .map(|l| l.simplify(epsilon))
                .collect(),
        )
    }
}

impl<T> Simplify<T> for MultiPolygon<T>
where
    T: Float,
{
    fn simplify(&self, epsilon: &T) -> MultiPolygon<T> {
        MultiPolygon(self.0.iter().map(|p| p.simplify(epsilon)).collect())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::polygon;

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
        let vec = Vec::new();
        let compare = Vec::new();
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

    #[test]
    fn multilinestring() {
        let mline = MultiLineString(vec![LineString::from(vec![
            (0.0, 0.0),
            (5.0, 4.0),
            (11.0, 5.5),
            (17.3, 3.2),
            (27.8, 0.1),
        ])]);

        let mline2 = mline.simplify(&1.0);

        assert_eq!(
            mline2,
            MultiLineString(vec![LineString::from(vec![
                (0.0, 0.0),
                (5.0, 4.0),
                (11.0, 5.5),
                (27.8, 0.1),
            ])])
        );
    }

    #[test]
    fn polygon() {
        let poly = polygon![
            (x: 0., y: 0.),
            (x: 0., y: 10.),
            (x: 5., y: 11.),
            (x: 10., y: 10.),
            (x: 10., y: 0.),
            (x: 0., y: 0.),
        ];

        let poly2 = poly.simplify(&2.);

        assert_eq!(
            poly2,
            polygon![
                (x: 0., y: 0.),
                (x: 0., y: 10.),
                (x: 10., y: 10.),
                (x: 10., y: 0.),
                (x: 0., y: 0.),
            ],
        );
    }

    #[test]
    fn multipolygon() {
        let mpoly = MultiPolygon(vec![polygon![
            (x: 0., y: 0.),
            (x: 0., y: 10.),
            (x: 5., y: 11.),
            (x: 10., y: 10.),
            (x: 10., y: 0.),
            (x: 0., y: 0.),
        ]]);

        let mpoly2 = mpoly.simplify(&2.);

        assert_eq!(
            mpoly2,
            MultiPolygon(vec![polygon![
                (x: 0., y: 0.),
                (x: 0., y: 10.),
                (x: 10., y: 10.),
                (x: 10., y: 0.),
                (x: 0., y: 0.)
            ]]),
        );
    }
}
