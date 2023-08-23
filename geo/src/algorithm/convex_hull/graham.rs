use super::trivial_hull;
use crate::kernels::*;
use crate::utils::lex_cmp;
use crate::{Coord, GeoNum, LineString};

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
/// Andrew, A. M. (1979). "Another efficient algorithm for convex hulls in two dimensions".
/// Information Processing Letters. 9 (5): 216–219.
/// [doi:10.1016/0020-0190(79)90072-3](https://doi.org/10.1016/0020-0190(79)90072-3).
///
/// [Graham's scan]: //en.wikipedia.org/wiki/Graham_scan
pub fn graham_hull<T>(points: &mut [Coord<T>], include_on_hull: bool) -> LineString<T>
where
    T: GeoNum,
{
    // Nothing to build with fewer than four points.
    if points.len() < 4 {
        return trivial_hull(points, include_on_hull);
    }

    // Sort the points lexicographically
    points.sort_unstable_by(lex_cmp);

    // Allocate output vector
    let mut output = Vec::with_capacity(points.len());

    // Compute lower hull part
    output.extend_from_slice(&points[..2]);

    for p in &points[2..] {
        output.push(*p);
        loop {
            let len = output.len();

            if len < 3 {
                break;
            }

            match T::Ker::orient2d(output[len - 3], output[len - 2], output[len - 1]) {
                Orientation::CounterClockwise => break,
                Orientation::Clockwise => {
                    output.remove(len - 2);
                }
                Orientation::Collinear => {
                    if include_on_hull {
                        break;
                    } else {
                        output.remove(len - 2);
                    }
                }
            }
        }
    }

    // Compute upper hull part
    let offset = output.len() - 1;

    let second_last = points.len() - 2;
    output.push(points[second_last]);

    for p in points[..second_last].iter().rev() {
        output.push(*p);
        loop {
            let len = output.len();

            if len - offset < 3 {
                break;
            }

            match T::Ker::orient2d(output[len - 3], output[len - 2], output[len - 1]) {
                Orientation::CounterClockwise => break,
                Orientation::Clockwise => {
                    output.remove(len - 2);
                }
                Orientation::Collinear => {
                    if include_on_hull {
                        break;
                    } else {
                        output.remove(len - 2);
                    }
                }
            }
        }
    }

    // Output the closed ccw line string
    LineString::new(output)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::IsConvex;
    fn test_convexity<T: GeoNum>(mut initial: Vec<Coord<T>>) {
        let hull = graham_hull(&mut initial, false);
        assert!(hull.is_strictly_ccw_convex());
        let hull = graham_hull(&mut initial, true);
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
        let initial = initial.iter().map(|e| Coord::from((e.0, e.1))).collect();
        test_convexity(initial);
    }

    #[test]
    fn graham_hull_test1() {
        let v: Vec<_> = vec![(0, 0), (4, 0), (4, 1), (1, 1), (1, 4), (0, 4), (0, 0)];
        let initial = v.iter().map(|e| Coord::from((e.0, e.1))).collect();
        test_convexity(initial);
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
        let initial = v.iter().map(|e| Coord::from((e.0, e.1))).collect();
        test_convexity(initial);
    }

    #[test]
    fn graham_test_complex() {
        test_convexity(geo_test_fixtures::poly1::<f64>().0);
    }

    #[test]
    fn quick_hull_test_complex_2() {
        test_convexity(geo_test_fixtures::poly2::<f64>().0);
    }
}
