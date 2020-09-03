use super::kernels::*;
use crate::{Coordinate, LineString};

// Utility function: swap idx to head(0th position), remove
// head (modifies the slice), and return head as a reference
fn swap_remove_to_first<'a, T>(slice: &mut &'a mut [T], idx: usize) -> &'a mut T {
    let tmp = std::mem::replace(slice, &mut []);
    tmp.swap(0, idx);
    let (h, t) = tmp.split_first_mut().unwrap();
    *slice = t;
    h
}

/// Graham Scan: https://en.wikipedia.org/wiki/Graham_scan
pub fn graham_hull<T>(mut points: &mut [Coordinate<T>], include_colinear: bool) -> LineString<T>
where
    T: HasKernel,
{
    if points.len() < 4 {
        // Nothing to build with fewer than four points. We
        // remove repeated points if any, and ensure ccw
        // invariant.
        use super::winding_order::Winding;
        let mut ls: LineString<T> = points
            .iter()
            .enumerate()
            .filter_map(|(i, pt)| {
                if i == 0 || pt != &points[i - 1] {
                    Some(*pt)
                } else {
                    None
                }
            })
            .collect();
        ls.close();
        ls.make_ccw_winding();
        return ls;
    }

    // Allocate output vector
    let mut output = Vec::with_capacity(points.len());

    // Find lexicographically least point
    use crate::utils::lexicographically_least_index;
    let min_idx = lexicographically_least_index(points);
    let head = swap_remove_to_first(&mut points, min_idx);
    output.push(*head);

    // Sort rest of the points by angle it makes with head
    // point. If two points are colinear with head, we sort
    // by distance. We use kernel predicates here.
    let cmp = |q: &Coordinate<T>, r: &Coordinate<T>| {
        use std::cmp::Ordering;
        match T::Ker::orient2d(*q, *head, *r) {
            Orientation::CounterClockwise => Ordering::Greater,
            Orientation::Clockwise => Ordering::Less,
            Orientation::Colinear => {
                let dist1 = T::Ker::square_euclidean_distance(*head, *q);
                let dist2 = T::Ker::square_euclidean_distance(*head, *r);
                dist1.partial_cmp(&dist2).unwrap()
            }
        }
    };
    points.sort_unstable_by(cmp);

    for pt in points.iter() {
        while output.len() > 1 {
            let len = output.len();
            match T::Ker::orient2d(output[len - 2], output[len - 1], *pt) {
                Orientation::CounterClockwise => {
                    break;
                }
                Orientation::Clockwise => {
                    output.pop();
                }
                Orientation::Colinear => {
                    if include_colinear {
                        break;
                    } else {
                        output.pop();
                    }
                }
            }
        }
        output.push(*pt);
    }

    // Close and output the line string
    let mut output = LineString(output);
    output.close();
    output
}

#[cfg(test)]
mod test {
    use super::*;
    use geo_types::CoordinateType;

    fn is_ccw_convex<T: CoordinateType + HasKernel>(mut ls: &[Coordinate<T>]) -> bool {
        if ls.len() > 1 && ls[0] == ls[ls.len() - 1] {
            ls = &ls[1..];
        }
        let n = ls.len();
        if n < 3 {
            return true;
        }

        ls.iter().enumerate().all(|(i, coord)| {
            let np = ls[(i + 1) % n];
            let nnp = ls[(i + 2) % n];
            T::Ker::orient2d(*coord, np, nnp) == Orientation::CounterClockwise
        })
    }

    fn test_convexity<T: CoordinateType + HasKernel>(initial: &[(T, T)]) {
        let mut v: Vec<_> = initial
            .iter()
            .map(|e| Coordinate::from((e.0, e.1)))
            .collect();
        let hull = graham_hull(&mut v, false);
        assert!(is_ccw_convex(&hull.0));
    }

    #[test]
    fn test_graham_hull_ccw() {
        let initial = vec![
            (1.0, 0.0),
            (2.0, 1.0),
            (1.75, 1.1),
            (1.0, 2.0),
            (0.0, 1.0),
            (1.0, 0.0),
        ];
        test_convexity(&initial);
    }

    #[test]
    fn graham_hull_test1() {
        let v: Vec<_> = vec![(0, 0), (4, 0), (4, 1), (1, 1), (1, 4), (0, 4), (0, 0)];
        test_convexity(&v);
    }

    #[test]
    fn graham_hull_test2() {
        let v = vec![
            (0, 10),
            (1, 1),
            (10, 0),
            (1, -1),
            (0, -10),
            (-1, -1),
            (-10, 0),
            (-1, 1),
            (0, 10),
        ];
        test_convexity(&v);
    }

    #[test]
    fn graham_test_complex() {
        let v = include!("test_fixtures/poly1.rs");
        test_convexity(&v);
    }

    #[test]
    fn quick_hull_test_complex_2() {
        let coords = include!("test_fixtures/poly2.rs");
        test_convexity(&coords);
    }
}
