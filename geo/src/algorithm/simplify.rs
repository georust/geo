use crate::GeoFloat;
use crate::algorithm::{CoordsIter, Distance, Euclidean};
use crate::geometry::{Coord, Line, LineString, MultiLineString, MultiPolygon, Polygon};

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
    let (first, last) = match rdp_indices {
        [] => return vec![],
        &[only] => return vec![only],
        &[first, last] => return vec![first, last],
        &[first, .., last] => (first, last),
    };

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

/// Per-ring retained-index output for the index-returning simplification
/// methods (`simplify_idx`, `simplify_vw_idx`, `simplify_vw_preserve_idx`) when
/// applied to a `Polygon`. The exterior and interior ring indices are kept
/// distinct rather than flattened into a single `Vec<usize>`; each index is
/// relative to its own ring's coordinate sequence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolygonIndices {
    exterior: Vec<usize>,
    interiors: Vec<Vec<usize>>,
}

impl PolygonIndices {
    pub(crate) fn new(exterior: Vec<usize>, interiors: Vec<Vec<usize>>) -> Self {
        Self {
            exterior,
            interiors,
        }
    }

    /// The retained indices of the polygon's exterior ring, relative to the
    /// exterior ring's own coordinate sequence.
    pub fn exterior(&self) -> &[usize] {
        &self.exterior
    }

    /// The retained indices of the polygon's interior rings, one `Vec` per
    /// interior ring, each relative to its own ring's coordinate sequence.
    pub fn interiors(&self) -> &[Vec<usize>] {
        &self.interiors
    }
}

// Indices are relative to this ring's own coord sequence, not polygon-global.
fn rdp_indices<T, const INITIAL_MIN: usize>(ring: &LineString<T>, epsilon: T) -> Vec<usize>
where
    T: GeoFloat,
{
    calculate_rdp_indices::<_, INITIAL_MIN>(
        &ring
            .0
            .iter()
            .enumerate()
            .map(|(index, &coord)| RdpIndex { index, coord })
            .collect::<Vec<RdpIndex<T>>>(),
        epsilon,
    )
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
    /// The index-output type of [`Simplify::simplify_idx`], which varies by geometry.
    type IndexOutput;

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

    /// Returns the indices of the points retained by [`Simplify::simplify`],
    /// relative to the input geometry. [`Self::IndexOutput`] is `Vec<usize>` for
    /// `LineString`, `Vec<Vec<usize>>` for `MultiLineString`, [`PolygonIndices`]
    /// for `Polygon`, and `Vec<PolygonIndices>` for `MultiPolygon`.
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
    /// let indices = line_string.simplify_idx(1.0);
    ///
    /// assert_eq!(indices, vec![0_usize, 1, 2, 4]);
    /// ```
    fn simplify_idx(&self, epsilon: T) -> Self::IndexOutput
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
#[deprecated(
    since = "0.34.0",
    note = "Please use the `simplify_idx` method from the `Simplify` trait instead"
)]
pub trait SimplifyIdx<T, Epsilon = T> {
    /// Returns the simplified indices of a geometry, using the [Ramer–Douglas–Peucker](https://en.wikipedia.org/wiki/Ramer–Douglas–Peucker_algorithm) algorithm
    ///
    /// # Examples
    ///
    /// ```
    /// #![allow(deprecated)]
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
    type IndexOutput = Vec<usize>;

    fn simplify(&self, epsilon: T) -> Self {
        LineString::from(rdp::<_, _, LINE_STRING_INITIAL_MIN>(
            self.coords_iter(),
            epsilon,
        ))
    }

    fn simplify_idx(&self, epsilon: T) -> Vec<usize> {
        rdp_indices::<_, LINE_STRING_INITIAL_MIN>(self, epsilon)
    }
}

#[allow(deprecated)]
impl<T> SimplifyIdx<T> for LineString<T>
where
    T: GeoFloat,
{
    fn simplify_idx(&self, epsilon: T) -> Vec<usize> {
        Simplify::simplify_idx(self, epsilon)
    }
}

impl<T> Simplify<T> for MultiLineString<T>
where
    T: GeoFloat,
{
    type IndexOutput = Vec<Vec<usize>>;

    fn simplify(&self, epsilon: T) -> Self {
        MultiLineString::new(self.iter().map(|l| l.simplify(epsilon)).collect())
    }

    fn simplify_idx(&self, epsilon: T) -> Vec<Vec<usize>> {
        self.iter()
            .map(|l| Simplify::simplify_idx(l, epsilon))
            .collect()
    }
}

impl<T> Simplify<T> for Polygon<T>
where
    T: GeoFloat,
{
    type IndexOutput = PolygonIndices;

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

    fn simplify_idx(&self, epsilon: T) -> PolygonIndices {
        PolygonIndices::new(
            rdp_indices::<_, POLYGON_INITIAL_MIN>(self.exterior(), epsilon),
            self.interiors()
                .iter()
                .map(|l| rdp_indices::<_, POLYGON_INITIAL_MIN>(l, epsilon))
                .collect(),
        )
    }
}

impl<T> Simplify<T> for MultiPolygon<T>
where
    T: GeoFloat,
{
    type IndexOutput = Vec<PolygonIndices>;

    fn simplify(&self, epsilon: T) -> Self {
        MultiPolygon::new(self.iter().map(|p| p.simplify(epsilon)).collect())
    }

    fn simplify_idx(&self, epsilon: T) -> Vec<PolygonIndices> {
        self.iter()
            .map(|p| Simplify::simplify_idx(p, epsilon))
            .collect()
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
    fn rdp_test_one_point_linestring() {
        let vec = vec![coord! { x: 27.8, y: 0.1 }];
        let compare = vec![coord! { x: 27.8, y: 0.1 }];
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
        let indices = Simplify::simplify_idx(&ls, -1.0);
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

#[cfg(test)]
mod hegel_props {
    use super::Simplify;
    use crate::utils::hegel_gens::{line_strings, monotone_line_strings, star_polygons};
    use crate::{Coord, Euclidean, Length, LineString};
    use hegel::generators;

    fn epsilons(tc: &hegel::TestCase) -> f64 {
        tc.draw(hegel::one_of!(
            generators::floats::<f64>().min_value(0.0).max_value(1e4),
            generators::floats::<f64>().min_value(-1e4).max_value(0.0),
        ))
    }

    // `simplify_idx` returns "the indices of the points retained by
    // `Simplify::simplify`, relative to the input geometry", so mapping the
    // indices back through the input must reproduce `simplify`'s output, and
    // the indices must be strictly increasing and in bounds.
    #[hegel::test]
    fn simplify_idx_indexes_the_output_of_simplify(tc: hegel::TestCase) {
        let line_string = tc.draw(line_strings(1e6, 24));
        let epsilon = epsilons(&tc);
        let indices = Simplify::simplify_idx(&line_string, epsilon);
        assert!(
            indices.windows(2).all(|w| w[0] < w[1]),
            "indices are not strictly increasing: {indices:?}"
        );
        let indexed: Vec<Coord<f64>> = indices.iter().map(|&i| line_string.0[i]).collect();
        assert_eq!(indexed, line_string.simplify(epsilon).0);
    }

    // "An `epsilon` less than or equal to zero will return an unaltered version
    // of the geometry."
    #[hegel::test]
    fn a_non_positive_epsilon_leaves_the_geometry_alone(tc: hegel::TestCase) {
        let line_string = tc.draw(line_strings(1e6, 24));
        let epsilon = tc.draw(generators::floats::<f64>().min_value(-1e6).max_value(0.0));
        assert_eq!(line_string.simplify(epsilon), line_string);
    }

    // RDP only ever drops vertices, so the output is a subsequence of the input.
    #[hegel::test]
    fn simplify_retains_a_subsequence_of_the_input(tc: hegel::TestCase) {
        let line_string = tc.draw(line_strings(1e6, 24));
        let epsilon = epsilons(&tc);
        let simplified = line_string.simplify(epsilon);
        let mut remaining = line_string.0.iter();
        for coord in &simplified.0 {
            assert!(
                remaining.any(|candidate| candidate == coord),
                "{simplified:?} is not a subsequence of {line_string:?}"
            );
        }
    }

    // Retaining a subsequence of vertices can only shorten the polyline, by the
    // triangle inequality — the property `fuzz/fuzz_targets/simplify.rs` checks
    // for polygon rings.
    //
    // Coordinates are capped at 1e150 to keep the cross product inside
    // `Euclidean.distance(coord, &Line)` clear of overflow; see
    // `simplify_drops_a_point_farther_than_epsilon` below.
    #[hegel::test]
    fn simplify_never_lengthens_a_line_string(tc: hegel::TestCase) {
        let line_string = tc.draw(line_strings(1e150, 24));
        let epsilon = epsilons(&tc);
        let simplified = line_string.simplify(epsilon);
        let before = Euclidean.length(&line_string);
        let after = Euclidean.length(&simplified);
        assert!(
            after <= before * (1.0 + 1e-9) + 1e-9,
            "simplification lengthened the line string: {before} -> {after}"
        );
    }

    // However aggressive the epsilon, a simplified ring keeps enough vertices
    // to stay a ring: `POLYGON_INITIAL_MIN` is 4, which is what
    // `simplify_line_string_polygon_initial_min` and `dont_oversimplify` above
    // pin for particular inputs.
    #[hegel::test]
    fn simplifying_a_polygon_leaves_each_ring_with_four_coords(tc: hegel::TestCase) {
        let polygon = tc.draw(star_polygons());
        let epsilon = epsilons(&tc);
        let simplified = polygon.simplify(epsilon);
        assert!(simplified.exterior().0.len() >= 4);
        assert!(simplified.exterior().is_closed());
    }

    // Every retained vertex is at most `epsilon` from the simplified output:
    // "points closer than `epsilon` distance from the simplified output may be
    // discarded", so a vertex farther than `epsilon` may not be. Distances are
    // measured against the output polyline with an independent point-to-segment
    // formula.
    //
    // The input is x-monotone so that the discarded vertex's nearest point on
    // the output is on a segment RDP actually considered; a self-crossing input
    // can put a dropped vertex far from the output without RDP being wrong.
    #[hegel::test]
    fn discarded_vertices_are_within_epsilon_of_the_output(tc: hegel::TestCase) {
        let line_string = tc.draw(monotone_line_strings(1e3, 16));
        let epsilon = tc.draw(generators::floats::<f64>().min_value(1e-3).max_value(1e3));
        let simplified = line_string.simplify(epsilon);
        for coord in &line_string.0 {
            let distance = distance_to_polyline(*coord, &simplified);
            assert!(
                distance <= epsilon * (1.0 + 1e-9) + 1e-9,
                "{coord:?} is {distance} from the simplified output, beyond epsilon {epsilon}"
            );
        }
    }

    /// Least distance from `coord` to any segment of `line_string`, by the
    /// projection formula, so the check does not reuse the distance code
    /// `compute_rdp` calls.
    fn distance_to_polyline(coord: Coord<f64>, line_string: &LineString<f64>) -> f64 {
        line_string
            .lines()
            .map(|line| {
                let d = line.end - line.start;
                let length_squared = d.x * d.x + d.y * d.y;
                let t = if length_squared == 0.0 {
                    0.0
                } else {
                    (((coord - line.start).x * d.x + (coord - line.start).y * d.y) / length_squared)
                        .clamp(0.0, 1.0)
                };
                let nearest = line.start + d * t;
                (coord - nearest).x.hypot((coord - nearest).y)
            })
            .fold(f64::INFINITY, f64::min)
    }

    // KNOWN FAILURE, #1606: `simplify` drops the middle vertex, which
    // sits about 3.0 from the output, for an `epsilon` of 2.0. Coordinates this
    // large also trip the library's own `debug_assert_ne!(farthest_index, 0)`
    // above, so this test panics rather than failing its assertion.
    #[test]
    #[ignore = "#1606: simplify drops a vertex farther than epsilon at large coordinates"]
    fn simplify_drops_a_point_farther_than_epsilon() {
        let line_string = LineString::from(vec![
            Coord {
                x: 0.0,
                y: -8.988465674311469e307,
            },
            Coord { x: 0.0, y: 0.0 },
            Coord { x: 3.0, y: -0.0 },
        ]);
        let simplified = line_string.simplify(2.0);
        for coord in &line_string.0 {
            assert!(distance_to_polyline(*coord, &simplified) <= 2.0);
        }
    }
}
