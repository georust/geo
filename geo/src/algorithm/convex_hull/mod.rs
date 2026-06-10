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

/// Index-tracking analogue of [`ConvexHull`]. Returns input-relative indices
/// of the hull-perimeter coords, in CCW order, closed (first index repeated
/// at the end).
///
/// Useful when per-vertex data alongside the input coords needs to be carried
/// into the hull output without rebuilding a coordinate buffer.
pub trait ConvexHullIdx<'a, T> {
    type Scalar: GeoNum;
    fn convex_hull_idx(&'a self) -> Vec<usize>;
}

impl<'a, T, G> ConvexHullIdx<'a, T> for G
where
    T: GeoNum,
    G: CoordsIter<Scalar = T>,
{
    type Scalar = T;

    fn convex_hull_idx(&'a self) -> Vec<usize> {
        let coords: Vec<Coord<T>> = self.exterior_coords_iter().collect();
        qhull::quick_hull_indices(&coords)
    }
}

#[cfg(test)]
mod test;

#[cfg(test)]
mod idx_tests {
    use super::{ConvexHull, ConvexHullIdx};
    use crate::algorithm::CoordsIter;
    use crate::{MultiPoint, Point, line_string, polygon};

    // Four square corners + one strictly interior point at index 4.
    fn fixture() -> MultiPoint<f64> {
        MultiPoint::from(vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(0.0, 10.0),
            Point::new(5.0, 5.0),
        ])
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
        let p = polygon![
            (x: 0., y: 0.),
            (x: 10., y: 0.),
            (x: 10., y: 10.),
            (x: 0., y: 10.),
            (x: 0., y: 0.),
        ];
        let indices = p.convex_hull_idx();
        let coords: Vec<_> = p.exterior_coords_iter().collect();
        for &i in &indices {
            assert!(i < coords.len(), "index {} out of range", i);
        }
        assert!(indices.len() >= 4);
    }

    #[test]
    fn convex_hull_idx_linestring() {
        let ls = line_string![
            (x: 0., y: 0.),
            (x: 10., y: 0.),
            (x: 5., y: 5.),
            (x: 10., y: 10.),
            (x: 0., y: 10.),
        ];
        let indices = ls.convex_hull_idx();
        assert!(
            !indices.contains(&2),
            "interior vertex should be dropped from hull"
        );
    }
}
