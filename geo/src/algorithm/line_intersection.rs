use crate::{Coordinate, GeoFloat, Line};
use geo_types::coord;

use crate::BoundingRect;
use crate::Intersects;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum LineIntersection<F: GeoFloat> {
    /// Lines intersect in a single point
    SinglePoint {
        intersection: Coordinate<F>,
        /// For Lines which intersect in a single point, that point may be either an endpoint
        /// or in the interior of each Line.
        /// If the point lies in the interior of both Lines, we call it a _proper_ intersection.
        ///
        /// # Note
        ///
        /// Due to the limited precision of most float data-types, the
        /// calculated intersection point may be snapped to one of the
        /// end-points even though all the end-points of the two
        /// lines are distinct points. In such cases, this field is
        /// still set to `true`. Please refer test_case:
        /// `test_central_endpoint_heuristic_failure_1` for such an
        /// example.
        is_proper: bool,
    },

    /// Overlapping Lines intersect in a line segment
    Collinear { intersection: Line<F> },
}

impl<F: GeoFloat> LineIntersection<F> {
    pub fn is_proper(&self) -> bool {
        match self {
            Self::Collinear { .. } => false,
            Self::SinglePoint { is_proper, .. } => *is_proper,
        }
    }
}

/// Returns the intersection between two [`Lines`](Line).
///
/// Lines can intersect in a Point or, for Collinear lines, in a Line. See [`LineIntersection`]
/// for more details about the result.
///
/// # Examples
///
/// ```
/// use geo_types::coord;
/// use geo::{Line, Coordinate};
/// use geo::line_intersection::{line_intersection, LineIntersection};
///
/// let line_1 = Line::new(coord! {x: 0.0, y: 0.0}, coord! { x: 5.0, y: 5.0 } );
/// let line_2 = Line::new(coord! {x: 0.0, y: 5.0}, coord! { x: 5.0, y: 0.0 } );
/// let expected = LineIntersection::SinglePoint { intersection: coord! { x: 2.5, y: 2.5 }, is_proper: true };
/// assert_eq!(line_intersection(line_1, line_2), Some(expected));
///
/// let line_1 = Line::new(coord! {x: 0.0, y: 0.0}, coord! { x: 5.0, y: 5.0 } );
/// let line_2 = Line::new(coord! {x: 0.0, y: 1.0}, coord! { x: 5.0, y: 6.0 } );
/// assert_eq!(line_intersection(line_1, line_2), None);
///
/// let line_1 = Line::new(coord! {x: 0.0, y: 0.0}, coord! { x: 5.0, y: 5.0 } );
/// let line_2 = Line::new(coord! {x: 5.0, y: 5.0}, coord! { x: 5.0, y: 0.0 } );
/// let expected = LineIntersection::SinglePoint { intersection: coord! { x: 5.0, y: 5.0 }, is_proper: false };
/// assert_eq!(line_intersection(line_1, line_2), Some(expected));
///
/// let line_1 = Line::new(coord! {x: 0.0, y: 0.0}, coord! { x: 5.0, y: 5.0 } );
/// let line_2 = Line::new(coord! {x: 3.0, y: 3.0}, coord! { x: 6.0, y: 6.0 } );
/// let expected = LineIntersection::Collinear { intersection: Line::new(coord! { x: 3.0, y: 3.0 }, coord! { x: 5.0, y: 5.0 })};
/// assert_eq!(line_intersection(line_1, line_2), Some(expected));
/// ```
/// Strongly inspired by, and meant to produce the same results as, [JTS's RobustLineIntersector](https://github.com/locationtech/jts/blob/master/modules/core/src/main/java/org/locationtech/jts/algorithm/RobustLineIntersector.java#L26).
pub fn line_intersection<F>(p: Line<F>, q: Line<F>) -> Option<LineIntersection<F>>
where
    F: GeoFloat,
{
    if !p.bounding_rect().intersects(&q.bounding_rect()) {
        return None;
    }

    use crate::kernels::{Kernel, Orientation::*, RobustKernel};
    let p_q1 = RobustKernel::orient2d(p.start, p.end, q.start);
    let p_q2 = RobustKernel::orient2d(p.start, p.end, q.end);
    if matches!(
        (p_q1, p_q2),
        (Clockwise, Clockwise) | (CounterClockwise, CounterClockwise)
    ) {
        return None;
    }

    let q_p1 = RobustKernel::orient2d(q.start, q.end, p.start);
    let q_p2 = RobustKernel::orient2d(q.start, q.end, p.end);
    if matches!(
        (q_p1, q_p2),
        (Clockwise, Clockwise) | (CounterClockwise, CounterClockwise)
    ) {
        return None;
    }

    if matches!(
        (p_q1, p_q2, q_p1, q_p2),
        (Collinear, Collinear, Collinear, Collinear)
    ) {
        return collinear_intersection(p, q);
    }

    // At this point we know that there is a single intersection point (since the lines are not
    // collinear).
    //
    // Check if the intersection is an endpoint. If it is, copy the endpoint as the
    // intersection point. Copying the point rather than computing it ensures the point has the
    // exact value, which is important for robustness. It is sufficient to simply check for an
    // endpoint which is on the other line, since at this point we know that the inputLines
    // must intersect.
    if p_q1 == Collinear || p_q2 == Collinear || q_p1 == Collinear || q_p2 == Collinear {
        // Check for two equal endpoints.
        // This is done explicitly rather than by the orientation tests below in order to improve
        // robustness.
        //
        // [An example where the orientation tests fail to be consistent is the following (where
        // the true intersection is at the shared endpoint
        // POINT (19.850257749638203 46.29709338043669)
        //
        // LINESTRING ( 19.850257749638203 46.29709338043669, 20.31970698357233 46.76654261437082 )
        // and
        // LINESTRING ( -48.51001596420236 -22.063180333403878, 19.850257749638203 46.29709338043669 )
        //
        // which used to produce the INCORRECT result: (20.31970698357233, 46.76654261437082, NaN)

        let intersection: Coordinate<F>;
        // false positives for this overzealous clippy https://github.com/rust-lang/rust-clippy/issues/6747
        #[allow(clippy::suspicious_operation_groupings)]
        if p.start == q.start || p.start == q.end {
            intersection = p.start;
        } else if p.end == q.start || p.end == q.end {
            intersection = p.end;
            // Now check to see if any endpoint lies on the interior of the other segment.
        } else if p_q1 == Collinear {
            intersection = q.start;
        } else if p_q2 == Collinear {
            intersection = q.end;
        } else if q_p1 == Collinear {
            intersection = p.start;
        } else {
            assert_eq!(q_p2, Collinear);
            intersection = p.end;
        }
        Some(LineIntersection::SinglePoint {
            intersection,
            is_proper: false,
        })
    } else {
        let intersection = proper_intersection(p, q);
        Some(LineIntersection::SinglePoint {
            intersection,
            is_proper: true,
        })
    }
}

fn collinear_intersection<F: GeoFloat>(p: Line<F>, q: Line<F>) -> Option<LineIntersection<F>> {
    fn collinear<F: GeoFloat>(intersection: Line<F>) -> LineIntersection<F> {
        LineIntersection::Collinear { intersection }
    }

    fn improper<F: GeoFloat>(intersection: Coordinate<F>) -> LineIntersection<F> {
        LineIntersection::SinglePoint {
            intersection,
            is_proper: false,
        }
    }

    let p_bounds = p.bounding_rect();
    let q_bounds = q.bounding_rect();
    Some(
        match (
            p_bounds.intersects(&q.start),
            p_bounds.intersects(&q.end),
            q_bounds.intersects(&p.start),
            q_bounds.intersects(&p.end),
        ) {
            (true, true, _, _) => collinear(q),
            (_, _, true, true) => collinear(p),
            (true, false, true, false) if q.start == p.start => improper(q.start),
            (true, _, true, _) => collinear(Line::new(q.start, p.start)),
            (true, false, false, true) if q.start == p.end => improper(q.start),
            (true, _, _, true) => collinear(Line::new(q.start, p.end)),
            (false, true, true, false) if q.end == p.start => improper(q.end),
            (_, true, true, _) => collinear(Line::new(q.end, p.start)),
            (false, true, false, true) if q.end == p.end => improper(q.end),
            (_, true, _, true) => collinear(Line::new(q.end, p.end)),
            _ => return None,
        },
    )
}

/// Finds the endpoint of the segments P and Q which is closest to the other segment.  This is
/// a reasonable surrogate for the true intersection points in ill-conditioned cases (e.g.
/// where two segments are nearly coincident, or where the endpoint of one segment lies almost
/// on the other segment).
///
/// This replaces the older CentralEndpoint heuristic, which chose the wrong endpoint in some
/// cases where the segments had very distinct slopes and one endpoint lay almost on the other
/// segment.
///
/// `returns` the nearest endpoint to the other segment
fn nearest_endpoint<F: GeoFloat>(p: Line<F>, q: Line<F>) -> Coordinate<F> {
    use geo_types::private_utils::point_line_euclidean_distance;

    let mut nearest_pt = p.start;
    let mut min_dist = point_line_euclidean_distance(p.start, q);

    let dist = point_line_euclidean_distance(p.end, q);
    if dist < min_dist {
        min_dist = dist;
        nearest_pt = p.end;
    }
    let dist = point_line_euclidean_distance(q.start, p);
    if dist < min_dist {
        min_dist = dist;
        nearest_pt = q.start;
    }
    let dist = point_line_euclidean_distance(q.end, p);
    if dist < min_dist {
        nearest_pt = q.end;
    }
    nearest_pt
}

fn raw_line_intersection<F: GeoFloat>(p: Line<F>, q: Line<F>) -> Option<Coordinate<F>> {
    let p_min_x = p.start.x.min(p.end.x);
    let p_min_y = p.start.y.min(p.end.y);
    let p_max_x = p.start.x.max(p.end.x);
    let p_max_y = p.start.y.max(p.end.y);

    let q_min_x = q.start.x.min(q.end.x);
    let q_min_y = q.start.y.min(q.end.y);
    let q_max_x = q.start.x.max(q.end.x);
    let q_max_y = q.start.y.max(q.end.y);

    let int_min_x = p_min_x.max(q_min_x);
    let int_max_x = p_max_x.min(q_max_x);
    let int_min_y = p_min_y.max(q_min_y);
    let int_max_y = p_max_y.min(q_max_y);

    let two = F::one() + F::one();
    let mid_x = (int_min_x + int_max_x) / two;
    let mid_y = (int_min_y + int_max_y) / two;

    // condition ordinate values by subtracting midpoint
    let p1x = p.start.x - mid_x;
    let p1y = p.start.y - mid_y;
    let p2x = p.end.x - mid_x;
    let p2y = p.end.y - mid_y;
    let q1x = q.start.x - mid_x;
    let q1y = q.start.y - mid_y;
    let q2x = q.end.x - mid_x;
    let q2y = q.end.y - mid_y;

    // unrolled computation using homogeneous coordinates eqn
    let px = p1y - p2y;
    let py = p2x - p1x;
    let pw = p1x * p2y - p2x * p1y;

    let qx = q1y - q2y;
    let qy = q2x - q1x;
    let qw = q1x * q2y - q2x * q1y;

    let xw = py * qw - qy * pw;
    let yw = qx * pw - px * qw;
    let w = px * qy - qx * py;

    let x_int = xw / w;
    let y_int = yw / w;

    // check for parallel lines
    if (x_int.is_nan() || x_int.is_infinite()) || (y_int.is_nan() || y_int.is_infinite()) {
        None
    } else {
        // de-condition intersection point
        Some(coord! {
            x: x_int + mid_x,
            y: y_int + mid_y,
        })
    }
}

/// This method computes the actual value of the intersection point.
/// To obtain the maximum precision from the intersection calculation,
/// the coordinates are normalized by subtracting the minimum
/// ordinate values (in absolute value).  This has the effect of
/// removing common significant digits from the calculation to
/// maintain more bits of precision.
fn proper_intersection<F: GeoFloat>(p: Line<F>, q: Line<F>) -> Coordinate<F> {
    // Computes a segment intersection using homogeneous coordinates.
    // Round-off error can cause the raw computation to fail,
    // (usually due to the segments being approximately parallel).
    // If this happens, a reasonable approximation is computed instead.
    let mut int_pt = raw_line_intersection(p, q).unwrap_or_else(|| nearest_endpoint(p, q));

    // NOTE: At this point, JTS does a `Envelope::contains(coord)` check, but confusingly,
    // Envelope::contains(coord) in JTS is actually an *intersection* check, not a true SFS
    // `contains`, because it includes the boundary of the rect.
    if !(p.bounding_rect().intersects(&int_pt) && q.bounding_rect().intersects(&int_pt)) {
        // compute a safer result
        // copy the coordinate, since it may be rounded later
        int_pt = nearest_endpoint(p, q);
    }
    int_pt
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::geo_types::coord;

    /// Based on JTS test `testCentralEndpointHeuristicFailure`
    /// > Following cases were failures when using the CentralEndpointIntersector heuristic.
    /// > This is because one segment lies at a significant angle to the other,
    /// > with only one endpoint is close to the other segment.
    /// > The CE heuristic chose the wrong endpoint to return.
    /// > The fix is to use a new heuristic which out of the 4 endpoints
    /// > chooses the one which is closest to the other segment.
    /// > This works in all known failure cases.
    #[test]
    fn test_central_endpoint_heuristic_failure_1() {
        let line_1 = Line::new(
            coord! {
                x: 163.81867067,
                y: -211.31840378,
            },
            coord! {
                x: 165.9174252,
                y: -214.1665075,
            },
        );
        let line_2 = Line::new(
            coord! {
                x: 2.84139601,
                y: -57.95412726,
            },
            coord! {
                x: 469.59990601,
                y: -502.63851732,
            },
        );
        let actual = line_intersection(line_1, line_2);
        let expected = LineIntersection::SinglePoint {
            intersection: coord! {
                x: 163.81867067,
                y: -211.31840378,
            },
            is_proper: true,
        };
        assert_eq!(actual, Some(expected));
    }

    /// Based on JTS test `testCentralEndpointHeuristicFailure2`
    /// > Test from Tomas Fa - JTS list 6/13/2012
    /// >
    /// > Fails using original JTS DeVillers determine orientation test.
    /// > Succeeds using DD and Shewchuk orientation
    #[test]
    fn test_central_endpoint_heuristic_failure_2() {
        let line_1 = Line::new(
            coord! {
                x: -58.00593335955,
                y: -1.43739086465,
            },
            coord! {
                x: -513.86101637525,
                y: -457.29247388035,
            },
        );
        let line_2 = Line::new(
            coord! {
                x: -215.22279674875,
                y: -158.65425425385,
            },
            coord! {
                x: -218.1208801283,
                y: -160.68343590235,
            },
        );
        let actual = line_intersection(line_1, line_2);
        let expected = LineIntersection::SinglePoint {
            intersection: coord! {
                x: -215.22279674875,
                y: -158.65425425385,
            },
            is_proper: true,
        };
        assert_eq!(actual, Some(expected));
    }

    /// Based on JTS test `testTomasFa_1`
    /// > Test from Tomas Fa - JTS list 6/13/2012
    /// >
    /// > Fails using original JTS DeVillers determine orientation test.
    /// > Succeeds using DD and Shewchuk orientation
    #[test]
    fn test_tomas_fa_1() {
        let line_1 = Line::<f64>::new(coord! { x: -42.0, y: 163.2 }, coord! { x: 21.2, y: 265.2 });
        let line_2 = Line::<f64>::new(coord! { x: -26.2, y: 188.7 }, coord! { x: 37.0, y: 290.7 });
        let actual = line_intersection(line_1, line_2);
        let expected = None;
        assert_eq!(actual, expected);
    }

    /// Based on JTS test `testTomasFa_2`
    ///
    /// > Test from Tomas Fa - JTS list 6/13/2012
    /// >
    /// > Fails using original JTS DeVillers determine orientation test.
    #[test]
    fn test_tomas_fa_2() {
        let line_1 = Line::<f64>::new(coord! { x: -5.9, y: 163.1 }, coord! { x: 76.1, y: 250.7 });
        let line_2 = Line::<f64>::new(coord! { x: 14.6, y: 185.0 }, coord! { x: 96.6, y: 272.6 });
        let actual = line_intersection(line_1, line_2);
        let expected = None;
        assert_eq!(actual, expected);
    }

    /// Based on JTS test `testLeduc_1`
    ///
    /// > Test involving two non-almost-parallel lines.
    /// > Does not seem to cause problems with basic line intersection algorithm.
    #[test]
    fn test_leduc_1() {
        let line_1 = Line::new(
            coord! {
                x: 305690.0434123494,
                y: 254176.46578338774,
            },
            coord! {
                x: 305601.9999843455,
                y: 254243.19999846347,
            },
        );
        let line_2 = Line::new(
            coord! {
                x: 305689.6153764265,
                y: 254177.33102743194,
            },
            coord! {
                x: 305692.4999844298,
                y: 254171.4999983967,
            },
        );
        let actual = line_intersection(line_1, line_2);
        let expected = LineIntersection::SinglePoint {
            intersection: coord! {
                x: 305690.0434123494,
                y: 254176.46578338774,
            },
            is_proper: true,
        };
        assert_eq!(actual, Some(expected));
    }

    /// Based on JTS test `testGEOS_1()`
    ///
    /// > Test from strk which is bad in GEOS (2009-04-14).
    #[test]
    fn test_geos_1() {
        let line_1 = Line::new(
            coord! {
                x: 588750.7429703881,
                y: 4518950.493668233,
            },
            coord! {
                x: 588748.2060409798,
                y: 4518933.9452804085,
            },
        );
        let line_2 = Line::new(
            coord! {
                x: 588745.824857241,
                y: 4518940.742239175,
            },
            coord! {
                x: 588748.2060437313,
                y: 4518933.9452791475,
            },
        );
        let actual = line_intersection(line_1, line_2);
        let expected = LineIntersection::SinglePoint {
            intersection: coord! {
                x: 588748.2060416829,
                y: 4518933.945284994,
            },
            is_proper: true,
        };
        assert_eq!(actual, Some(expected));
    }

    /// Based on JTS test `testGEOS_2()`
    ///
    /// > Test from strk which is bad in GEOS (2009-04-14).
    #[test]
    fn test_geos_2() {
        let line_1 = Line::new(
            coord! {
                x: 588743.626135934,
                y: 4518924.610969561,
            },
            coord! {
                x: 588732.2822865889,
                y: 4518925.4314047815,
            },
        );
        let line_2 = Line::new(
            coord! {
                x: 588739.1191384895,
                y: 4518927.235700594,
            },
            coord! {
                x: 588731.7854614238,
                y: 4518924.578370095,
            },
        );
        let actual = line_intersection(line_1, line_2);
        let expected = LineIntersection::SinglePoint {
            intersection: coord! {
                x: 588733.8306132929,
                y: 4518925.319423238,
            },
            is_proper: true,
        };
        assert_eq!(actual, Some(expected));
    }

    /// Based on JTS test `testDaveSkeaCase()`
    ///
    /// > This used to be a failure case (exception), but apparently works now.
    /// > Possibly normalization has fixed this?
    #[test]
    fn test_dave_skea_case() {
        let line_1 = Line::new(
            coord! {
                x: 2089426.5233462777,
                y: 1180182.387733969,
            },
            coord! {
                x: 2085646.6891757075,
                y: 1195618.7333999649,
            },
        );
        let line_2 = Line::new(
            coord! {
                x: 1889281.8148903656,
                y: 1997547.0560044837,
            },
            coord! {
                x: 2259977.3672236,
                y: 483675.17050843034,
            },
        );
        let actual = line_intersection(line_1, line_2);
        let expected = LineIntersection::SinglePoint {
            intersection: coord! {
                x: 2087536.6062609926,
                y: 1187900.560566967,
            },
            is_proper: true,
        };
        assert_eq!(actual, Some(expected));
    }

    /// Based on JTS test `testCmp5CaseWKT()`
    ///
    /// > Outside envelope using HCoordinate method.
    #[test]
    fn test_cmp_5_cask_wkt() {
        let line_1 = Line::new(
            coord! {
                x: 4348433.262114629,
                y: 5552595.478385733,
            },
            coord! {
                x: 4348440.849387404,
                y: 5552599.272022122,
            },
        );
        let line_2 = Line::new(
            coord! {
                x: 4348433.26211463,
                y: 5552595.47838573,
            },
            coord! {
                x: 4348440.8493874,
                y: 5552599.27202212,
            },
        );
        let actual = line_intersection(line_1, line_2);
        let expected = LineIntersection::SinglePoint {
            intersection: coord! {
                x: 4348440.8493874,
                y: 5552599.27202212,
            },
            is_proper: true,
        };
        assert_eq!(actual, Some(expected));
    }
}
