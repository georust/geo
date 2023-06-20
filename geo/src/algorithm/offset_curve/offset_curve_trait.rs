// TODO: Should I be doing `use crate ::{...}` or `use geo_types::{...}`
use crate::{Coord, CoordFloat, Line, LineString, MultiLineString};

use super::line_intersection::{
    FalseIntersectionPointType::AfterEnd,
    LineIntersectionResultWithRelationships,
    LineSegmentIntersectionType::{FalseIntersectionPoint, TrueIntersectionPoint},
};

use super::line_measured::LineMeasured;

use super::vector_extensions::VectorExtensions;

use super::offset_line_raw::offset_line_raw;

use super::offset_segments_iterator::{LineStringOffsetSegmentPairs, OffsetSegmentsIteratorItem};


/// The OffsetCurve trait is implemented for geometries where the edges of the
/// geometry can be offset perpendicular to the direction of the edges by some
/// positive or negative distance. For example, an offset [Line] will become a
/// [Line], and an offset [LineString] will become a [LineString].
/// Geometry with no length ([geo_types::Point]) cannot be offset as it has no
/// directionality.
///
/// > NOTE: The [OffsetCurve::offset_curve()] function is different to a `buffer` operation.
/// > A buffer (or inset / outset operation) would normally produce an enclosed
/// > shape; For example a [geo_types::Point] would become a circular
/// > [geo_types::Polygon], a [geo_types::Line] would become a capsule shaped
/// > [geo_types::Polygon].

pub trait OffsetCurve<T>
where
    T: CoordFloat,
    Self: Sized,
{
    /// Offset the edges of the geometry by `distance`.
    ///
    /// In a coordinate system where positive is up and to the right;
    /// when facing the direction of increasing coordinate index:
    ///
    /// - Positive `distance` will offset the edges of a geometry to the left
    /// - Negative `distance` will offset the edges of a geometry to the right
    ///
    /// If you are using 'screen coordinates' where the y axis is often flipped
    /// then the offset direction described above will be reversed.
    ///
    /// # Examples
    ///
    /// ```
    /// #use crate::{line_string, Coord};
    /// let input = line_string![
    ///     Coord { x: 0f64, y: 0f64 },
    ///     Coord { x: 0f64, y: 2f64 },
    ///     Coord { x: 2f64, y: 2f64 },
    /// ];
    /// let output_expected = line_string![
    ///     Coord { x: 1f64, y: 0f64 },
    ///     Coord { x: 1f64, y: 1f64 },
    ///     Coord { x: 2f64, y: 1f64 },
    /// ];
    /// let output_actual = input.offset_curve(-1f64).unwrap();
    /// assert_eq!(output_actual, output_expected);
    /// ```
    fn offset_curve(&self, distance: T) -> Option<Self>;
}

impl<T> OffsetCurve<T> for Line<T>
where
    T: CoordFloat,
{
    fn offset_curve(&self, distance: T) -> Option<Self> {
        if distance == T::zero() {
            // prevent unnecessary work;
            Some(self.clone())
            // TODO: for typical use cases the offset would rarely be zero;
            // This check may add unnecessary branching when there are a lot of
            // Lines. It makes more sense to do this performance check for
            // LineStrings...
        } else {
            let Line { start: a, end: b } = *self;
            match offset_line_raw(a, b, distance) {
                Some(LineMeasured { line, .. }) => Some(line),
                _ => None,
            }
        }
    }
}

impl<T> OffsetCurve<T> for LineString<T>
where
    T: CoordFloat,
{
    fn offset_curve(&self, distance: T) -> Option<Self> {
        // Loosely follows the algorithm described by
        // [Xu-Zheng Liu, Jun-Hai Yong, Guo-Qin Zheng, Jia-Guang Sun. An offset
        // algorithm for polyline curves. Computers in Industry, Elsevier, 2007,
        // 15p. inria-00518005]
        // (https://hal.inria.fr/inria-00518005/document)

        // Handle trivial cases;
        // Note: Docs say LineString is valid "if it is either empty or contains
        //       two or more coordinates"

        // TODO: is `self.into_inner()` rather than `self.0` preferred? The
        //       contents of the tuple struct are public.
        //       Issue #816 seems to suggest that `self.0` is to be deprecated
        match self.0.len() {
            0 => return Some(self.clone()),
            1 => return None,
            2 => {
                return match Line::new(self.0[0], self.0[1]).offset_curve(distance) {
                    Some(line) => Some(line.into()),
                    None => None,
                }
            }
            _ => (),
        }

        // Prevent unnecessary work:
        if T::is_zero(&distance) {
            return Some(self.clone());
        }

        // TODO: Parameterize miter limit, and miter limit distance / factor.
        let mitre_limit_factor = T::from(2.0).unwrap();
        let mitre_limit_distance = distance.abs() * mitre_limit_factor;
        let mitre_limit_distance_squared = mitre_limit_distance * mitre_limit_distance;

        let mut offset_points = Vec::with_capacity(self.0.len());

        for item in self.iter_offset_segment_pairs(distance) {
            println!("{item:?}");
            if let OffsetSegmentsIteratorItem {
                ab_offset: Some(LineMeasured { line, .. }),
                first: true,
                ..
            } = item
            {
                offset_points.push(line.start)
            };
            match item {
                OffsetSegmentsIteratorItem {
                    ab_offset:
                        Some(LineMeasured {
                            line: Line { start: m, end: n },
                            length: ab_len,
                        }),
                    bc_offset:
                        Some(LineMeasured {
                            line: Line { start: o, end: p },
                            length: bc_len,
                        }),
                    i:
                        Some(LineIntersectionResultWithRelationships {
                            ab,
                            cd,
                            intersection,
                        }),
                    ..
                } => match (ab, cd) {
                    (TrueIntersectionPoint, TrueIntersectionPoint) => {
                        // Inside elbow
                        // No mitre limit needed
                        offset_points.push(intersection)
                    }
                    (FalseIntersectionPoint(AfterEnd), FalseIntersectionPoint(_)) => {
                        // Outside elbow
                        // Check for Mitre Limit
                        let elbow_length_squared = (intersection - n).magnitude_squared();
                        if elbow_length_squared > mitre_limit_distance_squared {
                            // Mitre Limited / Truncated Corner
                            let mn: Coord<T> = n - m;
                            let op: Coord<T> = p - o;
                            offset_points.push(n + mn / ab_len * mitre_limit_distance);
                            offset_points.push(o - op / bc_len * mitre_limit_distance);
                        } else {
                            // Sharp Corner
                            offset_points.push(intersection)
                        }
                    }
                    _ => {
                        // Inside pinched elbow
                        // (ie forearm curled back through bicep 🙃)
                        //println!("CASE 3 - bridge");
                        offset_points.push(n);
                        offset_points.push(o);
                    }
                },
                OffsetSegmentsIteratorItem {
                    ab_offset:
                        Some(LineMeasured {
                            line: Line { end: n, .. },
                            ..
                        }),
                    i: None,
                    ..
                } => {
                    // Collinear
                    // TODO: this is not an elegant way to handle colinear
                    // input: in some (all?) cases this produces a redundant
                    // colinear point in the output. It might be easier to
                    // eliminate this redundant point in a pre-processing step
                    // rather than try do it here.
                    offset_points.push(n)
                }
                _ => {
                    // Several ways to end up here... probably one of the
                    // segments could not be offset
                    return None;
                }
            }
            if let OffsetSegmentsIteratorItem {
                bc_offset: Some(LineMeasured { line, .. }),
                last: true,
                ..
            } = item
            {
                offset_points.push(line.end)
            };
        }

        Some(offset_points.into())
    }
}

impl<T> OffsetCurve<T> for MultiLineString<T>
where
    T: CoordFloat,
{
    fn offset_curve(&self, distance: T) -> Option<Self> {
        self.iter()
            .map(|item| item.offset_curve(distance))
            .collect()
    }
}

#[cfg(test)]
mod test {

    use crate::{
        line_string,
        Coord,
        Line,
        //LineString,
        MultiLineString,
        OffsetCurve,
    };

    #[test]
    fn test_offset_line() {
        let input = Line::new(Coord { x: 1f64, y: 1f64 }, Coord { x: 1f64, y: 2f64 });
        let output_actual = input.offset_curve(-1.0);
        let output_expected = Some(Line::new(
            Coord { x: 2f64, y: 1f64 },
            Coord { x: 2f64, y: 2f64 },
        ));
        assert_eq!(output_actual, output_expected);
    }
    #[test]
    fn test_offset_line_negative() {
        let input = Line::new(Coord { x: 1f64, y: 1f64 }, Coord { x: 1f64, y: 2f64 });
        let output_actual = input.offset_curve(1.0);
        let output_expected = Some(Line::new(
            Coord { x: 0f64, y: 1f64 },
            Coord { x: 0f64, y: 2f64 },
        ));
        assert_eq!(output_actual, output_expected);
    }

    #[test]
    fn test_offset_line_string() {
        let input = line_string![
            Coord { x: 0f64, y: 0f64 },
            Coord { x: 0f64, y: 2f64 },
            Coord { x: 2f64, y: 2f64 },
        ];
        let output_actual = input.offset_curve(-1f64);
        let output_expected = Some(line_string![
            Coord { x: 1f64, y: 0f64 },
            Coord { x: 1f64, y: 1f64 },
            Coord { x: 2f64, y: 1f64 },
        ]);
        assert_eq!(output_actual, output_expected);
    }

    #[test]
    fn test_offset_line_string_invalid() {
        let input = line_string![Coord { x: 0f64, y: 0f64 },];
        let output_actual = input.offset_curve(-1f64);
        let output_expected = None;
        assert_eq!(output_actual, output_expected);
    }

    #[test]
    fn test_offset_multi_line_string() {
        let input = MultiLineString::new(vec![
            line_string![
                Coord { x: 0f64, y: 0f64 },
                Coord { x: 0f64, y: 2f64 },
                Coord { x: 2f64, y: 2f64 },
            ],
            line_string![
                Coord { x: 0f64, y: 0f64 },
                Coord { x: 0f64, y: -2f64 },
                Coord { x: -2f64, y: -2f64 },
            ],
        ]);
        let output_actual = input.offset_curve(-1f64);
        let output_expected = Some(MultiLineString::new(vec![
            line_string![
                Coord { x: 1f64, y: 0f64 },
                Coord { x: 1f64, y: 1f64 },
                Coord { x: 2f64, y: 1f64 },
            ],
            line_string![
                Coord { x: -1f64, y: 0f64 },
                Coord { x: -1f64, y: -1f64 },
                Coord { x: -2f64, y: -1f64 },
            ],
        ]));
        assert_eq!(output_actual, output_expected);
    }
}
