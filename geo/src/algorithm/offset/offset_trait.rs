use super::line_intersection::FalseIntersectionPointType::AfterEnd;
use super::line_intersection::LineSegmentIntersectionType::{
    FalseIntersectionPoint, TrueIntersectionPoint,
};
use super::line_intersection::{
    line_segment_intersection_with_relationships, LineIntersectionResultWithRelationships,
};
use super::slice_itertools::pairwise;

use crate::offset::cross_product::cross_product_2d;
// TODO: Should I be using the re-exported types or,
//       using these from the geo_types crate?
use crate::{Coord, CoordFloat, CoordNum, Line, LineString, MultiLineString};

/// # Offset Trait
///
/// The offset trait is implemented for geometries where the edges of the
/// geometry can be offset perpendicular to the direction of the edges by some
/// positive or negative distance. For example, an offset [Line] will become a
/// [Line], and an offset [LineString] will become a [LineString].
/// [geo_types::Point] cannot be offset as it has no directionality.
///
/// This the [Offset::offset()] function is a more primitive operation than a
/// 'buffer()' operation. A buffer or inset/outset operation will normally
/// produce an enclosed shape; For example a [geo_types::Point] would become a
/// circular [geo_types::Polygon], a [geo_types::Line] would be come a capsule
/// shaped [geo_types::Polygon].
///
///
///
/// The following are a list of known limitations,
/// some may be removed during development,
/// others are very hard to fix.
///
/// - No checking for zero length input.
///   Invalid results may be caused by division by zero.
/// - No check is implemented to prevent execution if the specified offset
///   distance is zero.
/// - Only local cropping where the output is self-intersecting.
///   Non-adjacent line segments in the output may be self-intersecting.
/// - There is no mitre-limit; A LineString which
///   doubles back on itself will produce an elbow at infinity
pub trait Offset<T>
where
    T: CoordFloat,
    Self: Sized,
{
    /// Offset the edges of the geometry by `distance`.
    ///
    ///
    /// `distance` may also be negative. Negative `distance` values will offset
    /// the edges of the geometry to the left, when facing the direction of
    /// increasing coordinate index.
    ///
    /// For a polygon with clockwise winding order, where the y axis is
    /// northward positive, a positive 'distance' corresponds with an 'inset'.
    /// This direction has been flagged as counter-intuitive in the PR comments.
    /// I will double check this later once I get the polygon offset working and
    /// possibly change the choice of direction.
    ///
    /// If you are using 'screen coordinates' where the y axis is often downward
    /// positive then the offset direction described above will be reversed.
    ///
    /// # TODO
    ///
    /// - [ ] PR comment suggested name change to
    ///       `OffsetCurve::offset_curve(...)` ?
    /// - [ ] check if any other part of Geo sets the northward-positive
    ///       assumption.
    /// - [ ] Consider a `Result` type with a message explaining the reason for
    ///       failure?
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
    /// let output_actual = input.offset(1f64).unwrap();
    /// assert_eq!(output_actual, output_expected);
    /// ```
    fn offset(&self, distance: T) -> Option<Self>;
}

impl<T> Offset<T> for Line<T>
where
    T: CoordFloat,
{
    fn offset(&self, distance: T) -> Option<Self> {
        if distance == T::zero() {
            // prevent unnecessary work
            Some(self.clone())
        } else {
            let delta = self.delta();
            let len = (delta.x * delta.x + delta.y * delta.y).sqrt();
            if T::is_zero(&len) {
                // Cannot offset a zero length Line
                None
            } else {
                let delta_norm = delta / len;
                // Rotate 90 degrees clockwise (right normal)
                // Note that the "rotation direction" depends on the direction of
                // the y coordinate: Geographic systems normally have the y axis
                // northward-positive (like a conventional axes in math). But screen
                // coordinates are sometimes downward-positive, in which case this is
                // the left_normal and everything gets reversed:
                let delta_norm_right = Coord {
                    x: delta_norm.y,
                    y: -delta_norm.x,
                };
                Some(Line::new(
                    self.start + delta_norm_right * distance,
                    self.end + delta_norm_right * distance,
                ))
            }
        }
    }
}

// TODO: Trying to get a custom iterator working to replace the pairwise
//       function
//
// struct OffsetSegmentsIterator<'a, T> where T:CoordNum{
//     line_string:& 'a LineString<T>,
//     distance:T,
//     last_offset_segment:Option<Line<T>>
// }
//
// impl<'a, T> Iterator for OffsetSegmentsIterator<'a, T> where T:CoordNum {
//     type Item = u32;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         Some(5)
//     }
// }
//
// fn line_string_offset_segments<'a, T>(line_string:&'a LineString<T>, distance:T) -> OffsetSegmentsIterator<'a, T> where T:CoordNum {
//     OffsetSegmentsIterator { line_string, distance, last_offset_segment: None }
// }

impl<T> Offset<T> for LineString<T>
where
    T: CoordFloat,
{
    fn offset(&self, distance: T) -> Option<Self> {
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
        match self.0.len() {
            0 => return Some(self.clone()),
            1 => return None,
            2 => {
                return match Line::new(self.0[0], self.0[1]).offset(distance) {
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

        // TODO: I feel like offset_segments should be lazily computed as part
        //       of the main iterator below if possible;
        //       - so we don't need to keep all this in memory at once
        //       - and so that if we have to bail out later we didn't do all this
        //         work for nothing
        //       However I haven't been able to get a nice lazy pairwise
        //       iterator working.. I suspect it requires unsafe code :/
        let offset_segments: Vec<Line<T>> =
            match self.lines().map(|item| item.offset(distance)).collect() {
                Some(a) => a,
                _ => return None, // bail out if any segment fails
            };

        if offset_segments.len() == 1 {
            return Some(offset_segments[0].into());
        }
        // First and last will always work, checked length above:
        // TODO: try to eliminate unwrap anyway?
        let first_point = offset_segments.first().unwrap().start;
        let last_point = offset_segments.last().unwrap().end;

        let mut result = Vec::with_capacity(self.0.len());
        result.push(first_point);
        result.extend(pairwise(&offset_segments[..]).flat_map(
            |(Line { start: a, end: b }, Line { start: c, end: d })| {
                match line_segment_intersection_with_relationships(a, b, c, d) {
                    None => {
                        // TODO: this is the colinear case;
                        // In some cases this creates a redundant point in the
                        // output. Colinear segments should maybe get
                        // merged before or after this algorithm. Not easy
                        // to fix here.
                        //println!("CASE 0 - colinear");
                        vec![*b]
                    }
                    Some(LineIntersectionResultWithRelationships {
                        ab,
                        cd,
                        intersection,
                    }) => match (ab, cd) {
                        (TrueIntersectionPoint, TrueIntersectionPoint) => {
                            // Inside elbow
                            // No mitre limit needed
                            vec![intersection]
                        }
                        (FalseIntersectionPoint(AfterEnd), FalseIntersectionPoint(_)) => {
                            // Outside elbow
                            // Check for Mitre Limit
                            // TODO: Mitre limit code below is awful;
                            //       - Some values calculated here were
                            //         previously calculated in
                            //         [line_segment_intersection_with_parameters()]
                            //       - Various optimizations are possible;
                            //         Check against magnitude squared
                            //       - Magnitude function to be moved somewhere
                            //         else
                            //
                            fn magnitude<T>(coord: Coord<T>) -> T
                            where
                                T: CoordFloat,
                            {
                                (coord.x * coord.x + coord.y * coord.y).sqrt()
                            }
                            let mitre_limit_factor = T::from(2.0).unwrap();
                            let mitre_limit_distance = distance.abs() * mitre_limit_factor;
                            let elbow_length = magnitude(intersection - *b);
                            if elbow_length > mitre_limit_distance {
                                // Mitre Limited / Truncated Corner
                                let ab: Coord<T> = *b - *a;
                                let cd: Coord<T> = *d - *c;
                                vec![
                                    *b + ab / magnitude(ab) * mitre_limit_distance,
                                    *c - cd / magnitude(cd) * mitre_limit_distance,
                                ]
                            } else {
                                // Sharp Corner
                                vec![intersection]
                            }
                        }

                        _ => {
                            //Inside pinched elbow (forearm curled back through
                            //                      bicep 🙃)
                            //println!("CASE 3 - bridge");
                            vec![*b, *c]
                        }
                    },
                }
            },
        ));
        result.push(last_point);
        // TODO: there are more steps to this algorithm which are not yet
        //       implemented. See rfcs\2022-11-11-offset.md
        Some(result.into())
    }
}

impl<T> Offset<T> for MultiLineString<T>
where
    T: CoordFloat,
{
    fn offset(&self, distance: T) -> Option<Self> {
        self.iter().map(|item| item.offset(distance)).collect()
    }
}

// impl<T> Offset<T> for Polygon<T>
// where
//     T: CoordFloat,
// {
//     fn offset(&self, distance: T) -> Self {
//         // TODO: not finished yet... need to do interiors
//         // self.interiors()
//         // TODO: is winding order guaranteed?
//         self.exterior();
//         todo!("Not finished")
//     }
// }

#[cfg(test)]
mod test {

    use crate::{
        line_string,
        Coord,
        Line,
        //LineString,
        MultiLineString,
        Offset,
    };

    #[test]
    fn test_offset_line() {
        let input = Line::new(Coord { x: 1f64, y: 1f64 }, Coord { x: 1f64, y: 2f64 });
        let output_actual = input.offset(1.0);
        let output_expected = Some(Line::new(
            Coord { x: 2f64, y: 1f64 },
            Coord { x: 2f64, y: 2f64 },
        ));
        assert_eq!(output_actual, output_expected);
    }
    #[test]
    fn test_offset_line_negative() {
        let input = Line::new(Coord { x: 1f64, y: 1f64 }, Coord { x: 1f64, y: 2f64 });
        let output_actual = input.offset(-1.0);
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
        let output_actual = input.offset(1f64);
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
        let output_actual = input.offset(1f64);
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
        let output_actual = input.offset(1f64);
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
