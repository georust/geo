use super::line_intersection::FalseIntersectionPointType::AfterEnd;
use super::line_intersection::LineSegmentIntersectionType::{
    FalseIntersectionPoint, TrueIntersectionPoint,
};
use super::line_intersection::{
    line_segment_intersection_with_relationships, LineIntersectionWithParameterResult,
};
use super::slice_itertools::pairwise;

use crate::{
    Coord,
    CoordFloat,
    // Kernel,
    // Orientation,
    Line,
    LineString,
    MultiLineString,
    // Polygon,
};

/// # Offset Trait
///
/// Signed offset of Geometry assuming cartesian coordinate system.
///
/// This is a cheap offset algorithm that is suitable for flat coordinate systems
/// (or if your lat/lon data is near the equator)
///
/// My Priority for implementing the trait is as follows:
/// - [X] Line<impl CoordFloat>
/// - [X] LineString<impl CoordFloat>
/// - [X] MultiLineString<impl CoordFloat>
/// - ... maybe some closed shapes like triangle, polygon?
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
{
    /// Offset the edges of the geometry by `distance`, where `distance` may be
    /// negative.
    ///
    /// Negative `distance` values will offset the edges of the geometry to the
    /// left, when facing the direction of increasing coordinate index.
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
    /// let output_actual = input.offset(1f64);
    /// assert_eq!(output_actual, output_expected);
    /// ```
    fn offset(&self, distance: T) -> Self;
}

impl<T> Offset<T> for Line<T>
where
    T: CoordFloat,
{
    fn offset(&self, distance: T) -> Self {
        let delta = self.delta();
        let len = (delta.x * delta.x + delta.y * delta.y).sqrt();
        let delta = Coord {
            x: delta.y / len,
            y: -delta.x / len,
        };
        Line::new(self.start + delta * distance, self.end + delta * distance)
    }
}

impl<T> Offset<T> for LineString<T>
where
    T: CoordFloat,
{
    fn offset(&self, distance: T) -> Self {
        if self.0.len() < 2 {
            // TODO: How should it fail on invalid input?
            return self.clone();
        }

        let offset_segments: Vec<Line<T>> =
            self.lines().map(|item| item.offset(distance)).collect();

        if offset_segments.len() == 1 {
            return offset_segments[0].into();
        }
        let first_point = offset_segments.first().unwrap().start;
        let last_point = offset_segments.last().unwrap().end;

        let mut result = Vec::with_capacity(self.0.len());
        result.push(first_point);
        result.extend(pairwise(&offset_segments[..]).flat_map(
            |(Line { start: a, end: b }, Line { start: c, end: d })| {
                match line_segment_intersection_with_relationships(a, b, c, d) {
                    None => {
                        // TODO: this is the colinear case;
                        // we are potentially creating a redundant point in the
                        // output here. Colinear segments should maybe get
                        // removed before or after this algorithm
                        vec![*b]
                    }, 
                    Some(LineIntersectionWithParameterResult {
                        ab,
                        cd,
                        intersection,
                    }) => match (ab, cd) {
                        (TrueIntersectionPoint, TrueIntersectionPoint) => vec![intersection],
                        (FalseIntersectionPoint(AfterEnd), FalseIntersectionPoint(_)) => {
                            // TODO: Mitre limit logic goes here
                            vec![intersection]
                        }
                        _ => vec![*b, *c],
                    },
                }
            },
        ));
        result.push(last_point);
        // TODO: there are more steps to this algorithm which are not yet
        //       implemented. See rfcs\2022-11-11-offset.md
        result.into()
    }
}

impl<T> Offset<T> for MultiLineString<T>
where
    T: CoordFloat,
{
    fn offset(&self, distance: T) -> Self {
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
//         // TODO: is the winding order configurable?
//         self.exterior();
//         todo!("Not finished")
//     }
// }

#[cfg(test)]
mod test {

    use crate::{line_string, Coord, Line, LineString, MultiLineString, Offset};

    use super::super::slice_itertools::pairwise;

    #[test]
    fn test_offset_line() {
        let input = Line::new(Coord { x: 1f64, y: 1f64 }, Coord { x: 1f64, y: 2f64 });
        let actual_result = input.offset(1.0);
        assert_eq!(
            actual_result,
            Line::new(Coord { x: 2f64, y: 1f64 }, Coord { x: 2f64, y: 2f64 },)
        );
    }
    #[test]
    fn test_offset_line_negative() {
        let input = Line::new(Coord { x: 1f64, y: 1f64 }, Coord { x: 1f64, y: 2f64 });
        let output_actual = input.offset(-1.0);
        let output_expected = Line::new(Coord { x: 0f64, y: 1f64 }, Coord { x: 0f64, y: 2f64 });
        assert_eq!(output_actual, output_expected);
    }

    #[test]
    fn test_offset_line_string() {
        let input = line_string![
            Coord { x: 0f64, y: 0f64 },
            Coord { x: 0f64, y: 2f64 },
            Coord { x: 2f64, y: 2f64 },
        ];
        let output_expected = line_string![
            Coord { x: 1f64, y: 0f64 },
            Coord { x: 1f64, y: 1f64 },
            Coord { x: 2f64, y: 1f64 },
        ];
        let output_actual = input.offset(1f64);
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
        let output_expected = MultiLineString::new(vec![
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
        ]);
        let output_actual = input.offset(1f64);
        assert_eq!(output_actual, output_expected);
    }

    /// Function to draw test output to geogebra.org for inspection
    ///
    /// Paste the output  into the javascript console on geogebra.org to
    /// visualize the result
    ///
    /// The following snippet will extract existing (points and vectors) from geogebra:
    ///
    /// ```javascript
    /// console.log([
    ///     "line_string![",
    ///     ...ggbApplet.getAllObjectNames().filter(item=>item==item.toUpperCase()).map(name=>`    Coord{x:${ggbApplet.getXcoord(name)}f64, y:${ggbApplet.getYcoord(name)}f64},`),
    ///     "]",
    /// ].join("\n"))
    /// ```
    ///
    fn print_geogebra_draw_commands(input: &LineString, prefix: &str, r: u8, g: u8, b: u8) {
        let prefix_upper = prefix.to_uppercase();
        let prefix_lower = prefix.to_lowercase();
        input
            .coords()
            .enumerate()
            .for_each(|(index, Coord { x, y })| {
                println!(r#"ggbApplet.evalCommand("{prefix_upper}_{{{index}}} = ({x:?},{y:?})")"#)
            });
        let x: Vec<_> = input.coords().enumerate().collect();
        pairwise(&x[..]).for_each(|((a, _), (b, _))|{
            println!(r#"ggbApplet.evalCommand("{prefix_lower}_{{{a},{b}}} = Vector({prefix_upper}_{a},{prefix_upper}_{b})")"#);
            ()
        });
        let (dim_r, dim_g, dim_b) = (r / 2, g / 2, b / 2);
        println!(
            r#"ggbApplet.getAllObjectNames().filter(item=>item.startsWith("{prefix_upper}_")).forEach(item=>ggbApplet.setColor(item,{r},{g},{b}))"#
        );
        println!(
            r#"ggbApplet.getAllObjectNames().filter(item=>item.startsWith("{prefix_lower}_")).forEach(item=>ggbApplet.setColor(item,{dim_r},{dim_g},{dim_b}))"#
        );
    }

    #[test]
    fn test_offset_line_string_all_branch() {
        // attempts to hit all branches of the line extension / cropping test
        let input = line_string![
            Coord { x: 3f64, y: 2f64 },
            Coord {
                x: 2.740821628422733f64,
                y: 2.2582363315313816f64
            },
            Coord {
                x: 5.279039119779313f64,
                y: 2.516847170273373f64
            },
            Coord { x: 5f64, y: 2f64 },
            Coord {
                x: 3.2388869474813826f64,
                y: 4.489952088082639f64
            },
            Coord { x: 3f64, y: 4f64 },
            Coord { x: 4f64, y: 4f64 },
            Coord { x: 5.5f64, y: 4f64 },
            Coord {
                x: 5.240726402928647f64,
                y: 4.250497607765981f64
            },
        ];
        print_geogebra_draw_commands(&input, "I", 90, 90, 90);
        print_geogebra_draw_commands(&input.offset(-0.1f64), "L", 0, 200, 0);
        print_geogebra_draw_commands(&input.offset(0.1f64), "R", 200, 0, 0);

        // TODO: test always fails
        assert!(false);
    }
}
