use crate::CoordFloat;
use crate::EuclideanLength;
use crate::Line;
use crate::LineString;

/// The result of the [measure_line_string] function
#[derive(PartialEq, Debug)]
pub struct LineStringMeasurements<Scalar> {
    /// Total length of the [LineString]
    pub length_total: Scalar,
    /// The length of each segment ([Line]) in the [LineString]
    pub length_segments: Vec<Scalar>,
}

/// Measure a [LineString] and return [`Option<LineStringMeasurements>`](LineStringMeasurements);
/// The result contains both the `total_length` and the `length_segments` (the length
/// of each segment or [Line])
/// 
/// Returns [None] 
/// - if the [LineString] has less than two [coords](crate::Coord)
/// - if total_length is not finite
pub fn measure_line_string<Scalar>(
    line_string: &LineString<Scalar>,
) -> Option<LineStringMeasurements<Scalar>>
where
    Scalar: CoordFloat,
    Line<Scalar>: EuclideanLength<Scalar>,
{
    if line_string.0.len() < 2 {
        return None;
    }
    let result = line_string.lines().fold(
        LineStringMeasurements {
            length_total: Scalar::zero(),
            length_segments: Vec::new(),
        },
        |LineStringMeasurements {
             length_total,
             mut length_segments,
         },
         current| {
            let segment_length = current.euclidean_length();
            length_segments.push(segment_length);
            LineStringMeasurements {
                length_total: length_total + segment_length,
                length_segments,
            }
        },
    );
    if !result.length_total.is_finite() {
        None
    } else {
        Some(result)
    }
}

#[cfg(test)]
mod test {

    use geo_types::{line_string, LineString};

    use super::{measure_line_string, LineStringMeasurements};

    #[test]
    fn measure_line_string_typical() {
        let line_string: LineString<f32> = line_string![
            (x:0.0, y:0.0),
            (x:1.0, y:0.0),
            (x:1.0, y:1.0),
            (x:2.0, y:1.0),
        ];
        let LineStringMeasurements {
            length_total,
            length_segments,
        } = measure_line_string(&line_string).unwrap();
        assert_eq!(length_total, 3.0);
        assert_eq!(length_segments, vec![1.0_f32, 1.0_f32, 1.0_f32]);
    }

    #[test]
    fn measure_line_string_malformed_zero() {
        let line_string: LineString<f32> = line_string![];
        assert!(measure_line_string(&line_string).is_none());
    }

    #[test]
    fn measure_line_string_malformed_one() {
        let line_string: LineString<f32> = line_string![
            (x:0.0, y:0.0),
        ];
        assert!(measure_line_string(&line_string).is_none());
    }

    #[test]
    fn measure_line_string_malformed_nan() {
        let line_string: LineString<f32> = line_string![
            (x:0.0, y:0.0),
            (x:1.0, y:f32::NAN),
        ];
        assert!(measure_line_string(&line_string).is_none());
    }

    #[test]
    fn measure_line_string_malformed_nan2() {
        let line_string: LineString<f32> = line_string![
            (x:0.0, y:0.0),
            (x:1.0, y:f32::NAN),
            (x:1.0, y:1.0),
            (x:2.0, y:1.0),
        ];
        assert!(measure_line_string(&line_string).is_none());
    }

    #[test]
    fn measure_line_string_malformed_inf() {
        let line_string: LineString<f32> = line_string![
            (x:0.0, y:0.0),
            (x:1.0, y:f32::INFINITY),
        ];
        assert!(measure_line_string(&line_string).is_none());
    }
}
