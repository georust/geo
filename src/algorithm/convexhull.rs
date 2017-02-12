use std::cmp::Ordering;
use num_traits::Float;
use types::{Point, Polygon, LineString};
use std::collections::BTreeSet;

impl<T> Eq for Point<T> where T: Float {}

impl<T> PartialOrd for Point<T>
    where T: Float
{
    fn partial_cmp(&self, other: &Point<T>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Point<T>
    where T: Float
{
    fn cmp(&self, other: &Point<T>) -> Ordering {
        (self.x(), self.y()).partial_cmp(&(other.x(), other.y())).unwrap()
    }
}

impl<T> Point<T>
    where T: Float
{
    // Draw a horizontal line through this point, connect it with other,
    // and measure the angle between these two lines.
    fn angle(&self, other: &Point<T>) -> T {
        if self == other {
            T::zero()
        } else {
            (other.y() - self.y()).atan2(other.x() - self.x())
        }
    }
}

// Barycentric method for point-in-triangle, see http://blackpawn.com/texts/pointinpoly
impl<T> Polygon<T>
    where T: Float
{
    fn barycentric_contains(&self, p: &Point<T>) -> bool {
        let v0 = self.exterior.0[2] - self.exterior.0[0];
        let v1 = self.exterior.0[1] - self.exterior.0[0];
        let v2 = *p - self.exterior.0[0];
        let dot00 = v0.dot(&v0);
        let dot01 = v0.dot(&v1);
        let dot02 = v0.dot(&v2);
        let dot11 = v1.dot(&v1);
        let dot12 = v1.dot(&v2);
        let inv_denom = (dot00 * dot11 - dot01 * dot01).recip();
        let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
        let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;
        (u > T::zero()) && (v > T::zero()) && (u + v < T::one())
    }
}

// Adapted from http://codereview.stackexchange.com/a/141752/2630
// The algorithm is from Heineman, G.T., Pollice, G., Selkow, S., 2008.
// "Algorithms in a Nutshell". O’Reilly Media, Inc., pp261–8
pub fn convex_hull<T>(points: &BTreeSet<Point<T>>) -> Vec<Point<T>>
    where T: Float
{
    // You must have at least 3 points to construct a hull
    if points.len() < 4 {
        return points.clone().into_iter().collect::<Vec<Point<T>>>()
    }
    // Remove a single point from the set
    let minus_one = |p: &Point<T>| {
        let mut subset = points.clone();
        subset.remove(p);
        subset
    };
    // The set of points that are marked as internal
    let mut p_internal_set = BTreeSet::new();
    // Check all permutations of 4 points:
    // is the fourth point contained in the triangle
    for p_i in points {
        let minus_i = minus_one(p_i);
        for p_j in minus_i {
            let minus_j = minus_one(&p_j);
            for p_k in minus_j {
                let minus_k = minus_one(&p_k);
                for p_m in minus_k {
                    let poly = Polygon::new(LineString(vec![*p_i, p_j, p_k]), vec![]);
                    if poly.barycentric_contains(&p_m) {
                        p_internal_set.insert(p_m);
                    }
                }
            }
        }
    }
    // The set of points that are not internal
    let mut hull: Vec<_> = points.difference(&p_internal_set).cloned().collect();
    // Sort by coordinates, so that the first point is the leftmost
    hull.sort();
    let head = hull[0];

    // Sort by the angle with the first point
    // when that's equal, sort by distance to head
    hull.sort_by(|a, b| {
        let angle_a = head.angle(a);
        let angle_b = head.angle(b);
        angle_a.partial_cmp(&angle_b).unwrap()
    });
    // we need to close the Polygon
    let final_element = *hull.first().unwrap();
    hull.push(final_element);
    hull.into_iter().collect::<Vec<Point<T>>>()
}

pub trait Convexhull<T> {
    /// Returns the convex hull of a Polygon
    ///
    /// ```
    /// use geo::{Point, LineString, Polygon};
    /// use geo::convexhull::Convexhull;
    /// // an L shape
    /// let coords = vec![(0.0, 0.0), (4.0, 0.0), (4.0, 1.0), (1.0, 1.0), (1.0, 4.0), (0.0, 4.0), (0.0, 0.0)];
    /// let ls = LineString(coords.iter().map(|e| Point::new(e.0, e.1)).collect());
    /// let poly = Polygon::new(ls, vec![]);
    ///
    /// // The correct convex hull coordinates
    /// let hull_coords = vec![(0.0, 0.0), (4.0, 0.0), (4.0, 1.0), (1.0, 4.0), (0.0, 4.0), (0.0, 0.0)];
    /// let correct_hull = LineString(hull_coords.iter().map(|e| Point::new(e.0, e.1)).collect());
    ///
    /// let res = poly.convexhull();
    /// assert_eq!(res.exterior, correct_hull);
    /// ```
    fn convexhull(&self) -> Self where T: Float;
}

impl<T> Convexhull<T> for Polygon<T>
    where T: Float
{
    fn convexhull(&self) -> Polygon<T> {
        let bts = self.exterior.0.clone().into_iter().collect::<BTreeSet<Point<T>>>();
        Polygon::new(LineString(convex_hull(&bts)), vec![])
    }
}

#[cfg(test)]
mod test {
    use types::Point;
    use super::*;

    #[macro_export]
    macro_rules! btreeset {
        ($($x: expr),*) => {{
             let mut set = ::std::collections::BTreeSet::new();
             $( set.insert($x); )*
             set
        }}
    }

    #[test]
    fn convex_hull_test() {
        let points = btreeset!(Point::new(0.0, 0.0),
                               Point::new(4.0, 0.0),
                               Point::new(4.0, 1.0),
                               Point::new(1.0, 1.0),
                               Point::new(1.0, 4.0),
                               Point::new(0.0, 4.0));
        // from Shapely, following an "Orient" call
        let correct = vec![Point::new(0.0, 0.0),
                           Point::new(4.0, 0.0),
                           Point::new(4.0, 1.0),
                           Point::new(1.0, 4.0),
                           Point::new(0.0, 4.0),
                           Point::new(0.0, 0.0)];
        let sorted = convex_hull(&points);
        assert_eq!(sorted, correct);
    }
}
