use crate::algorithm::{CoordsIter, Distance, Euclidean};
use crate::geometry::{Coord, Line, LineString, MultiLineString, MultiPolygon, Polygon};
use crate::GeoFloat;

const LINE_STRING_INITIAL_MIN: usize = 2;
const POLYGON_INITIAL_MIN: usize = 4;

// Because the RDP algorithm is recursive, we can't assign an index to a point inside the loop
// instead, we wrap a simple struct around index and point in a wrapper function,
// passing that around instead, extracting either points or indices on the way back out
#[derive(Copy, Clone)]
struct RdpIndex<T>
where
    T: GeoFloat,
{
    index: usize,
    coord: Coord<T>,
}

// Wrapper for the RDP algorithm, returning simplified points
fn rdp<T, I: Iterator<Item = Coord<T>>, const INITIAL_MIN: usize>(
    coords: I,
    epsilon: T,
) -> Vec<Coord<T>>
where
    T: GeoFloat,
{
    // Epsilon must be greater than zero for any meaningful simplification to happen
    if epsilon <= T::zero() {
        return coords.collect::<Vec<Coord<T>>>();
    }
    let rdp_indices = &coords
        .enumerate()
        .map(|(idx, coord)| RdpIndex { index: idx, coord })
        .collect::<Vec<RdpIndex<T>>>();
    let mut simplified_len = rdp_indices.len();
    let simplified_coords: Vec<_> =
        compute_rdp::<T, INITIAL_MIN>(rdp_indices, &mut simplified_len, epsilon)
            .into_iter()
            .map(|rdpindex| rdpindex.coord)
            .collect();
    debug_assert_eq!(simplified_coords.len(), simplified_len);
    simplified_coords
}

// Wrapper for the RDP algorithm, returning simplified point indices
fn calculate_rdp_indices<T, const INITIAL_MIN: usize>(
    rdp_indices: &[RdpIndex<T>],
    epsilon: T,
) -> Vec<usize>
where
    T: GeoFloat,
{
    if epsilon <= T::zero() {
        return rdp_indices
            .iter()
            .map(|rdp_index| rdp_index.index)
            .collect();
    }

    let mut simplified_len = rdp_indices.len();
    let simplified_coords =
        compute_rdp::<T, INITIAL_MIN>(rdp_indices, &mut simplified_len, epsilon)
            .into_iter()
            .map(|rdpindex| rdpindex.index)
            .collect::<Vec<usize>>();
    debug_assert_eq!(simplified_len, simplified_coords.len());
    simplified_coords
}

// Ramer–Douglas-Peucker line simplification algorithm
// This function returns both the retained points, and their indices in the original geometry,
// for more flexible use by FFI implementers
fn compute_rdp<T, const INITIAL_MIN: usize>(
    rdp_indices: &[RdpIndex<T>],
    simplified_len: &mut usize,
    epsilon: T,
) -> Vec<RdpIndex<T>>
where
    T: GeoFloat,
{
    if rdp_indices.is_empty() {
        return vec![];
    }

    let first = rdp_indices[0];
    let last = rdp_indices[rdp_indices.len() - 1];
    if rdp_indices.len() == 2 {
        return vec![first, last];
    }

    let first_last_line = Line::new(first.coord, last.coord);

    // Find the farthest `RdpIndex` from `first_last_line`
    let (farthest_index, farthest_distance) = rdp_indices
        .iter()
        .enumerate()
        .take(rdp_indices.len() - 1) // Don't include the last index
        .skip(1) // Don't include the first index
        .map(|(index, rdp_index)| (index, Euclidean.distance(rdp_index.coord, &first_last_line)))
        .fold(
            (0usize, T::zero()),
            |(farthest_index, farthest_distance), (index, distance)| {
                if distance >= farthest_distance {
                    (index, distance)
                } else {
                    (farthest_index, farthest_distance)
                }
            },
        );
    debug_assert_ne!(farthest_index, 0);

    if farthest_distance > epsilon {
        // The farthest index was larger than epsilon, so we will recursively simplify subsegments
        // split by the farthest index.
        let mut intermediate =
            compute_rdp::<T, INITIAL_MIN>(&rdp_indices[..=farthest_index], simplified_len, epsilon);

        intermediate.pop(); // Don't include the farthest index twice

        intermediate.extend_from_slice(&compute_rdp::<T, INITIAL_MIN>(
            &rdp_indices[farthest_index..],
            simplified_len,
            epsilon,
        ));
        return intermediate;
    }

    // The farthest index was less than or equal to epsilon, so we will retain only the first
    // and last indices, resulting in the indices inbetween getting culled.

    // Update `simplified_len` to reflect the new number of indices by subtracting the number
    // of indices we're culling.
    let number_culled = rdp_indices.len() - 2;
    let new_length = *simplified_len - number_culled;

    // If `simplified_len` is now lower than the minimum number of indices needed, then don't
    // perform the culling and return the original input.
    if new_length < INITIAL_MIN {
        return rdp_indices.to_owned();
    }
    *simplified_len = new_length;

    // Cull indices between `first` and `last`.
    vec![first, last]
}

/// Simplifies a geometry.
///
/// The [Ramer–Douglas–Peucker
/// algorithm](https://en.wikipedia.org/wiki/Ramer–Douglas–Peucker_algorithm) simplifies a
/// linestring. Polygons are simplified by running the RDP algorithm on all their constituent
/// rings. This may result in invalid Polygons, and has no guarantee of preserving topology.
///
/// Multi* objects are simplified by simplifying all their constituent geometries individually.
///
/// A larger `epsilon` means being more aggressive about removing points with less concern for
/// maintaining the existing shape.
///
/// Specifically, points closer than `epsilon` distance from the simplified output may be
/// discarded.
///
/// An `epsilon` less than or equal to zero will return an unaltered version of the geometry.
pub trait Simplify<T, Epsilon = T> {
    /// Returns the simplified representation of a geometry, using the [Ramer–Douglas–Peucker](https://en.wikipedia.org/wiki/Ramer–Douglas–Peucker_algorithm) algorithm
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::Simplify;
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
    /// let simplified = line_string.simplify(1.0);
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
    fn simplify(&self, epsilon: T) -> Self
    where
        T: GeoFloat;
}

/// Simplifies a geometry, returning the retained _indices_ of the input.
///
/// This operation uses the [Ramer–Douglas–Peucker algorithm](https://en.wikipedia.org/wiki/Ramer–Douglas–Peucker_algorithm)
/// and does not guarantee that the returned geometry is valid.
///
/// A larger `epsilon` means being more aggressive about removing points with less concern for
/// maintaining the existing shape.
///
/// Specifically, points closer than `epsilon` distance from the simplified output may be
/// discarded.
///
/// An `epsilon` less than or equal to zero will return an unaltered version of the geometry.
pub trait SimplifyIdx<T, Epsilon = T> {
    /// Returns the simplified indices of a geometry, using the [Ramer–Douglas–Peucker](https://en.wikipedia.org/wiki/Ramer–Douglas–Peucker_algorithm) algorithm
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::SimplifyIdx;
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
    /// let simplified = line_string.simplify_idx(1.0);
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
    fn simplify_idx(&self, epsilon: T) -> Vec<usize>
    where
        T: GeoFloat;
}

impl<T> Simplify<T> for LineString<T>
where
    T: GeoFloat,
{
    fn simplify(&self, epsilon: T) -> Self {
        LineString::from(rdp::<_, _, LINE_STRING_INITIAL_MIN>(
            self.coords_iter(),
            epsilon,
        ))
    }
}

impl<T> SimplifyIdx<T> for LineString<T>
where
    T: GeoFloat,
{
    fn simplify_idx(&self, epsilon: T) -> Vec<usize> {
        calculate_rdp_indices::<_, LINE_STRING_INITIAL_MIN>(
            &self
                .0
                .iter()
                .enumerate()
                .map(|(idx, coord)| RdpIndex {
                    index: idx,
                    coord: *coord,
                })
                .collect::<Vec<RdpIndex<T>>>(),
            epsilon,
        )
    }
}

impl<T> Simplify<T> for MultiLineString<T>
where
    T: GeoFloat,
{
    fn simplify(&self, epsilon: T) -> Self {
        MultiLineString::new(self.iter().map(|l| l.simplify(epsilon)).collect())
    }
}

impl<T> Simplify<T> for Polygon<T>
where
    T: GeoFloat,
{
    fn simplify(&self, epsilon: T) -> Self {
        Polygon::new(
            LineString::from(rdp::<_, _, POLYGON_INITIAL_MIN>(
                self.exterior().coords_iter(),
                epsilon,
            )),
            self.interiors()
                .iter()
                .map(|l| {
                    LineString::from(rdp::<_, _, POLYGON_INITIAL_MIN>(l.coords_iter(), epsilon))
                })
                .collect(),
        )
    }
}

impl<T> Simplify<T> for MultiPolygon<T>
where
    T: GeoFloat,
{
    fn simplify(&self, epsilon: T) -> Self {
        MultiPolygon::new(self.iter().map(|p| p.simplify(epsilon)).collect())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{coord, line_string, polygon};

    #[test]
    fn recursion_test() {
        let input = [
            coord! { x: 8.0, y: 100.0 },
            coord! { x: 9.0, y: 100.0 },
            coord! { x: 12.0, y: 100.0 },
        ];
        let actual = rdp::<_, _, 2>(input.into_iter(), 1.0);
        let expected = [coord! { x: 8.0, y: 100.0 }, coord! { x: 12.0, y: 100.0 }];
        assert_eq!(actual, expected);
    }

    #[test]
    fn rdp_test() {
        let vec = vec![
            coord! { x: 0.0, y: 0.0 },
            coord! { x: 5.0, y: 4.0 },
            coord! { x: 11.0, y: 5.5 },
            coord! { x: 17.3, y: 3.2 },
            coord! { x: 27.8, y: 0.1 },
        ];
        let compare = vec![
            coord! { x: 0.0, y: 0.0 },
            coord! { x: 5.0, y: 4.0 },
            coord! { x: 11.0, y: 5.5 },
            coord! { x: 27.8, y: 0.1 },
        ];
        let simplified = rdp::<_, _, 2>(vec.into_iter(), 1.0);
        assert_eq!(simplified, compare);
    }
    #[test]
    fn rdp_test_empty_linestring() {
        let vec = Vec::new();
        let compare = Vec::new();
        let simplified = rdp::<_, _, 2>(vec.into_iter(), 1.0);
        assert_eq!(simplified, compare);
    }
    #[test]
    fn rdp_test_two_point_linestring() {
        let vec = vec![coord! { x: 0.0, y: 0.0 }, coord! { x: 27.8, y: 0.1 }];
        let compare = vec![coord! { x: 0.0, y: 0.0 }, coord! { x: 27.8, y: 0.1 }];
        let simplified = rdp::<_, _, 2>(vec.into_iter(), 1.0);
        assert_eq!(simplified, compare);
    }

    #[test]
    fn multilinestring() {
        let mline = MultiLineString::new(vec![LineString::from(vec![
            (0.0, 0.0),
            (5.0, 4.0),
            (11.0, 5.5),
            (17.3, 3.2),
            (27.8, 0.1),
        ])]);

        let mline2 = mline.simplify(1.0);

        assert_eq!(
            mline2,
            MultiLineString::new(vec![LineString::from(vec![
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

        let poly2 = poly.simplify(2.);

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
        let mpoly = MultiPolygon::new(vec![polygon![
            (x: 0., y: 0.),
            (x: 0., y: 10.),
            (x: 5., y: 11.),
            (x: 10., y: 10.),
            (x: 10., y: 0.),
            (x: 0., y: 0.),
        ]]);

        let mpoly2 = mpoly.simplify(2.);

        assert_eq!(
            mpoly2,
            MultiPolygon::new(vec![polygon![
                (x: 0., y: 0.),
                (x: 0., y: 10.),
                (x: 10., y: 10.),
                (x: 10., y: 0.),
                (x: 0., y: 0.)
            ]]),
        );
    }

    #[test]
    fn simplify_negative_epsilon() {
        let ls = line_string![
            (x: 0., y: 0.),
            (x: 0., y: 10.),
            (x: 5., y: 11.),
            (x: 10., y: 10.),
            (x: 10., y: 0.),
        ];
        let simplified = ls.simplify(-1.0);
        assert_eq!(ls, simplified);
    }

    #[test]
    fn simplify_idx_negative_epsilon() {
        let ls = line_string![
            (x: 0., y: 0.),
            (x: 0., y: 10.),
            (x: 5., y: 11.),
            (x: 10., y: 10.),
            (x: 10., y: 0.),
        ];
        let indices = ls.simplify_idx(-1.0);
        assert_eq!(vec![0usize, 1, 2, 3, 4], indices);
    }

    // https://github.com/georust/geo/issues/142
    #[test]
    fn simplify_line_string_polygon_initial_min() {
        let ls = line_string![
            ( x: 1.4324054e-16, y: 1.4324054e-16 ),
            ( x: 1.4324054e-16, y: 1.4324054e-16 ),
            ( x: -5.9730447e26, y: 1.5590374e-27 ),
            ( x: 1.4324054e-16, y: 1.4324054e-16 ),
        ];
        let epsilon: f64 = 3.46e-43;

        // LineString result should be three coordinates
        let result = ls.simplify(epsilon);
        assert_eq!(
            line_string![
                ( x: 1.4324054e-16, y: 1.4324054e-16 ),
                ( x: -5.9730447e26, y: 1.5590374e-27 ),
                ( x: 1.4324054e-16, y: 1.4324054e-16 ),
            ],
            result
        );

        // Polygon result should be five coordinates
        let result = Polygon::new(ls, vec![]).simplify(epsilon);
        assert_eq!(
            polygon![
                ( x: 1.4324054e-16, y: 1.4324054e-16 ),
                ( x: 1.4324054e-16, y: 1.4324054e-16 ),
                ( x: -5.9730447e26, y: 1.5590374e-27 ),
                ( x: 1.4324054e-16, y: 1.4324054e-16 ),
            ],
            result,
        );
    }

    // https://github.com/georust/geo/issues/995
    #[test]
    fn dont_oversimplify() {
        let unsimplified = line_string![
            (x: 0.0, y: 0.0),
            (x: 5.0, y: 4.0),
            (x: 11.0, y: 5.5),
            (x: 17.3, y: 3.2),
            (x: 27.8, y: 0.1)
        ];
        let actual = unsimplified.simplify(30.0);
        let expected = line_string![
            (x: 0.0, y: 0.0),
            (x: 27.8, y: 0.1)
        ];
        assert_eq!(actual, expected);
    }
}
