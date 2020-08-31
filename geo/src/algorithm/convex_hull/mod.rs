use crate::*;
use super::kernels::*;

pub trait ConvexHull {
    /// Returns the convex hull of a Polygon. The hull is always oriented counter-clockwise.
    ///
    /// This implementation uses the QuickHull algorithm,
    /// based on [Barber, C. Bradford; Dobkin, David P.; Huhdanpaa, Hannu (1 December 1996)](https://dx.doi.org/10.1145%2F235815.235821)
    /// Original paper here: http://www.cs.princeton.edu/~dpd/Papers/BarberDobkinHuhdanpaa.pdf
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::{line_string, polygon};
    /// use geo::algorithm::convex_hull::ConvexHull;
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
    ///     (x: 0.0, y: 0.0),
    ///     (x: 4.0, y: 0.0),
    ///     (x: 4.0, y: 1.0),
    ///     (x: 1.0, y: 4.0),
    ///     (x: 0.0, y: 4.0),
    ///     (x: 0.0, y: 0.0),
    /// ];
    ///
    /// let res = poly.convex_hull();
    /// assert_eq!(res.exterior(), &correct_hull);
    /// ```
    type Scalar: CoordinateType;
    fn convex_hull(&self) -> Polygon<Self::Scalar>;
}

impl<T> ConvexHull for Polygon<T>
where
    T: HasKernel,
{
    type Scalar = T;
    fn convex_hull(&self) -> Polygon<T> {
        Polygon::new(
            quick_hull(&mut self.exterior().0.clone()),
            vec![],
        )
    }
}

impl<T> ConvexHull for MultiPolygon<T>
where
    T: HasKernel,
{
    type Scalar = T;
    fn convex_hull(&self) -> Polygon<T> {
        let mut aggregated: Vec<_> = self
            .0
            .iter()
            .flat_map(|elem| elem.exterior().0.iter().copied())
            .collect();
        Polygon::new(quick_hull(&mut aggregated), vec![])
    }
}

impl<T> ConvexHull for LineString<T>
where
    T: HasKernel,
{
    type Scalar = T;
    fn convex_hull(&self) -> Polygon<T> {
        Polygon::new(
            quick_hull(&mut self.0.clone()),
            vec![],
        )
    }
}

impl<T> ConvexHull for MultiLineString<T>
where
    T: HasKernel,
{
    type Scalar = T;
    fn convex_hull(&self) -> Polygon<T> {
        let mut aggregated: Vec<_> = self
            .0
            .iter()
            .flat_map(|elem| elem.clone().0)
            .collect();
        Polygon::new(quick_hull(&mut aggregated), vec![])
    }
}

impl<T> ConvexHull for MultiPoint<T>
where
    T: HasKernel,
{
    type Scalar = T;
    fn convex_hull(&self) -> Polygon<T> {
        let mut aggregated: Vec<_> = self
            .0
            .iter()
            .map(|p| p.0)
            .collect();
        Polygon::new(quick_hull(&mut aggregated), vec![])
    }
}

pub mod qhull;
use qhull::quick_hull;

pub mod graham;
pub use graham::graham_hull;

// Helper function that outputs the convex hull in the
// trivial case: input with at most 3 points. It ensures the
// output is ccw, and does not repeat points unless
// required.
fn trivial_hull<T>(
    points: &mut [Coordinate<T>],
    include_on_hull: bool,
) -> LineString<T>
where
    T: HasKernel,
{
    assert!(points.len() < 4);

    // Remove repeated points unless colinear points
    // are to be included.
    let mut ls: LineString<T> = if include_on_hull {
        points
            .iter()
            .copied()
            .collect()
    } else {
        points
            .iter()
            .enumerate()
            .filter_map(|(i, pt)| {
                // Do not care if first and last are
                // same, as we anyway close the linestring
                if i == 0 || pt != &points[i - 1] {
                    Some(*pt)
                } else {
                    None
                }
            })
            .collect()
    };

    ls.close();

    // Maintain the CCW invariance
    use super::winding_order::Winding;
    ls.make_ccw_winding();
    return ls;
}

// Utility function: swap idx to head(0th position), remove
// head (modifies the slice), and return head as a reference
fn swap_remove_to_first<'a, T>(slice: &mut &'a mut [T], idx: usize) -> &'a mut T {
    let tmp = std::mem::replace(slice, &mut []);
    tmp.swap(0, idx);
    let (h, t) = tmp.split_first_mut().unwrap();
    *slice = t;
    h
}

#[cfg(test)]
mod test;
