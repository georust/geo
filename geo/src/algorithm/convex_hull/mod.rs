use crate::geometry::{Coordinate, LineString, Polygon};
use crate::kernels::*;
use crate::GeoNum;

/// Returns the convex hull of a Polygon. The hull is always oriented counter-clockwise.
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
    G: CoordsIter<'a, Scalar = T>,
{
    type Scalar = T;

    fn convex_hull(&'a self) -> Polygon<T> {
        let mut exterior: Vec<_> = self.exterior_coords_iter().collect();
        Polygon::new(quick_hull(&mut exterior), vec![])
    }
}

pub mod qhull;
pub use qhull::quick_hull;

pub mod graham;
pub use graham::graham_hull;

// Helper function that outputs the convex hull in the
// trivial case: input with at most 3 points. It ensures the
// output is ccw, and does not repeat points unless
// required.
fn trivial_hull<T>(points: &mut [Coordinate<T>], include_on_hull: bool) -> LineString<T>
where
    T: GeoNum,
{
    assert!(points.len() < 4);

    // Remove repeated points unless collinear points
    // are to be included.
    let mut ls: Vec<Coordinate<T>> = points.to_vec();
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

// Utility function: swap idx to head(0th position), remove
// head (modifies the slice), and return head as a reference
fn swap_remove_to_first<'a, T>(slice: &mut &'a mut [T], idx: usize) -> &'a mut T {
    // temporarily replace `slice` with an empty value
    let tmp = std::mem::take(slice);
    tmp.swap(0, idx);
    let (h, t) = tmp.split_first_mut().unwrap();
    *slice = t;
    h
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::coord;

    #[test]
    fn test_zero_points() {
        let mut v: Vec<Coordinate<i64>> = vec![];
        let correct = vec![];
        let res = trivial_hull(&mut v, false);
        assert_eq!(res.0, correct);
    }

    #[test]
    fn test_zero_points_include_on_hull() {
        let mut v: Vec<Coordinate<i64>> = vec![];
        let correct = vec![];
        let res = trivial_hull(&mut v, true);
        assert_eq!(res.0, correct);
    }

    #[test]
    fn test_one_point() {
        let mut v = vec![coord! { x: 0, y: 0 }];
        let correct = vec![coord! { x: 0, y: 0 }, coord! { x: 0, y: 0 }];
        let res = trivial_hull(&mut v, false);
        assert_eq!(res.0, correct);
    }

    #[test]
    fn test_one_point_include_on_hull() {
        let mut v = vec![coord! { x: 0, y: 0 }];
        let correct = vec![coord! { x: 0, y: 0 }, coord! { x: 0, y: 0 }];
        let res = trivial_hull(&mut v, true);
        assert_eq!(res.0, correct);
    }

    #[test]
    fn test_two_points() {
        let mut v = vec![coord! { x: 0, y: 0 }, coord! { x: 1, y: 1 }];
        let correct = vec![
            coord! { x: 0, y: 0 },
            coord! { x: 1, y: 1 },
            coord! { x: 0, y: 0 },
        ];
        let res = trivial_hull(&mut v, false);
        assert_eq!(res.0, correct);
    }

    #[test]
    fn test_two_points_include_on_hull() {
        let mut v = vec![coord! { x: 0, y: 0 }, coord! { x: 1, y: 1 }];
        let correct = vec![
            coord! { x: 0, y: 0 },
            coord! { x: 1, y: 1 },
            coord! { x: 0, y: 0 },
        ];
        let res = trivial_hull(&mut v, true);
        assert_eq!(res.0, correct);
    }

    #[test]
    fn test_two_points_duplicated() {
        let mut v = vec![coord! { x: 0, y: 0 }, coord! { x: 0, y: 0 }];
        let correct = vec![coord! { x: 0, y: 0 }, coord! { x: 0, y: 0 }];
        let res = trivial_hull(&mut v, false);
        assert_eq!(res.0, correct);
    }

    #[test]
    fn test_two_points_duplicated_include_on_hull() {
        let mut v = vec![coord! { x: 0, y: 0 }, coord! { x: 0, y: 0 }];
        let correct = vec![coord! { x: 0, y: 0 }, coord! { x: 0, y: 0 }];
        let res = trivial_hull(&mut v, true);
        assert_eq!(res.0, correct);
    }

    #[test]
    fn test_three_points_ccw() {
        let mut v = vec![
            coord! { x: 0, y: 0 },
            coord! { x: 1, y: 0 },
            coord! { x: 1, y: 1 },
        ];
        let correct = vec![
            coord! { x: 0, y: 0 },
            coord! { x: 1, y: 0 },
            coord! { x: 1, y: 1 },
            coord! { x: 0, y: 0 },
        ];
        let res = trivial_hull(&mut v, false);
        assert_eq!(res.0, correct);
    }

    #[test]
    fn test_three_points_cw() {
        let mut v = vec![
            coord! { x: 0, y: 0 },
            coord! { x: 1, y: 1 },
            coord! { x: 1, y: 0 },
        ];
        let correct = vec![
            coord! { x: 0, y: 0 },
            coord! { x: 1, y: 0 },
            coord! { x: 1, y: 1 },
            coord! { x: 0, y: 0 },
        ];
        let res = trivial_hull(&mut v, false);
        assert_eq!(res.0, correct);
    }

    #[test]
    fn test_three_points_two_duplicated() {
        let mut v = vec![
            coord! { x: 0, y: 0 },
            coord! { x: 1, y: 1 },
            coord! { x: 0, y: 0 },
        ];
        let correct = vec![
            coord! { x: 0, y: 0 },
            coord! { x: 1, y: 1 },
            coord! { x: 0, y: 0 },
        ];
        let res = trivial_hull(&mut v, false);
        assert_eq!(res.0, correct);
    }

    #[test]
    fn test_three_points_two_duplicated_include_on_hull() {
        let mut v = vec![
            coord! { x: 0, y: 0 },
            coord! { x: 1, y: 1 },
            coord! { x: 0, y: 0 },
        ];
        let correct = vec![
            coord! { x: 0, y: 0 },
            coord! { x: 1, y: 1 },
            coord! { x: 0, y: 0 },
        ];
        let res = trivial_hull(&mut v, true);
        assert_eq!(res.0, correct);
    }

    #[test]
    fn test_three_points_duplicated() {
        let mut v = vec![
            coord! { x: 0, y: 0 },
            coord! { x: 0, y: 0 },
            coord! { x: 0, y: 0 },
        ];
        let correct = vec![coord! { x: 0, y: 0 }, coord! { x: 0, y: 0 }];
        let res = trivial_hull(&mut v, false);
        assert_eq!(res.0, correct);
    }

    #[test]
    fn test_three_points_duplicated_include_on_hull() {
        let mut v = vec![
            coord! { x: 0, y: 0 },
            coord! { x: 0, y: 0 },
            coord! { x: 0, y: 0 },
        ];
        let correct = vec![
            coord! { x: 0, y: 0 },
            coord! { x: 0, y: 0 },
            coord! { x: 0, y: 0 },
        ];
        let res = trivial_hull(&mut v, true);
        assert_eq!(res.0, correct);
    }

    #[test]
    fn test_three_collinear_points() {
        let mut v = vec![
            coord! { x: 0, y: 0 },
            coord! { x: 1, y: 1 },
            coord! { x: 2, y: 2 },
        ];
        let correct = vec![
            coord! { x: 0, y: 0 },
            coord! { x: 2, y: 2 },
            coord! { x: 0, y: 0 },
        ];
        let res = trivial_hull(&mut v, false);
        assert_eq!(res.0, correct);
    }

    #[test]
    fn test_three_collinear_points_include_on_hull() {
        let mut v = vec![
            coord! { x: 0, y: 0 },
            coord! { x: 1, y: 1 },
            coord! { x: 2, y: 2 },
        ];
        let correct = vec![
            coord! { x: 0, y: 0 },
            coord! { x: 1, y: 1 },
            coord! { x: 2, y: 2 },
            coord! { x: 0, y: 0 },
        ];
        let res = trivial_hull(&mut v, true);
        assert_eq!(res.0, correct);
    }
}
