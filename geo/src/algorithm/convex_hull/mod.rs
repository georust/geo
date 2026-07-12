use crate::GeoNum;
use crate::geometry::{Coord, LineString, Polygon};
use crate::kernels::*;

/// Returns the convex hull of a geometry. The hull is always oriented counter-clockwise.
///
/// This implementation uses the QuickHull algorithm,
/// based on [Barber, C. Bradford; Dobkin, David P.; Huhdanpaa, Hannu (1 December 1996)](https://dx.doi.org/10.1145%2F235815.235821)
/// Original paper here: <http://www.cs.princeton.edu/~dpd/Papers/BarberDobkinHuhdanpaa.pdf>
///
/// # Examples
///
/// ```
/// use geo::{line_string, polygon};
/// use geo::ConvexHull;
///
/// // an L shape
/// let poly = polygon![
///     (x: 0.0, y: 0.0),
///     (x: 4.0, y: 0.0),
///     (x: 4.0, y: 1.0),
///     (x: 1.0, y: 1.0),
///     (x: 1.0, y: 4.0),
///     (x: 0.0, y: 4.0),
///     (x: 0.0, y: 0.0),
/// ];
///
/// // The correct convex hull coordinates
/// let correct_hull = line_string![
///     (x: 4.0, y: 0.0),
///     (x: 4.0, y: 1.0),
///     (x: 1.0, y: 4.0),
///     (x: 0.0, y: 4.0),
///     (x: 0.0, y: 0.0),
///     (x: 4.0, y: 0.0),
/// ];
///
/// let res = poly.convex_hull();
/// assert_eq!(res.exterior(), &correct_hull);
/// assert_eq!(res.interiors(), &[]);
/// ```
pub trait ConvexHull<'a, T> {
    type Scalar: GeoNum;
    fn convex_hull(&'a self) -> Polygon<Self::Scalar>;

    /// Returns the indices of the input coords (as yielded by
    /// [`CoordsIter::exterior_coords_iter`]) that form the convex hull, in CCW
    /// order and closed (the first index is repeated at the end). Useful for
    /// carrying per-vertex data into the hull without rebuilding a coord buffer.
    fn convex_hull_idx(&'a self) -> Vec<usize>;
}

use crate::algorithm::CoordsIter;
use crate::utils::lex_cmp;

impl<'a, T, G> ConvexHull<'a, T> for G
where
    T: GeoNum,
    G: CoordsIter<Scalar = T>,
{
    type Scalar = T;

    fn convex_hull(&'a self) -> Polygon<T> {
        let mut exterior: Vec<_> = self.exterior_coords_iter().collect();
        Polygon::new(quick_hull(&mut exterior), vec![])
    }

    fn convex_hull_idx(&'a self) -> Vec<usize> {
        let coords: Vec<Coord<T>> = self.exterior_coords_iter().collect();
        qhull::quick_hull_indices(&coords)
    }
}

pub mod qhull;
pub use qhull::{quick_hull, quick_hull_indices};

pub mod graham;
pub use graham::graham_hull;

// Helper function that outputs the convex hull in the
// trivial case: input with at most 3 points. It ensures the
// output is ccw, and does not repeat points unless
// required.
fn trivial_hull<T>(points: &mut [Coord<T>], include_on_hull: bool) -> LineString<T>
where
    T: GeoNum,
{
    assert!(points.len() < 4);

    // Remove repeated points unless collinear points
    // are to be included.
    let mut ls: Vec<Coord<T>> = points.to_vec();
    if !include_on_hull {
        ls.sort_unstable_by(lex_cmp);
        if ls.len() == 3 && T::Ker::orient2d(ls[0], ls[1], ls[2]) == Orientation::Collinear {
            ls.remove(1);
        }
    }

    // A linestring with a single point is invalid.
    if ls.len() == 1 {
        ls.push(ls[0]);
    }

    let mut ls = LineString::new(ls);
    ls.close();

    // Maintain the CCW invariance
    use super::winding_order::Winding;
    ls.make_ccw_winding();
    ls
}

/// Utility function for convex hull ops
///
/// 1. _swap_ the element at `idx` with the element at `head` (0th position)
/// 2. remove the _new_ `head` element (modifying the slice)
/// 3. return a _mutable ref_ to the removed head element
fn swap_with_first_and_remove<'a, T>(slice: &mut &'a mut [T], idx: usize) -> &'a mut T {
    // temporarily replace `slice` with an empty value
    let tmp = std::mem::take(slice);
    tmp.swap(0, idx);
    let (h, t) = tmp.split_first_mut().unwrap();
    *slice = t;
    h
}

#[cfg(test)]
mod test;

#[cfg(test)]
mod idx_tests {
    use super::ConvexHull;
    use crate::algorithm::CoordsIter;
    use crate::{MultiPoint, wkt};

    // Four square corners + one strictly interior point at index 4.
    fn fixture() -> MultiPoint<f64> {
        wkt!(MULTIPOINT(0. 0.,10. 0.,10. 10.,0. 10.,5. 5.))
    }

    #[test]
    fn convex_hull_idx_matches_convex_hull_coords() {
        let mp = fixture();
        let coords: Vec<_> = mp.exterior_coords_iter().collect();
        let from_idx: Vec<_> = mp
            .convex_hull_idx()
            .into_iter()
            .map(|i| coords[i])
            .collect();
        let from_hull = mp.convex_hull().exterior().0.clone();
        assert_eq!(from_idx, from_hull);
    }

    #[test]
    fn convex_hull_idx_drops_interior_point() {
        let mp = fixture();
        assert!(!mp.convex_hull_idx().contains(&4));
    }

    #[test]
    fn convex_hull_idx_polygon() {
        let p = wkt!(POLYGON((0. 0.,10. 0.,10. 10.,0. 10.,0. 0.)));
        // CCW hull perimeter, closed: corners at exterior-coord indices
        // 1 (10 0), 2 (10 10), 3 (0 10), 0 (0 0), back to 1.
        assert_eq!(p.convex_hull_idx(), vec![1, 2, 3, 0, 1]);
    }

    #[test]
    fn convex_hull_idx_linestring() {
        // Index 2 (5 5) is strictly interior and must be dropped.
        let ls = wkt!(LINESTRING(0. 0.,10. 0.,5. 5.,10. 10.,0. 10.));
        assert_eq!(ls.convex_hull_idx(), vec![1, 3, 4, 0, 1]);
    }
}
