use super::{swap_remove_to_first, trivial_hull};
use crate::algorithm::kernels::*;
use crate::utils::partition_slice;
use crate::{Coordinate, LineString};

// Determines if `p_c` lies on the positive side of the
// segment `p_a` to `p_b`. In other words, whether segment
// `p_a` to `p_c` is a counter-clockwise rotation from the
// segment. We use kernels to ensure this predicate is
// exact.
#[inline]
fn is_ccw<T>(p_a: Coordinate<T>, p_b: Coordinate<T>, p_c: Coordinate<T>) -> bool
where
    T: HasKernel,
{
    let o = T::Ker::orient2d(p_a, p_b, p_c);
    o == Orientation::CounterClockwise
}

// Adapted from https://web.archive.org/web/20180409175413/http://www.ahristov.com/tutorial/geometry-games/convex-hull.html
pub fn quick_hull<T>(mut points: &mut [Coordinate<T>]) -> Result<LineString<T>, ()>
where
    T: HasKernel,
{
    // can't build a hull from fewer than four points
    if points.len() < 4 {
        return Ok(trivial_hull(points, false));
    }
    let mut hull = vec![];

    use crate::utils::least_and_greatest_index;
    let (min, max) = {
        let (min_idx, mut max_idx) = least_and_greatest_index(&points);
        let min = swap_remove_to_first(&mut points, min_idx);

        // Two special cases to consider:
        // (1) max_idx = 0, and got swapped
        if max_idx == 0 {
            max_idx = min_idx;
        }

        // (2) max_idx = min_idx: then any point could be
        // chosen as max. But from case (1), it could now be
        // 0, and we should not decrement it.
        if max_idx > 0 {
            max_idx -= 1;
        }
        let max = swap_remove_to_first(&mut points, max_idx);
        (min, max)
    };

    {
        let (mut points, _) = partition_slice(&mut points, |p| is_ccw(*max, *min, *p));
        hull_set(*max, *min, &mut points, &mut hull);
    }
    hull.push(*max);
    let (mut points, _) = partition_slice(&mut points, |p| is_ccw(*min, *max, *p));
    hull_set(*min, *max, &mut points, &mut hull);
    hull.push(*min);
    // close the polygon
    let mut hull: LineString<_> = hull.into();
    hull.close();
    Ok(hull)
}

// recursively calculate the convex hull of a subset of points
fn hull_set<T>(
    p_a: Coordinate<T>,
    p_b: Coordinate<T>,
    mut set: &mut [Coordinate<T>],
    hull: &mut Vec<Coordinate<T>>,
) -> Result<(), ()>
where
    T: HasKernel,
{
    if set.is_empty() {
        return Ok(());
    }
    if set.len() == 1 {
        hull.push(set[0]);
        return Ok(());
    }

    // Construct orthogonal vector to `p_b` - `p_a` We
    // compute inner product of this with `v` - `p_a` to
    // find the farthest point from the line segment a-b.
    let p_orth = Coordinate {
        y: p_b.x - p_a.x,
        x: p_a.y - p_b.y,
    };

    let (farthest_idx, _) = set
        .iter()
        .map(|pt| {
            let p_diff = Coordinate {
                x: pt.x - p_a.x,
                y: pt.y - p_a.y,
            };
            p_orth.x * p_diff.x + p_orth.y * p_diff.y
        })
        .enumerate()
        .try_fold(
            (0, T::zero()),
            |(farthest_idx, farthest_dist), (curr_idx, curr_dist)| {
                // Ensure `distance` is not NaN
                // TODO: we need a better check
                if curr_dist != curr_dist {
                    return Err(());
                }

                Ok(if curr_dist > farthest_dist {
                    (curr_idx, curr_dist)
                } else {
                    (farthest_idx, farthest_dist)
                })
            },
        )?;

    // move Coordinate at farthest_point from set into hull
    let farthest_point = swap_remove_to_first(&mut set, farthest_idx);
    // points over PB
    {
        let (mut points, _) = partition_slice(set, |p| is_ccw(*farthest_point, p_b, *p));
        hull_set(*farthest_point, p_b, &mut points, hull)?;
    }
    hull.push(*farthest_point);
    // points over AP
    let (mut points, _) = partition_slice(set, |p| is_ccw(p_a, *farthest_point, *p));
    hull_set(p_a, *farthest_point, &mut points, hull)?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::algorithm::is_convex::IsConvex;

    #[test]
    fn quick_hull_test1() {
        let mut v = vec![
            Coordinate { x: 0.0, y: 0.0 },
            Coordinate { x: 4.0, y: 0.0 },
            Coordinate { x: 4.0, y: 1.0 },
            Coordinate { x: 1.0, y: 1.0 },
            Coordinate { x: 1.0, y: 4.0 },
            Coordinate { x: 0.0, y: 4.0 },
            Coordinate { x: 0.0, y: 0.0 },
        ];
        let res = quick_hull(&mut v).unwrap();
        assert!(res.is_strictly_ccw_convex());
    }

    #[test]
    fn quick_hull_test2() {
        let mut v = vec![
            Coordinate { x: 0, y: 10 },
            Coordinate { x: 1, y: 1 },
            Coordinate { x: 10, y: 0 },
            Coordinate { x: 1, y: -1 },
            Coordinate { x: 0, y: -10 },
            Coordinate { x: -1, y: -1 },
            Coordinate { x: -10, y: 0 },
            Coordinate { x: -1, y: 1 },
            Coordinate { x: 0, y: 10 },
        ];
        let correct = vec![
            Coordinate { x: 0, y: -10 },
            Coordinate { x: 10, y: 0 },
            Coordinate { x: 0, y: 10 },
            Coordinate { x: -10, y: 0 },
            Coordinate { x: 0, y: -10 },
        ];
        let res = quick_hull(&mut v).unwrap();
        assert_eq!(res.0, correct);
    }

    #[test]
    // test whether output is ccw
    fn quick_hull_test_ccw() {
        let initial = vec![
            (1.0, 0.0),
            (2.0, 1.0),
            (1.75, 1.1),
            (1.0, 2.0),
            (0.0, 1.0),
            (1.0, 0.0),
        ];
        let mut v: Vec<_> = initial
            .iter()
            .map(|e| Coordinate { x: e.0, y: e.1 })
            .collect();
        let correct = vec![(1.0, 0.0), (2.0, 1.0), (1.0, 2.0), (0.0, 1.0), (1.0, 0.0)];
        let v_correct: Vec<_> = correct
            .iter()
            .map(|e| Coordinate { x: e.0, y: e.1 })
            .collect();
        let res = quick_hull(&mut v).unwrap();
        assert_eq!(res.0, v_correct);
    }

    #[test]
    fn quick_hull_test_ccw_maintain() {
        // initial input begins at min y, is oriented ccw
        let initial = vec![
            (0., 0.),
            (2., 0.),
            (2.5, 1.75),
            (2.3, 1.7),
            (1.75, 2.5),
            (1.3, 2.),
            (0., 2.),
            (0., 0.),
        ];
        let mut v: Vec<_> = initial
            .iter()
            .map(|e| Coordinate { x: e.0, y: e.1 })
            .collect();
        let res = quick_hull(&mut v).unwrap();
        assert!(res.is_strictly_ccw_convex());
    }

    #[test]
    fn quick_hull_test_complex() {
        let coords = include!("../test_fixtures/poly1.rs");
        let mut v: Vec<_> = coords
            .iter()
            .map(|e| Coordinate { x: e.0, y: e.1 })
            .collect();
        let correct = include!("../test_fixtures/poly1_hull.rs");
        let v_correct: Vec<_> = correct
            .iter()
            .map(|e| Coordinate { x: e.0, y: e.1 })
            .collect();
        let res = quick_hull(&mut v).unwrap();
        assert_eq!(res.0, v_correct);
    }

    #[test]
    fn quick_hull_test_complex_2() {
        let coords = include!("../test_fixtures/poly2.rs");
        let mut v: Vec<_> = coords
            .iter()
            .map(|e| Coordinate { x: e.0, y: e.1 })
            .collect();
        let res = quick_hull(&mut v).unwrap();
        assert!(res.is_strictly_ccw_convex());
    }

    #[test]
    fn quick_hull_test_collinear() {
        // Initial input begins at min x, but not min y
        // There are three points with same x.
        // Output should not contain the middle point.
        let initial = vec![
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
        let mut v: Vec<_> = initial
            .iter()
            .map(|e| Coordinate { x: e.0, y: e.1 })
            .collect();
        let res = quick_hull(&mut v).unwrap();
        assert!(res.is_strictly_ccw_convex());
    }
}
