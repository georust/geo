use super::{swap_remove_to_first, trivial_hull};
use crate::algorithm::kernels::*;
use crate::{Coordinate, LineString};

/// The [Graham's scan] algorithm to compute the convex hull
/// of a collection of points. This algorithm is less
/// performant than the quick hull, but allows computing all
/// the points on the convex hull, as opposed to a strict
/// convex hull that does not include collinear points.
///
/// # References
///
/// Graham, R.L. (1972). ["An Efficient Algorithm for
/// Determining the Convex Hull of a Finite Planar
/// Set"](http://www.math.ucsd.edu/~ronspubs/72_10_convex_hull.pdf)
/// (PDF). \
/// Information Processing Letters. 1 (4): 132–133.
/// [doi:10.1016/0020-0190(72)90045-2](https://doi.org/10.1016%2F0020-0190%2872%2990045-2)
///
/// [Graham's scan]: //en.wikipedia.org/wiki/Graham_scan
pub fn graham_hull<T>(mut points: &mut [Coordinate<T>], include_on_hull: bool) -> LineString<T>
where
    T: HasKernel,
{
    if points.len() < 4 {
        // Nothing to build with fewer than four points.
        return trivial_hull(points, include_on_hull);
    }

    // Allocate output vector
    let mut output = Vec::with_capacity(points.len());

    // Find lexicographically least point and add to hull
    use crate::utils::least_index;
    use std::cmp::Ordering;
    let min_idx = least_index(points);
    let head = swap_remove_to_first(&mut points, min_idx);
    output.push(*head);

    // Sort rest of the points by angle it makes with head
    // point. If two points are collinear with head, we sort
    // by distance. We use kernel predicates here.
    let cmp = |q: &Coordinate<T>, r: &Coordinate<T>| match T::Ker::orient2d(*q, *head, *r) {
        Orientation::CounterClockwise => Ordering::Greater,
        Orientation::Clockwise => Ordering::Less,
        Orientation::Collinear => {
            let dist1 = T::Ker::square_euclidean_distance(*head, *q);
            let dist2 = T::Ker::square_euclidean_distance(*head, *r);
            dist1.partial_cmp(&dist2).unwrap()
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
                Orientation::Collinear => {
                    if include_on_hull {
                        break;
                    } else {
                        output.pop();
                    }
                }
            }
        }
        // Corner case: if the lex. least point added before
        // this loop is repeated, then we should not end up
        // adding it here (because output.len() == 1 in the
        // first iteration)
        if include_on_hull || pt != output.last().unwrap() {
            output.push(*pt);
        }
    }

    // Close and output the line string
    let mut output = LineString(output);
    output.close();
    output
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::algorithm::is_convex::IsConvex;
    use std::fmt::Debug;
    fn test_convexity<T: HasKernel + Debug>(initial: &[(T, T)]) {
        let mut v: Vec<_> = initial
            .iter()
            .map(|e| Coordinate::from((e.0, e.1)))
            .collect();
        let hull = graham_hull(&mut v, false);
        assert!(hull.is_strictly_ccw_convex());
        let hull = graham_hull(&mut v, true);
        assert!(hull.is_ccw_convex());
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
        let v = include!("../test_fixtures/poly1.rs");
        test_convexity(&v);
    }

    #[test]
    fn quick_hull_test_complex_2() {
        let coords = include!("../test_fixtures/poly2.rs");
        test_convexity(&coords);
    }
}
