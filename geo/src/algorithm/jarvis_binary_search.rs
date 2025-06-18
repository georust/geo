use std::ops::Div;

use crate::{CoordNum, Distance, Euclidean, GeoFloat, GeoNum, Orientation, Point, Polygon};

use crate::algorithm::kernels::Kernel;

/// Describes the tangent from a point outside of a convex polygon
///
/// Lower tangent is when all points of the polygon are to the left of the tangent.
/// Upper tangent is the opposite.
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Target {
    UpperTangent,
    LowerTangent,
}

/// An implementation of an algorithm meant to find the the lower and upper tangent of a convex hull from a point strictly outside of it.
/// This is based on the (Jarvis March algorithm)[https://en.wikipedia.org/wiki/Gift_wrapping_algorithm] and thus often described as Jarvis binary search.
///
/// Several implementation exist, one can be found (here)[https://gist.github.com/tixxit/252229], but often fail to pick the right summit when the edge is aligned with the outside point. There are several solution to this problem but to reliably output the correct summit without edge cases, the following implementation has been done:
///
/// 1. Find the closest and furthest summits of the convex hull to the point
/// 2. Search for the right or left tangent between those two points (essentially cutting the convex hull in 2)
///
/// The final complexity is O(N log(N)) * 3 => O(N log(N)) with N being the summits of the convex hull.
///
/// This trait also provides a way to find the closest and furthest summit from a point outside of the convex hull.
pub trait JarvisBinarySearch<T: CoordNum> {
    fn find_boundary_point(&self, q: Point<T>, max: bool) -> usize;

    fn closest_summit(&self, q: Point<T>) -> usize;

    fn furthest_summit(&self, q: Point<T>) -> usize;

    fn jarvis_binary_search(&self, q: Point<T>, tangent: Target) -> usize;
}

impl<T: GeoFloat> JarvisBinarySearch<T> for Polygon<T> {
    /// Find the closest summit from `q` to `self`
    fn closest_summit(&self, q: Point<T>) -> usize {
        self.find_boundary_point(q, false)
    }

    /// Find the furthest summit from `q` to `self`
    fn furthest_summit(&self, q: Point<T>) -> usize {
        self.find_boundary_point(q, true)
    }

    /// Find the tangent from point `q` to `self` (`self` is assumed to be counter clockwise).
    ///
    /// You can select which tangent to find using [Target].
    /// This algorithm runs in `O(log(H))` where `H < N` with `N` being the number of points in `self.exterior()`.
    /// It assumes `self` is in CW order.
    fn jarvis_binary_search(&self, q: Point<T>, tangent: Target) -> usize {
        // Reduce the search and facilitate search direction criteria
        let (close, far) = (self.closest_summit(q), self.furthest_summit(q));
        // skip redundant last point
        let poly_len = self.exterior().0.len();

        let (mut start, mut end, mut step) = section_properties(far, close, tangent, poly_len);
        let mut mid = (start + step) % poly_len;

        while step > 0 {
            let (prev_o, next_o) = get_orientations(mid, q, self);

            if correct_tangent(next_o, prev_o, tangent) {
                return mid;
            }

            let direction = get_direction(tangent, prev_o, next_o);

            if direction {
                // go left
                end = (mid + poly_len - 1) % poly_len;
            } else {
                // go right
                start = (mid + 1) % poly_len;
            }

            step /= 2;
            mid = (start + step) % poly_len;

            if mid > end {
                mid = end
            }
        }

        mid
    }

    /// Find a boundary point of `self` from `q`
    ///
    /// `self` is expected to contain the points of a convex polygon sorted in CW order.
    /// `q` is a point outside of `self`.
    /// A boundary point is either the closest or further point of `self` from `q`.
    ///
    /// You can select which point to get using the `max` bool.
    /// Select `true` for maximum and `false` for minimum.
    ///
    /// When the functions ends the last point should be either the unique maximum or unique minimum, or it should be one the them if there are many.
    fn find_boundary_point(&self, q: Point<T>, max: bool) -> usize {
        // skip redundant last point
        let poly_len = self.exterior().0.len() - 1;
        let mut index = poly_len / 2;
        let mut step = index;

        while step > 0 {
            let dist_prev = Euclidean.distance(
                q,
                self.exterior().0[get_previous_point(index, poly_len)].into(),
            );
            let dist_cur = Euclidean.distance(q, self.exterior().0[index].into());
            let dist_next = Euclidean.distance(q, self.exterior().0[(index + 1) % poly_len].into());

            if check_points(dist_prev, dist_cur, dist_next, max) {
                return index;
            }

            if get_local_slope(dist_cur, dist_prev, dist_next, max) {
                // go left
                index = (index + poly_len - step) % poly_len;
            } else {
                // go right
                index = (index + step) % poly_len;
            }

            step /= 2;
        }

        index
    }
}

/// Get the orientation next to the index.
///
/// This function returns the orientation to the previous point and to the next point in that order.
/// It assumes `polygon` is in CW order.
fn get_orientations<T>(
    index: usize,
    q: Point<T>,
    polygon: &Polygon<T>,
) -> (Orientation, Orientation)
where
    T: GeoNum,
{
    let p_i_next_o = T::Ker::orient2d(
        q.into(),
        polygon.exterior().0[index],
        polygon.exterior().0[(index + 1) % (polygon.exterior().0.len() - 1)],
    );
    let p_i_prev_o = T::Ker::orient2d(
        q.into(),
        polygon.exterior().0[index],
        polygon.exterior().0[get_previous_point(index, polygon.exterior().0.len() - 1)],
    );

    (p_i_prev_o, p_i_next_o)
}

/// Returns a triplet of properties in the following order start, end, step.
fn section_properties(
    far_point: usize,
    close_point: usize,
    tangent: Target,
    ring_length: usize,
) -> (usize, usize, usize) {
    let (start, end) = match tangent {
        Target::UpperTangent => (close_point, far_point),
        Target::LowerTangent => (far_point, close_point),
    };

    let section_len = if start >= end {
        ring_length - start + end
    } else {
        end - start
    };

    (start, end, (section_len + 1).div(2))
}

/// Check if the triplets orientation is correct in relation to the target
///
/// The polygon should not contain consecutive edges that are [Orientation::Colinear].
/// Such a polygon is considered invalid.
/// [Target] can be used to select which verification to perform.
///
/// # Panic
///
/// If the current polygon is invalid.
fn correct_tangent(
    next_orientation: Orientation,
    prev_orientation: Orientation,
    tangent_target: Target,
) -> bool {
    // this case should not be possible
    if next_orientation == Orientation::Collinear && next_orientation == prev_orientation {
        panic!("Polygon's ring is not a convex hull")
    }

    match tangent_target {
        Target::UpperTangent => {
            (next_orientation == Orientation::Clockwise
                || next_orientation == Orientation::Collinear)
                && prev_orientation == Orientation::Clockwise
        }
        Target::LowerTangent => {
            next_orientation == Orientation::CounterClockwise
                && (prev_orientation == Orientation::CounterClockwise
                    || prev_orientation == Orientation::Collinear)
        }
    }
}

/// Get the direction encoded as left is 0 and right is 1
fn get_direction(
    target: Target,
    prev_orientation: Orientation,
    next_orientation: Orientation,
) -> bool {
    if target == Target::LowerTangent && next_orientation == Orientation::Collinear {
        return false;
    } else if target == Target::UpperTangent && prev_orientation == Orientation::Collinear {
        return true;
    } else {
        // must check which side is wrong
    }

    match target {
        Target::UpperTangent => next_orientation == Orientation::Clockwise,
        Target::LowerTangent => next_orientation == Orientation::CounterClockwise,
    }
}

fn get_previous_point(index: usize, len: usize) -> usize {
    if index == 0 {
        len - 1
    } else {
        index - 1
    }
}

fn check_points<T>(a: T, b: T, c: T, max: bool) -> bool
where
    T: num_traits::Float,
{
    match max {
        true => a < b && b > c,
        false => a > b && b < c,
    }
}

/// Get the slope between two points of the list.
///
/// If `true` the slope indicates the minimum or the maximum is to the left.
fn get_local_slope<T>(mid: T, left: T, right: T, max: bool) -> bool
where
    T: num_traits::Float,
{
    match max {
        true => {
            // we are on min
            if (mid < left && mid < right) || mid == right {
                left > right
            } else {
                mid > right
            }
        }
        false => {
            // we are on max
            if (mid > left && mid > right) || mid == right {
                left < right
            } else {
                mid < right
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use geo_types::*;

    macro_rules! exterior_polygon {
        () => {
            Polygon::new(
                LineString::from(vec![
                    (7., 8.),  // 0, C = (7, 8)
                    (11., 6.), // 1, D = (11, 6)
                    (12., 2.), // 2, E = (12, 2)
                    (8., -2.), // 3, F = (8, -2)
                    (4., -1.), // 4, G = (4, -1)
                    (3., 2.),  // 5, A = (3,2)
                    (4., 7.),  // 6, B = (4, 7)
                ]),
                vec![],
            )
        };
    }

    macro_rules! interior_polygon {
        () => {
            Polygon::new(
                LineString::from(vec![
                    (6., 5.),  // 0, I = (6, 5)
                    (10., 3.), // 1, J = (10, 3)
                    (9., 0.),  // 2, K = (9, 0)
                    (6., 1.),  // 3, L = (6, 1)
                    (5., 2.),  // 4, H = (5, 2)
                ]),
                vec![],
            )
        };
    }

    #[test]
    fn find_max_in_cyclic_list() {
        let exterior_polygon = exterior_polygon!();
        let interior_polygon = interior_polygon!();
        let expected_point = [2, 4, 4, 0, 1, 1, 2];

        #[allow(clippy::needless_range_loop)]
        for q in 0..exterior_polygon.exterior().0.len() - 1 {
            let res =
                interior_polygon.furthest_summit(geo_types::Point(exterior_polygon.exterior()[q]));

            assert_eq!(res, expected_point[q])
        }
    }

    #[test]
    fn find_min_in_cyclic_list() {
        let exterior_polygon = exterior_polygon!();
        let interior_polygon = interior_polygon!();
        let expected_point = [0, 1, 1, 2, 3, 4, 0];

        #[allow(clippy::needless_range_loop)]
        for q in 0..exterior_polygon.exterior().0.len() - 1 {
            let res =
                interior_polygon.closest_summit(geo_types::Point(exterior_polygon.exterior()[q]));

            assert_eq!(res, expected_point[q])
        }
    }

    #[test]
    fn find_lower_tangents() {
        let exterior_polygon = exterior_polygon!();
        let interior_polygon = interior_polygon!();
        let expected_results = [0, 0, 1, 2, 2, 3, 4];

        assert_eq!(
            expected_results.len(),
            exterior_polygon.exterior().0.len() - 1
        );

        for (q, expected_res) in expected_results.iter().enumerate() {
            let res = interior_polygon.jarvis_binary_search(
                geo_types::Point(exterior_polygon.exterior()[q]),
                Target::LowerTangent,
            );

            assert_eq!(res, *expected_res)
        }
    }

    #[test]
    fn find_upper_tangents() {
        let exterior_polygon = exterior_polygon!();
        let interior_polygon = interior_polygon!();
        let expected_results = [1, 1, 2, 4, 4, 0, 1];

        assert_eq!(
            expected_results.len(),
            exterior_polygon.exterior().0.len() - 1
        );

        for (q, expected_res) in expected_results.iter().enumerate() {
            let res = interior_polygon.jarvis_binary_search(
                geo_types::Point(exterior_polygon.exterior()[q]),
                Target::UpperTangent,
            );

            assert_eq!(res, *expected_res)
        }
    }

    #[test]
    fn find_edge_case() {
        let q = Point::new(16., 25.);
        let next_layer = Polygon::new(
            LineString::from(vec![(24., 11.), (21., 17.), (6., 14.)]),
            vec![],
        );

        assert_eq!(next_layer.furthest_summit(q), 0);

        let q = Point::new(28., 16.);
        let next_layer = Polygon::new(
            LineString::from(vec![(6., 5.), (21., 16.), (3., 12.)]),
            vec![],
        );

        assert_eq!(next_layer.furthest_summit(q), 2);

        let q = Point::new(22., 25.);
        let next_layer = Polygon::new(
            LineString::from(vec![(7., 5.), (14., 6.), (20., 7.), (4., 23.)]),
            vec![],
        );

        assert_eq!(next_layer.closest_summit(q), 3)
    }
}
