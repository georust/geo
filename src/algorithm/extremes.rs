use num_traits::Float;
use types::{Point, LineString, Polygon};
use algorithm::convexhull::ConvexHull;
use types::Extremes;

// Useful direction vectors:
// 1., 0. = largest x
// 0., 1. = largest y
// 0., -1. = smallest y
// -1, 0. = smallest x

// various tests for vector orientation relative to a direction vector u
fn up<T>(u: &Point<T>, v: &Point<T>) -> bool
    where T: Float
{
    u.dot(v) > T::zero()
}

fn direction_sign<T>(u: &Point<T>, vi: &Point<T>, vj: &Point<T>) -> T
    where T: Float
{
    u.dot(&(*vi - *vj))
}

// true if Vi is above Vj
fn above<T>(u: &Point<T>, vi: &Point<T>, vj: &Point<T>) -> bool
    where T: Float
{
    direction_sign(u, vi, vj) > T::zero()
}

// true if Vi is below Vj
fn below<T>(u: &Point<T>, vi: &Point<T>, vj: &Point<T>) -> bool
    where T: Float
{
    direction_sign(u, vi, vj) < T::zero()
}

// wrapper for extreme-finding function
fn find_extremes<T, F>(func: F, polygon: &Polygon<T>, convex: bool, oriented: bool) -> Extremes<T>
    where T: Float,
          F: Fn(&Point<T>, &Polygon<T>) -> Result<Point<T>, ()>
{
    // TODO: we can't use this until Orient lands
    // let mut processed = false;
    // if !convex {
    //     let mut poly = polygon.convex_hull();
    //     let processed = true;
    // }
    // if !oriented and processed {
    //    poly = poly.orient()
    // } else if !oriented and !processed {
    //     poly = polygon.orient();
    // } else {
    //    poly = polygon;
    // }
    let directions = vec![Point::new(T::zero(), -T::one()),
                          Point::new(T::one(), T::zero()),
                          Point::new(T::zero(), T::one()),
                          Point::new(-T::one(), T::zero())];
    directions
        .iter()
        .map(|p| func(&p, &polygon).unwrap())
        .collect::<Vec<Point<T>>>()
        .into()
}

// find a convex, counter-clockwise oriented polygon's maximum vertex in a specified direction
// u: a direction vector. We're using a point to represent this, which is a hack tbh
// this is O(n), because polymax() can't yet calculate minimum x
fn polymax_naive<T>(u: &Point<T>, poly: &Polygon<T>) -> Result<Point<T>, ()>
    where T: Float
{
    let vertices = &poly.exterior.0;
    let mut max: usize = 0;
    for (i, _) in vertices.iter().enumerate() {
        // if vertices[i] is above prior vertices[max]
        if above(u, &vertices[i], &vertices[max]) {
            max = i;
        }
    }
    return Ok(vertices[max]);
}

// ported from a c++ implementation at
// http://geomalgorithms.com/a14-_extreme_pts.html#polyMax_2D()
// original copyright notice:
// Copyright 2002 softSurfer, 2012-2013 Dan Sunday
// This code may be freely used, distributed and modified for any
// purpose providing that this copyright notice is included with it.
// SoftSurfer makes no warranty for this code, and cannot be held
// liable for any real or imagined damage resulting from its use.
// Users of this code must verify correctness for their application.

// Original implementation:
// Joseph O'Rourke, Computational Geometry in C (2nd Edition),
// Sect 7.9 "Extreme Point of Convex Polygon" (1998)

// find a convex, counter-clockwise oriented polygon's maximum vertex in a specified direction
// u: a direction vector. We're using a point to represent this, which is a hack tbh
// this should run in O(log n) time.
// This implementation can't calculate minimum x
// see Section 5.1 at http://codeforces.com/blog/entry/48868 for a discussion
fn polymax<T>(u: &Point<T>, poly: &Polygon<T>) -> Result<Point<T>, ()>
    where T: Float
{
    let vertices = &poly.exterior.0;
    let n = vertices.len();
    // these are used to divide the vertices slice
    // start chain = [0, n] with vertices[n] = vertices[0]
    let mut start: usize = 0;
    let mut end: usize = n;
    let mut mid: usize;

    // edge vectors at vertices[a] and vertices[c]
    let mut vec_c: Point<T>;
    let vec_a = vertices[1] - vertices[0];
    // test for "up" direction of vec_a
    let mut up_a = up(u, &vec_a);

    // test if vertices[0] is a local maximum
    if !up_a && !above(u, &vertices[n - 1], &vertices[0]) {
        return Ok(vertices[0]);
    }
    loop {
        mid = (start + end) / 2;
        vec_c = vertices[mid + 1] - vertices[mid];
        let up_c = up(u, &vec_c);
        if !up_c && !above(u, &vertices[mid - 1], &vertices[mid]) {
            // vertices[mid] is a local maximum, thus it is a maximum
            return Ok(vertices[mid]);
        }
        // no max yet, so continue with the binary search
        // pick one of the two subchains [start, mid]  or [mid, end]
        if up_a {
            // vec_a points up
            if !up_c {
                // vec_c points down
                end = mid; // select [start, mid]
            } else {
                // vec_c points up
                if above(u, &vertices[start], &vertices[mid]) {
                    // vertices[start] is above vertices[mid]
                    end = mid; // select [start, mid]
                } else {
                    // vertices[start] is below vertices[mid]
                    start = mid; // select [mid, end]
                    up_a = up_c;
                }
            }
        } else {
            // vec_a points down
            if up_c {
                // vec_c points up
                start = mid; // select [mid, end]
                up_a = up_c;
            } else {
                // vec_c points down
                if below(u, &vertices[start], &vertices[mid]) {
                    // vertices[start] is below vertices[mid]
                    end = mid; // select [start, mid]
                } else {
                    // vertices[start] is above vertices[mid]
                    start = mid; // select [mid, end]
                    up_a = up_c;
                }
            }
        }
        // something went really badly wrong
        if end <= start + 1 {
            return Err(());
        }
    }
}

pub trait ExtremePoints<T: Float> {
    /// Find the extreme `x` and `y` points of a Polygon
    ///
    /// The polygon must be convex and properly oriented; if you're unsure whether
    /// the polygon has these properties:
    ///
    /// - If you aren't sure whether the polygon is convex, choose `convex=false, oriented=false`
    /// - If the polygon is convex but oriented clockwise, choose `convex=true, oriented=false`
    ///
    /// Convex-hull processing is `O(n log(n))` on average
    /// and is thus an upper bound on point-finding, which is otherwise `O(n)` for a `Polygon`.
    /// For a `MultiPolygon`, its convex hull must always be calculated first.
    ///
    /// ```
    /// use geo::{Point, LineString, Polygon};
    /// use geo::extremes::ExtremePoints;
    /// // a diamond shape
    /// let points_raw = vec![(1.0, 0.0), (2.0, 1.0), (1.0, 2.0), (0.0, 1.0), (1.0, 0.0)];
    /// let points = points_raw.iter().map(|e| Point::new(e.0, e.1)).collect::<Vec<_>>();
    /// let poly = Polygon::new(LineString(points), vec![]);
    /// // Polygon is both convex and oriented counter-clockwise
    /// let extremes = poly.extreme_points(true, true);
    /// assert_eq!(extremes.ymin, Point::new(1.0, 0.0));
    /// assert_eq!(extremes.xmax, Point::new(2.0, 1.0));
    /// assert_eq!(extremes.ymax, Point::new(1.0, 2.0));
    /// assert_eq!(extremes.xmin, Point::new(0.0, 1.0));
    /// ```
    fn extreme_points(&self, convex: bool, oriented: bool) -> Extremes<T>;
}

impl<T> ExtremePoints<T> for Polygon<T>
    where T: Float
{
    fn extreme_points(&self, convex: bool, oriented: bool) -> Extremes<T> {
        find_extremes(polymax_naive, self, convex, oriented)
    }
}

#[cfg(test)]
mod test {
    use types::Point;
    use super::*;
    #[test]
    fn test_polygon_extreme_x() {
        // a diamond shape
        let points_raw = vec![(1.0, 0.0), (2.0, 1.0), (1.0, 2.0), (0.0, 1.0), (1.0, 0.0)];
        let points = points_raw
            .iter()
            .map(|e| Point::new(e.0, e.1))
            .collect::<Vec<_>>();
        let poly1 = Polygon::new(LineString(points), vec![]);
        let min_x = polymax_naive(&Point::new(-1., 0.), &poly1).unwrap();
        let correct = Point::new(0., 1.);
        assert_eq!(min_x, correct);
    }
    #[test]
    #[should_panic]
    // this test should panic, because the algorithm can't find minimum x
    fn test_polygon_extreme_x_fast() {
        // a diamond shape
        let points_raw = vec![(1.0, 0.0), (2.0, 1.0), (1.0, 2.0), (0.0, 1.0), (1.0, 0.0)];
        let points = points_raw
            .iter()
            .map(|e| Point::new(e.0, e.1))
            .collect::<Vec<_>>();
        let poly1 = Polygon::new(LineString(points), vec![]);
        let min_x = polymax(&Point::new(-1., 0.), &poly1).unwrap();
        let correct = Point::new(0., 1.);
        assert_eq!(min_x, correct);
    }
    #[test]
    fn test_polygon_extreme_wrapper() {
        // a diamond shape with a bump on the top-right edge
        let points_raw =
            vec![(1.0, 0.0), (2.0, 1.0), (1.75, 1.75), (1.0, 2.0), (0.0, 1.0), (1.0, 0.0)];
        let points = points_raw
            .iter()
            .map(|e| Point::new(e.0, e.1))
            .collect::<Vec<_>>();
        let poly1 = Polygon::new(LineString(points), vec![]);
        let extremes = find_extremes(polymax_naive, &poly1, true, true);
        let correct = Extremes {
            ymin: Point::new(1.0, 0.0),
            xmax: Point::new(2.0, 1.0),
            ymax: Point::new(1.0, 2.0),
            xmin: Point::new(0.0, 1.0),
        };
        assert_eq!(extremes, correct);
    }
}
