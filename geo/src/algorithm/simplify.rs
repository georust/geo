use crate::algorithm::euclidean_distance::EuclideanDistance;
use crate::{Line, LineString, MultiLineString, MultiPolygon, Point, Polygon};
use num_traits::Float;

// Because the RDP algorithm is recursive, we can't assign an index to a point inside the loop
// instead, we wrap a simple struct around index and point in a wrapper function,
// passing that around instead, extracting either points or indices on the way back out
#[derive(Copy, Clone)]
struct RdpIndex<T>
where
    T: Float,
{
    index: usize,
    point: Point<T>,
}

// Wrapper for the RDP algorithm, returning simplified points
fn rdp<T>(points: &[Point<T>], epsilon: &T) -> Vec<Point<T>>
where
    T: Float,
{
    // Epsilon must be greater than zero for any meaningful simplification to happen
    if *epsilon <= T::zero() {
        points.to_vec();
    }
    compute_rdp(
        &points
            .into_iter()
            .enumerate()
            .map(|(idx, point)| RdpIndex {
                index: idx,
                point: *point,
            })
            .collect::<Vec<RdpIndex<T>>>(),
        epsilon,
    )
    .into_iter()
    .map(|rdpindex| rdpindex.point)
    .collect::<Vec<Point<T>>>()
}

// Wrapper for the RDP algorithm, returning simplified point indices
fn rdp_indices<T>(points: &[RdpIndex<T>], epsilon: &T) -> Vec<usize>
where
    T: Float,
{
    compute_rdp(points, epsilon)
        .iter()
        .map(|rdpindex| rdpindex.index)
        .collect::<Vec<usize>>()
}

// Ramer–Douglas-Peucker line simplification algorithm
// This function returns both the retained points, and their indices in the original geometry,
// for more flexible use by FFI implementers
fn compute_rdp<T>(points: &[RdpIndex<T>], epsilon: &T) -> Vec<RdpIndex<T>>
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
        distance = points[i]
            .point
            .euclidean_distance(&Line::new(points[0].point, points.last().unwrap().point));
        if distance > dmax {
            index = i;
            dmax = distance;
        }
    }
    if dmax > *epsilon {
        let mut intermediate = compute_rdp(&points[..=index], &*epsilon);
        intermediate.pop();
        intermediate.extend_from_slice(&compute_rdp(&points[index..], &*epsilon));
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
///
/// An epsilon less than or equal to zero will return an unaltered version of the geometry.
pub trait Simplify<T, Epsilon = T> {
    /// Returns the simplified representation of a geometry, using the [Ramer–Douglas–Peucker](https://en.wikipedia.org/wiki/Ramer–Douglas–Peucker_algorithm) algorithm
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::algorithm::simplify::Simplify;
    /// use geo::line_string;
    ///
    /// let line_string = line_string![
    ///     (x: 0.0, y: 0.0),
    ///     (x: 5.0, y: 4.0),
    ///     (x: 11.0, y: 5.5),
    ///     (x: 17.3, y: 3.2),
    ///     (x: 27.8, y: 0.1),
    /// ];
    ///
    /// let simplified = line_string.simplify(&1.0);
    ///
    /// let expected = line_string![
    ///     (x: 0.0, y: 0.0),
    ///     (x: 5.0, y: 4.0),
    ///     (x: 11.0, y: 5.5),
    ///     (x: 27.8, y: 0.1),
    /// ];
    ///
    /// assert_eq!(expected, simplified)
    /// ```
    fn simplify(&self, epsilon: &T) -> Self
    where
        T: Float;
}

/// Simplifies a geometry, returning the retained _indices_ of the input.
///
/// This operation uses the [Ramer–Douglas–Peucker algorithm](https://en.wikipedia.org/wiki/Ramer–Douglas–Peucker_algorithm)
/// and does not guarantee that the returned geometry is valid.
///
/// An epsilon less than or equal to zero will return an unaltered version of the geometry.
pub trait SimplifyIdx<T, Epsilon = T> {
    /// Returns the simplified indices of a geometry, using the [Ramer–Douglas–Peucker](https://en.wikipedia.org/wiki/Ramer–Douglas–Peucker_algorithm) algorithm
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::algorithm::simplify::SimplifyIdx;
    /// use geo::line_string;
    ///
    /// let line_string = line_string![
    ///     (x: 0.0, y: 0.0),
    ///     (x: 5.0, y: 4.0),
    ///     (x: 11.0, y: 5.5),
    ///     (x: 17.3, y: 3.2),
    ///     (x: 27.8, y: 0.1),
    /// ];
    ///
    /// let simplified = line_string.simplify_idx(&1.0);
    ///
    /// let expected = vec![
    ///     0_usize,
    ///     1_usize,
    ///     2_usize,
    ///     4_usize,
    /// ];
    ///
    /// assert_eq!(expected, simplified);
    /// ```
    fn simplify_idx(&self, epsilon: &T) -> Vec<usize>
    where
        T: Float;
}

impl<T> Simplify<T> for LineString<T>
where
    T: Float,
{
    fn simplify(&self, epsilon: &T) -> Self {
        LineString::from(rdp(&self.clone().into_points(), epsilon))
    }
}

impl<T> SimplifyIdx<T> for LineString<T>
where
    T: Float,
{
    fn simplify_idx(&self, epsilon: &T) -> Vec<usize> {
        rdp_indices(
            &self
                .points_iter()
                .enumerate()
                .map(|(idx, point)| RdpIndex { index: idx, point })
                .collect::<Vec<RdpIndex<T>>>(),
            epsilon,
        )
    }
}

impl<T> Simplify<T> for MultiLineString<T>
where
    T: Float,
{
    fn simplify(&self, epsilon: &T) -> Self {
        MultiLineString(self.iter().map(|l| l.simplify(epsilon)).collect())
    }
}

impl<T> Simplify<T> for Polygon<T>
where
    T: Float,
{
    fn simplify(&self, epsilon: &T) -> Self {
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
    fn simplify(&self, epsilon: &T) -> Self {
        MultiPolygon(self.iter().map(|p| p.simplify(epsilon)).collect())
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
