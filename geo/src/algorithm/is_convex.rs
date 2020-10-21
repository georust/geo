use crate::kernels::*;
use crate::{Coordinate, LineString};

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
/// be closed, but the rest of the predicates do.
///
/// - This definition is closely related to the notion
/// of [convexity of polygons][convex set]. In particular, a
/// [`Polygon`](crate::Polygon) is convex, if and only if its `exterior` is
/// convex, and `interiors` is empty.
///
/// - The [`ConvexHull`] algorithm always returns a strictly
/// convex `LineString` unless the input is empty or
/// collinear. The [`graham_hull`] algorithm provides an
/// option to include collinear points, producing a
/// (possibly non-strict) convex `LineString`.
///
/// # Edge Cases
///
/// - the convexity, and collinearity of an empty
/// `LineString` is _unspecified_ and must not be relied
/// upon.
///
/// - A closed `LineString` with at most three coordinates
/// (including the possibly repeated first coordinate) is
/// both convex and collinear. However, the strict convexity
/// is _unspecified_ and must not be relied upon.
///
/// [convex combination]: //en.wikipedia.org/wiki/Convex_combination
/// [convex set]: //en.wikipedia.org/wiki/Convex_set
/// [`ConvexHull`]: crate::algorithm::convex_hull::ConvexHull
/// [`graham_hull`]: crate::algorithm::convex_hull::graham_hull
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

impl<T: HasKernel> IsConvex for LineString<T> {
    fn convex_orientation(
        &self,
        allow_collinear: bool,
        specific_orientation: Option<Orientation>,
    ) -> Option<Orientation> {
        if !self.is_closed() || self.0.is_empty() {
            None
        } else {
            is_convex_shaped(&self.0[1..], allow_collinear, specific_orientation)
        }
    }

    fn is_collinear(&self) -> bool {
        self.0.is_empty()
            || is_convex_shaped(&self.0[1..], true, Some(Orientation::Collinear)).is_some()
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
    coords: &[Coordinate<T>],
    allow_collinear: bool,
    specific_orientation: Option<Orientation>,
) -> Option<Orientation>
where
    T: HasKernel,
{
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
        .find(|&(_, orientation)| match orientation {
            Orientation::Collinear => !allow_collinear,
            orientation => !(orientation == first_non_collinear),
        })
        .is_none()
    {
        Some(first_non_collinear)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_types::line_string;

    #[test]
    fn test_corner_cases() {
        // This is just tested to ensure there is no panic
        // due to out-of-index access
        let empty: LineString<f64> = line_string!();
        assert!(empty.is_collinear());
        assert!(!empty.is_convex());
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
}
