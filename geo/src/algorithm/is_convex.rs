use std::cmp::Ordering;
use std::collections::VecDeque;

use crate::kernels::*;
use crate::{Coord, GeoNum, LineString, Orientation};

/// Predicates to test the convexity of a [ `LineString` ].
/// A closed `LineString` is said to be _convex_ if it
/// encloses a [convex set]. It is said to be _strictly
/// convex_ if in addition, no three consecutive vertices
/// are collinear. It is _collinear_ if all the vertices lie
/// on the same line.
///
/// # Remarks
///
/// - Collinearity does not require that the `LineString`
///   be closed, but the rest of the predicates do.
///
/// - This definition is closely related to the notion
///   of [convexity of polygons][convex set]. In particular, a
///   [`Polygon`](crate::Polygon) is convex, if and only if its `exterior` is
///   convex, and `interiors` is empty.
///
/// - The [`ConvexHull`] algorithm always returns a strictly
///   convex `LineString` unless the input is empty or
///   collinear. The [`graham_hull`] algorithm provides an
///   option to include collinear points, producing a
///   (possibly non-strict) convex `LineString`.
///
/// # Edge Cases
///
/// - the convexity, and collinearity of an empty
///   `LineString` is _unspecified_ and must not be relied
///   upon.
///
/// - A closed `LineString` with at most three coordinates
///   (including the possibly repeated first coordinate) is
///   both convex and collinear. However, the strict convexity
///   is _unspecified_ and must not be relied upon.
///
/// [convex combination]: //en.wikipedia.org/wiki/Convex_combination
/// [convex set]: //en.wikipedia.org/wiki/Convex_set
/// [`ConvexHull`]: crate::ConvexHull
/// [`graham_hull`]: crate::convex_hull::graham_hull
pub trait IsConvex {
    /// Test and get the orientation if the shape is convex.
    /// Tests for strict convexity if `allow_collinear`, and
    /// only accepts a specific orientation if provided.
    ///
    /// The return value is `None` if either:
    ///
    /// 1. the shape is not convex
    ///
    /// 1. the shape is not strictly convex, and
    ///    `allow_collinear` is false
    ///
    /// 1. an orientation is specified, and some three
    ///    consecutive vertices where neither collinear, nor
    ///    in the specified orientation.
    ///
    /// In all other cases, the return value is the
    /// orientation of the shape, or `Orientation::Collinear`
    /// if all the vertices are on the same line.
    ///
    /// **Note.** This predicate is not equivalent to
    /// `is_collinear` as this requires that the input is
    /// closed.
    fn convex_orientation(
        &self,
        allow_collinear: bool,
        specific_orientation: Option<Orientation>,
    ) -> Option<Orientation>;

    /// Test if the shape is convex.
    fn is_convex(&self) -> bool {
        self.convex_orientation(true, None).is_some()
    }

    /// Test if the shape is convex, and oriented
    /// counter-clockwise.
    fn is_ccw_convex(&self) -> bool {
        self.convex_orientation(true, Some(Orientation::CounterClockwise))
            .is_some()
    }

    /// Test if the shape is convex, and oriented clockwise.
    fn is_cw_convex(&self) -> bool {
        self.convex_orientation(true, Some(Orientation::Clockwise))
            .is_some()
    }

    /// Test if the shape is strictly convex.
    fn is_strictly_convex(&self) -> bool {
        self.convex_orientation(false, None).is_some()
    }

    /// Test if the shape is strictly convex, and oriented
    /// counter-clockwise.
    fn is_strictly_ccw_convex(&self) -> bool {
        self.convex_orientation(false, Some(Orientation::CounterClockwise))
            == Some(Orientation::CounterClockwise)
    }

    /// Test if the shape is strictly convex, and oriented
    /// clockwise.
    fn is_strictly_cw_convex(&self) -> bool {
        self.convex_orientation(false, Some(Orientation::Clockwise)) == Some(Orientation::Clockwise)
    }

    /// Test if the shape lies on a line.
    fn is_collinear(&self) -> bool;
}

impl<T: GeoNum> IsConvex for LineString<T> {
    fn convex_orientation(
        &self,
        allow_collinear: bool,
        specific_orientation: Option<Orientation>,
    ) -> Option<Orientation> {
        if !self.is_closed() || self.0.is_empty() {
            None
        } else {
            // Use sign flip algorithm which detects both non-convexity and self-intersections
            if is_convex_sign_flips(&self.0) {
                // If sign flips indicate convex, determine orientation
                is_convex_shaped(&self.0[1..], allow_collinear, specific_orientation)
            } else {
                None
            }
        }
    }

    fn is_collinear(&self) -> bool {
        self.0.is_empty()
            || is_convex_shaped(&self.0[1..], true, Some(Orientation::Collinear)).is_some()
    }

    /// Test if the shape is convex.
    fn is_convex(&self) -> bool {
        is_convex_sign_flips(&self.0)
    }
}
/// Check if a LineString is convex using the sign flip algorithm.
///
/// This implementation is based on the algorithm described at:
/// https://math.stackexchange.com/questions/1743995/determine-whether-a-polygon-is-convex-based-on-its-vertices/1745427#1745427
///
/// # Algorithm Overview
///
/// The sign flip algorithm works by:
/// 1. Computing edge vectors between consecutive vertices
/// 2. Tracking sign changes in the x and y components of these vectors
/// 3. Checking orientation consistency using cross product
/// 4. A convex polygon has exactly 2 sign flips along each axis
///
/// This approach detects both:
/// - Orientation changes (concave vertices)
/// - Self-intersections (star polygons)
///
/// # Time Complexity
///
/// - **Time**: O(n) where n is the number of vertices
/// - **Early exit**: Returns false immediately when >2 flips are detected or orientation changes
///
///
/// # Edge Cases
///
/// - Empty polygons (n = 0): Considered convex
/// - Single points (n = 1): Considered convex
/// - Line segments (n = 3): Considered convex
/// - Triangles (n = 4): Special handling with relaxed flip constraints
/// - Regular polygons (n ≥ 4): Standard 2-flip rule applies
/// - Zero-length edges: Ignored in sign flip counting
/// - Collinear vertices: Detected via cross product consistency
///
/// # Examples
///
/// ```text
/// Convex square: [(0,0), (1,0), (1,1), (0,1)] → x_flips=2, y_flips=2 → true
/// Star polygon: [(0,0), (10,0), (7,-1), (4,5), (6,5), (3,-1)] → >2 flips → false
/// Triangle: [(0,0), (1,0), (0,1)] → orientation consistent → true
/// ```
fn is_convex_sign_flips<T: GeoNum>(coords: &[Coord<T>]) -> bool {
    // Convexity only makes sense for closed shapes
    if coords.first() != coords.last() {
        return false;
    }
    let n = coords.len();

    // Edge case 1: Handle degenerate cases first
    // // These cases are considered convex by definition
    if n <= 1 {
        return true; // Empty or single point
    }
    if n == 2 {
        return true; // Single point (with closing duplicate)
    }
    // For actual polygons with 3+ vertices (4+ coords including closing), use sign flip algorithm
    let mut w_orientation: Option<Orientation> = None; // Track orientation consistency via robust predicate

    // Sign flip counters for x-axis
    let mut x_sign = 0i8; // Current sign: -1 (negative), 0 (zero), 1 (positive)
    let mut x_first_sign = 0i8; // First non-zero sign encountered
    let mut x_flips = 0; // Number of sign changes

    // Sign flip counters for y-axis
    let mut y_sign = 0i8;
    let mut y_first_sign = 0i8;
    let mut y_flips = 0;

    // The last vertex is the repeated closing vertex, so we use n - 1
    let vertices = &coords[..n - 1];
    let vertex_count = vertices.len();

    // Initialize sliding window: start with the last two actual vertices
    // This allows us to compute edge vectors for the wraparound case
    let mut prev = vertices[vertex_count - 1];
    let mut curr = vertices[0];
    // Main algorithm loop: process each vertex and its incident edges
    // Time complexity: O(n): single pass through all vertices
    // Use iterator with cycle() to handle wraparound
    for next in vertices.iter().cycle().skip(1).take(vertex_count) {
        // Compute edge vectors for sign flip tracking
        let ax = next.x - curr.x; // "after" edge: curr → next
        let ay = next.y - curr.y;

        // Track sign flips in x direction
        // The sign flip algorithm principle: a convex polygon traversed in order
        // will have edge vectors that change sign exactly twice per axis

        match ax.partial_cmp(&T::zero()) {
            Some(Ordering::Greater) => {
                match x_sign.cmp(&0) {
                    Ordering::Equal => x_first_sign = 1,
                    Ordering::Less => x_flips += 1,
                    _ => {}
                }
                x_sign = 1;
            }
            Some(Ordering::Less) => {
                match x_sign.cmp(&0) {
                    Ordering::Equal => x_first_sign = -1,
                    Ordering::Greater => x_flips += 1,
                    _ => {}
                }
                x_sign = -1;
            }
            _ => {
                // ax is 0 or NaN, which doesn't change sign state
            }
        }

        // Early exit optimization: if we've seen more than 2 flips, it's not convex
        if x_flips > 2 {
            return false;
        }

        // Track sign flips in y direction (same logic as x)
        match ay.partial_cmp(&T::zero()) {
            Some(Ordering::Greater) => {
                match y_sign.cmp(&0) {
                    Ordering::Equal => y_first_sign = 1,
                    Ordering::Less => y_flips += 1,
                    _ => {}
                }
                y_sign = 1;
            }
            Some(Ordering::Less) => {
                match y_sign.cmp(&0) {
                    Ordering::Equal => y_first_sign = -1,
                    Ordering::Greater => y_flips += 1,
                    _ => {}
                }
                y_sign = -1;
            }
            _ => {
                // ay is 0 or NaN, which doesn't change sign state
            }
        }

        // Early exit optimization: if we've seen more than 2 flips, it's not convex
        if y_flips > 2 {
            return false;
        }
        // Check orientation consistency using robust orientation predicate
        // This detects when the polygon "turns" in different directions
        // (indicating concavity or self-intersection)
        let current_orientation = T::Ker::orient2d(prev, curr, *next);

        // Skip collinear points for orientation consistency check
        if current_orientation != Orientation::Collinear {
            if let Some(established_orientation) = w_orientation {
                if current_orientation != established_orientation {
                    // Orientation change detected - not convex
                    return false;
                }
            } else {
                w_orientation = Some(current_orientation); // Establish initial orientation
            }
        }
        // Advance sliding window to next vertex
        prev = curr;
        curr = *next;
    }

    // Check final/wraparound sign flips
    // We need to check if the final sign differs from the first sign
    // to account for the wraparound from last vertex back to first
    if x_sign != 0 && x_first_sign != 0 && x_sign != x_first_sign {
        x_flips += 1;
    }
    if y_sign != 0 && y_first_sign != 0 && y_sign != y_first_sign {
        y_flips += 1;
    }

    // Final convexity check based on sign flip count
    // Mathematical principle: convex polygons have exactly 2 sign flips per axis
    // This is because as you traverse a convex polygon, you go:
    // - Right → Up → Left → Down (or similar pattern) = 2 flips per axis
    if vertex_count == 3 {
        // Edge case 2: Triangle special handling
        // Triangles can have irregular sign flip patterns but are always convex
        // if orientation is consistent (already checked above with robust predicate)
        w_orientation.is_some() || (x_flips <= 2 && y_flips <= 2)
    } else {
        // Standard case: Regular polygons with 4+ vertices
        // Must have exactly 2 sign flips per axis for convexity
        x_flips == 2 && y_flips == 2
    }
}

/// A utility that tests convexity of a sequence of
/// coordinates. It verifies that for all `0 <= i < n`, the
/// vertices at positions `i`, `i+1`, `i+2` (mod `n`) have
/// the same orientation, optionally accepting collinear
/// triplets, and expecting a specific orientation. The
/// output is `None` or the only non-collinear orientation,
/// unless everything is collinear.
fn is_convex_shaped<T>(
    coords: &[Coord<T>],
    allow_collinear: bool,
    specific_orientation: Option<Orientation>,
) -> Option<Orientation>
where
    T: GeoNum,
{
    if coords.is_empty() {
        // empty here means original linestring had 1 point
        return Some(Orientation::Collinear);
    }

    // drop second of adjacent identical points
    let c0 = coords[0];
    let mut coords: VecDeque<Coord<T>> = coords
        .windows(2)
        .filter_map(|w| if w[0] == w[1] { None } else { Some(w[1]) })
        .collect();
    coords.push_front(c0);

    let n = coords.len();

    let orientation_at = |i: usize| {
        let coord = coords[i];
        let next = coords[(i + 1) % n];
        let nnext = coords[(i + 2) % n];
        (i, T::Ker::orient2d(coord, next, nnext))
    };

    let find_first_non_collinear = (0..n).map(orientation_at).find_map(|(i, orientation)| {
        match orientation {
            Orientation::Collinear => {
                // If collinear accepted, we skip, otherwise
                // stop.
                if allow_collinear {
                    None
                } else {
                    Some((i, orientation))
                }
            }
            _ => Some((i, orientation)),
        }
    });

    let (i, first_non_collinear) = if let Some((i, orientation)) = find_first_non_collinear {
        match orientation {
            Orientation::Collinear => {
                // Only happens if !allow_collinear
                assert!(!allow_collinear);
                return None;
            }
            _ => (i, orientation),
        }
    } else {
        // Empty or everything collinear, and allowed.
        return Some(Orientation::Collinear);
    };

    // If a specific orientation is expected, accept only that.
    if let Some(req_orientation) = specific_orientation {
        if req_orientation != first_non_collinear {
            return None;
        }
    }

    // Now we have a fixed orientation expected at the rest
    // of the coords. Loop to check everything matches it.
    if ((i + 1)..n)
        .map(orientation_at)
        .all(|(_, orientation)| match orientation {
            Orientation::Collinear => allow_collinear,
            orientation => orientation == first_non_collinear,
        })
    {
        Some(first_non_collinear)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithm::Convert;
    use crate::wkt;
    use geo_types::line_string;

    #[test]
    fn test_corner_cases() {
        // This is just tested to ensure there is no panic
        // due to out-of-index access
        let empty: LineString = line_string!();
        assert!(empty.is_collinear());
        assert!(empty.is_convex());
        assert!(!empty.is_strictly_ccw_convex());

        let one = line_string![(x: 0., y: 0.)];
        assert!(one.is_collinear());
        assert!(one.is_convex());
        assert!(one.is_cw_convex());
        assert!(one.is_ccw_convex());
        assert!(one.is_strictly_convex());
        assert!(!one.is_strictly_ccw_convex());
        assert!(!one.is_strictly_cw_convex());

        let one_rep = line_string![(x: 0, y: 0), (x: 0, y: 0)];
        assert!(one_rep.is_collinear());
        assert!(one_rep.is_convex());
        assert!(one_rep.is_cw_convex());
        assert!(one_rep.is_ccw_convex());
        assert!(!one_rep.is_strictly_convex());
        assert!(!one_rep.is_strictly_ccw_convex());
        assert!(!one_rep.is_strictly_cw_convex());

        let mut two = line_string![(x: 0, y: 0), (x: 1, y: 1)];
        assert!(two.is_collinear());
        assert!(!two.is_convex());

        two.close();
        assert!(two.is_cw_convex());
        assert!(two.is_ccw_convex());
        assert!(!two.is_strictly_convex());
        assert!(!two.is_strictly_ccw_convex());
        assert!(!two.is_strictly_cw_convex());
    }

    #[test]
    fn test_duplicate_pt() {
        let ls_unclosed: LineString<f64> = wkt! (LINESTRING (0 0, 1 1, 2 0)).convert();

        let ls_cw: LineString<f64> = wkt! (LINESTRING (0 0, 2 0, 1 1, 0 0)).convert();

        let ls_1: LineString<f64> = wkt! (LINESTRING (0 0, 1 1, 2 0, 0 0)).convert();
        let ls_2: LineString<f64> = wkt! (LINESTRING (0 0, 1 1, 2 0, 2 0, 0 0)).convert();
        let ls_3: LineString<f64> = wkt! (LINESTRING (0 0, 1 1, 2 0, 2 0, 2 0, 0 0)).convert();

        assert!(!ls_unclosed.is_convex());

        assert!(ls_cw.is_convex());
        assert!(ls_cw.is_ccw_convex());

        assert!(ls_1.is_convex());
        assert!(ls_1.is_cw_convex());
        assert!(ls_2.is_convex());
        assert!(ls_2.is_cw_convex());
        assert!(ls_3.is_convex());
        assert!(ls_3.is_cw_convex());
    }

    #[test]
    fn test_single_point() {
        // single point is closed
        // will panic if is_empty check in `is_convex_shaped` is removed
        let ls: LineString<f64> = wkt! (LINESTRING (0 0)).convert();
        assert!(ls.is_strictly_convex());
    }

    #[test]
    fn test_star_polygon_not_convex() {
        // Star polygons have self-intersecting edges and are not convex
        // even though all vertices have the same orientation
        let ls: LineString<f64> =
            wkt! (LINESTRING (0 0, 10 0, 7 -1, 4 5, 6 5, 3 -1, 0 0)).convert();
        assert!(!ls.is_convex());
    }
}
