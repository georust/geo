use super::{swap_with_first_and_remove, trivial_hull};
use crate::kernels::{Kernel, Orientation};
use crate::utils::{lex_cmp, partition_slice};
use crate::{Coord, CoordNum, GeoNum, LineString, coord};
use std::cmp::Ordering;

// Generic point-like element the kernel can operate on. Coord<T> carries no
// extra information; (usize, Coord<T>) tags each input with its position so
// the indices fall out of the algorithm.
trait HasCoord<T: CoordNum> {
    fn coord(&self) -> Coord<T>;
}

impl<T: CoordNum> HasCoord<T> for Coord<T> {
    fn coord(&self) -> Coord<T> {
        *self
    }
}

impl<T: CoordNum> HasCoord<T> for (usize, Coord<T>) {
    fn coord(&self) -> Coord<T> {
        self.1
    }
}

// Determines if `p_c` lies on the positive side of the
// segment `p_a` to `p_b`. In other words, whether segment
// `p_a` to `p_c` is a counter-clockwise rotation from the
// segment. We use kernels to ensure this predicate is
// exact.
#[inline]
fn is_ccw<T>(p_a: Coord<T>, p_b: Coord<T>, p_c: Coord<T>) -> bool
where
    T: GeoNum,
{
    let o = T::Ker::orient2d(p_a, p_b, p_c);
    o == Orientation::CounterClockwise
}

// Adapted from https://web.archive.org/web/20180409175413/http://www.ahristov.com/tutorial/geometry-games/convex-hull.html
pub fn quick_hull<T>(points: &mut [Coord<T>]) -> LineString<T>
where
    T: GeoNum,
{
    if points.len() < 4 {
        return trivial_hull(points, false);
    }
    let hull = quick_hull_kernel::<T, Coord<T>>(points);
    let mut ls: LineString<T> = hull.into();
    ls.close();
    ls
}

/// Index-tracking analogue of [`quick_hull`]. Returns input indices of the
/// hull-perimeter coords, in CCW order, closed (first index repeated at the end).
pub fn quick_hull_indices<T>(points: &[Coord<T>]) -> Vec<usize>
where
    T: GeoNum,
{
    if points.len() < 4 {
        // trivial_hull's coord output is a strict subset of the input;
        // recover indices by lookup. Linear scan is bounded by 4 hull vertices.
        let mut working = points.to_vec();
        let hull = trivial_hull(&mut working, false);
        return hull
            .0
            .iter()
            .map(|hc| {
                points
                    .iter()
                    .position(|c| c == hc)
                    .expect("hull vertex must come from the input coords")
            })
            .collect();
    }
    let mut tagged: Vec<(usize, Coord<T>)> = points.iter().copied().enumerate().collect();
    let hull = quick_hull_kernel::<T, (usize, Coord<T>)>(&mut tagged);
    let mut indices: Vec<usize> = hull.into_iter().map(|(i, _)| i).collect();
    // close the polygon: repeat the first index at the end if not already there
    if !indices.is_empty() && indices.first() != indices.last() {
        indices.push(indices[0]);
    }
    indices
}

fn quick_hull_kernel<T, P>(mut points: &mut [P]) -> Vec<P>
where
    T: GeoNum,
    P: HasCoord<T> + Copy,
{
    let mut hull = vec![];

    let (min, max) = {
        let (min_idx, mut max_idx) = least_and_greatest_index(points);
        let min = swap_with_first_and_remove(&mut points, min_idx);

        // Two special cases to consider:
        // (1) max_idx = 0, and got swapped
        if max_idx == 0 {
            max_idx = min_idx;
        }

        // (2) max_idx = min_idx: then any point could be
        // chosen as max. But from case (1), it could now be
        // 0, and we should not decrement it.
        max_idx = max_idx.saturating_sub(1);

        let max = swap_with_first_and_remove(&mut points, max_idx);
        (*min, *max)
    };

    {
        let (points, _) = partition_slice(points, |p| is_ccw(max.coord(), min.coord(), p.coord()));
        hull_set(max, min, points, &mut hull);
    }
    hull.push(max);
    let (points, _) = partition_slice(points, |p| is_ccw(min.coord(), max.coord(), p.coord()));
    hull_set(min, max, points, &mut hull);
    hull.push(min);
    hull
}

/// Recursively calculate the convex hull of a subset of points
fn hull_set<T, P>(p_a: P, p_b: P, mut set: &mut [P], hull: &mut Vec<P>)
where
    T: GeoNum,
    P: HasCoord<T> + Copy,
{
    if set.is_empty() {
        return;
    }
    if set.len() == 1 {
        hull.push(set[0]);
        return;
    }

    // Construct orthogonal vector to `p_b` - `p_a` We
    // compute inner product of this with `v` - `p_a` to
    // find the farthest point from the line segment a-b.
    let p_orth = coord! {
        x: p_a.coord().y - p_b.coord().y,
        y: p_b.coord().x - p_a.coord().x,
    };

    let furthest_idx = set
        .iter()
        .map(|pt| {
            let c = pt.coord();
            let p_diff = coord! {
                x: c.x - p_a.coord().x,
                y: c.y - p_a.coord().y,
            };
            p_orth.x * p_diff.x + p_orth.y * p_diff.y
        })
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .unwrap()
        .0;

    // move element at furthest_idx from set into hull
    let furthest_point = swap_with_first_and_remove(&mut set, furthest_idx);
    // points over PB
    {
        let (points, _) = partition_slice(set, |p| {
            is_ccw(furthest_point.coord(), p_b.coord(), p.coord())
        });
        hull_set(*furthest_point, p_b, points, hull);
    }
    hull.push(*furthest_point);
    // points over AP
    let (points, _) = partition_slice(set, |p| {
        is_ccw(p_a.coord(), furthest_point.coord(), p.coord())
    });
    hull_set(p_a, *furthest_point, points, hull);
}

fn least_and_greatest_index<T, P>(pts: &[P]) -> (usize, usize)
where
    T: GeoNum,
    P: HasCoord<T>,
{
    assert_ne!(pts.len(), 0);
    let (mut min_i, mut max_i) = (0, 0);
    for i in 1..pts.len() {
        let c = pts[i].coord();
        if lex_cmp(&c, &pts[min_i].coord()) == Ordering::Less {
            min_i = i;
        }
        if lex_cmp(&c, &pts[max_i].coord()) == Ordering::Greater {
            max_i = i;
        }
    }
    (min_i, max_i)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::IsConvex;

    #[test]
    fn quick_hull_test1() {
        let mut v = vec![
            coord! { x: 0.0, y: 0.0 },
            coord! { x: 4.0, y: 0.0 },
            coord! { x: 4.0, y: 1.0 },
            coord! { x: 1.0, y: 1.0 },
            coord! { x: 1.0, y: 4.0 },
            coord! { x: 0.0, y: 4.0 },
            coord! { x: 0.0, y: 0.0 },
        ];
        let res = quick_hull(&mut v);
        assert!(res.is_strictly_ccw_convex());
    }

    #[test]
    fn quick_hull_test2() {
        let mut v = vec![
            coord! { x: 0, y: 10 },
            coord! { x: 1, y: 1 },
            coord! { x: 10, y: 0 },
            coord! { x: 1, y: -1 },
            coord! { x: 0, y: -10 },
            coord! { x: -1, y: -1 },
            coord! { x: -10, y: 0 },
            coord! { x: -1, y: 1 },
            coord! { x: 0, y: 10 },
        ];
        let correct = vec![
            coord! { x: 0, y: -10 },
            coord! { x: 10, y: 0 },
            coord! { x: 0, y: 10 },
            coord! { x: -10, y: 0 },
            coord! { x: 0, y: -10 },
        ];
        let res = quick_hull(&mut v);
        assert_eq!(res.0, correct);
    }

    #[test]
    // test whether output is ccw
    fn quick_hull_test_ccw() {
        let initial = [
            (1.0, 0.0),
            (2.0, 1.0),
            (1.75, 1.1),
            (1.0, 2.0),
            (0.0, 1.0),
            (1.0, 0.0),
        ];
        let mut v: Vec<_> = initial.iter().map(|e| coord! { x: e.0, y: e.1 }).collect();
        let correct = [(1.0, 0.0), (2.0, 1.0), (1.0, 2.0), (0.0, 1.0), (1.0, 0.0)];
        let v_correct: Vec<_> = correct.iter().map(|e| coord! { x: e.0, y: e.1 }).collect();
        let res = quick_hull(&mut v);
        assert_eq!(res.0, v_correct);
    }

    #[test]
    fn quick_hull_test_ccw_maintain() {
        // initial input begins at min y, is oriented ccw
        let initial = [
            (0., 0.),
            (2., 0.),
            (2.5, 1.75),
            (2.3, 1.7),
            (1.75, 2.5),
            (1.3, 2.),
            (0., 2.),
            (0., 0.),
        ];
        let mut v: Vec<_> = initial.iter().map(|e| coord! { x: e.0, y: e.1 }).collect();
        let res = quick_hull(&mut v);
        assert!(res.is_strictly_ccw_convex());
    }

    #[test]
    fn quick_hull_test_complex() {
        let mut coords = geo_test_fixtures::poly1::<f64>().0;
        let correct = geo_test_fixtures::poly1_hull::<f64>().0;
        let res = quick_hull(&mut coords);
        assert_eq!(res.0, correct);
    }

    #[test]
    fn quick_hull_test_complex_2() {
        let mut coords = geo_test_fixtures::poly2::<f64>().0;
        let correct = geo_test_fixtures::poly2_hull::<f64>().0;
        let res = quick_hull(&mut coords);
        assert_eq!(res.0, correct);
    }

    #[test]
    fn quick_hull_test_collinear() {
        // Initial input begins at min x, but not min y
        // There are three points with same x.
        // Output should not contain the middle point.
        let initial = [
            (-1., 0.),
            (-1., -1.),
            (-1., 1.),
            (0., 0.),
            (0., -1.),
            (0., 1.),
            (1., 0.),
            (1., -1.),
            (1., 1.),
        ];
        let mut v: Vec<_> = initial.iter().map(|e| coord! { x: e.0, y: e.1 }).collect();
        let res = quick_hull(&mut v);
        assert!(res.is_strictly_ccw_convex());
    }
}
