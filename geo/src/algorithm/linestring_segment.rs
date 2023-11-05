use crate::line_interpolate_point::LineInterpolatePoint;
use crate::{Coord, Densify, EuclideanLength, LineString, LinesIter, MultiLineString};

/// Segments a LineString into `n` equal length LineStrings as a MultiLineString.
/// `None` will be returned when `n` is equal to 0 or when a point
/// cannot be interpolated on a `Line` segment.
///
///
/// # Examples
/// ```
/// use geo::{LineString, MultiLineString, LineStringSegmentize};
/// // Create a simple line string
/// let lns: LineString<f64> = vec![[0.0, 0.0], [1.0, 2.0], [3.0, 6.0]].into();
/// // Segment it into n LineStrings inside of a MultiLineString
/// let n = 6;
/// let segmentized = lns.line_segmentize(n).unwrap();
/// // Compare the number of elements
/// assert_eq!(n, segmentized.0.len());
///```
pub trait LineStringSegmentize {
    fn line_segmentize(&self, n: usize) -> Option<MultiLineString>;
}

impl LineStringSegmentize for LineString {
    fn line_segmentize(&self, n: usize) -> Option<MultiLineString> {
        // Return None if n is 0 or the maximum usize
        if (n == usize::MIN) || (n == usize::MAX) {
            return None;
        } else if n == 1 {
            let mlns = MultiLineString::from(self.clone());
            return Some(mlns);
        }

        // Vec to allocate the  new LineString segments Coord Vec
        // will be iterated over at end to create new vecs
        let mut res_coords: Vec<Vec<Coord>> = Vec::with_capacity(n);

        // calculate total length to track cumulative against
        let total_length = self.euclidean_length().abs();

        // tracks total length
        let mut cum_length = 0_f64;

        // calculate the target fraction for the first iteration
        // fraction will change based on each iteration
        let segment_prop = (1_f64) / (n as f64);
        let segment_length = total_length * segment_prop;

        // densify the LineString so that each `Line` segment is not longer
        // than the segment length ensuring that we will never partition one
        // Line more than once.
        // in the case of super small distances floating point errors can arise
        // the solution is to subtract by f64::EPSILON for these edge cases.
        let densified = self.densify(segment_length - f64::EPSILON);

        // if the densified line is exactly equal to the number of requested
        // segments, return early. This will happen when a LineString has
        // exactly 2 coordinates
        if densified.lines().count() == n {
            let linestrings = densified
                .lines()
                .map(LineString::from)
                .collect::<Vec<LineString>>();

            return Some(MultiLineString::new(linestrings));
        };

        // count the number of lines that will be iterated through
        let n_lines = densified.lines().count();

        let lns = densified.lines_iter();
        // instantiate the first Vec<Coord>
        let mut ln_vec: Vec<Coord> = Vec::new();

        // iterate through each line segment in the LineString
        for (i, segment) in lns.enumerate() {
            // All iterations only keep track of the second coordinate
            // in the Line. We need to push the first coordinate in the
            // first line string to ensure the linestring starts at the
            // correct place
            if i == 0 {
                ln_vec.push(segment.start)
            }

            let length = segment.euclidean_length().abs();

            // update cumulative length
            cum_length += length;

            if (cum_length >= segment_length) && (i != (n_lines - 1)) {
                let remainder = cum_length - segment_length;
                // if we get None, we exit the function and return None
                let endpoint = segment.line_interpolate_point((length - remainder) / length)?;

                // add final coord to ln_vec
                ln_vec.push(endpoint.into());

                // now we drain all elements from the vector into an iterator
                // this will be collected into a vector to be pushed into the
                // results coord vec of vec
                let to_push = ln_vec.drain(..);

                // now collect & push this vector into the results vector
                res_coords.push(to_push.collect::<Vec<Coord>>());

                // now add the last endpoint as the first coord
                // and the endpoint of the linesegment as well only
                if i != n_lines {
                    ln_vec.push(endpoint.into());
                }

                cum_length = remainder;
            }

            // push the end coordinate into the Vec<Coord> to continue
            // building the linestring
            ln_vec.push(segment.end);
        }

        // push the last linestring vector which isn't done by the for loop
        res_coords.push(ln_vec);

        // collect the coords into vectors of LineStrings so we can createa
        // a multi linestring
        let res_lines = res_coords
            .into_iter()
            .map(LineString::new)
            .collect::<Vec<LineString>>();

        Some(MultiLineString::new(res_lines))
    }
}

#[cfg(test)]
mod test {
    use approx::RelativeEq;

    use super::*;
    use crate::{EuclideanLength, LineString};

    #[test]
    fn n_elems_bug() {
        // Test for an edge case that seems to fail:
        // https://github.com/georust/geo/issues/1075
        // https://github.com/JosiahParry/rsgeo/issues/28

        let linestring: LineString = vec![
            [324957.69921197, 673670.123131518],
            [324957.873557727, 673680.139281405],
            [324959.863123514, 673686.784106964],
            [324961.852683597, 673693.428933452],
            [324963.822867622, 673698.960855279],
            [324969.636546456, 673709.992098018],
            [324976.718443977, 673722.114520549],
            [324996.443964294, 673742.922904206],
        ]
        .into();
        let segments = linestring.line_segmentize(2).unwrap();
        assert_eq!(segments.0.len(), 2);
        let segments = linestring.line_segmentize(3).unwrap();
        assert_eq!(segments.0.len(), 3);
        let segments = linestring.line_segmentize(4).unwrap();
        assert_eq!(segments.0.len(), 4);

        assert_eq!(segments.euclidean_length(), linestring.euclidean_length());
    }

    #[test]
    fn long_end_segment() {
        let linestring: LineString = vec![
            [325581.792390628, 674398.495901267],
            [325585.576868499, 674400.657039341],
            [325589.966469742, 674401.694493658],
            [325593.750940609, 674403.855638851],
            [325599.389217394, 674404.871546368],
            [325604.422360924, 674407.011146146],
            [325665.309662534, 674424.885671739],
        ]
        .into();

        let segments = linestring.line_segmentize(5).unwrap();
        assert_eq!(segments.0.len(), 5);
        assert_relative_eq!(
            linestring.euclidean_length(),
            segments.euclidean_length(),
            epsilon = f64::EPSILON
        );
    }

    #[test]
    fn two_coords() {
        let linestring: LineString = vec![[0.0, 0.0], [0.0, 1.0]].into();

        let segments = linestring.line_segmentize(5).unwrap();
        assert_eq!(segments.0.len(), 5);
        assert_relative_eq!(
            linestring.euclidean_length(),
            segments.euclidean_length(),
            epsilon = f64::EPSILON
        );
    }

    #[test]
    fn long_middle_segments() {
        let linestring: LineString = vec![
            [325403.816883668, 673966.295402012],
            [325410.280933752, 673942.805501254],
            [325410.280933752, 673942.805501254],
            [325439.782082601, 673951.201057316],
            [325439.782082601, 673951.201057316],
            [325446.064640793, 673953.318876004],
            [325446.064640793, 673953.318876004],
            [325466.14184472, 673958.537886844],
            [325466.14184472, 673958.537886844],
            [325471.799973648, 673960.666539074],
            [325471.799973648, 673960.666539074],
            [325518.255916084, 673974.335722824],
            [325518.255916084, 673974.335722824],
            [325517.669972133, 673976.572326305],
            [325517.669972133, 673976.572326305],
            [325517.084028835, 673978.808929878],
            [325517.084028835, 673978.808929878],
            [325515.306972763, 673984.405833764],
            [325515.306972763, 673984.405833764],
            [325513.549152184, 673991.115645844],
            [325513.549152184, 673991.115645844],
            [325511.772106396, 673996.712551354],
        ]
        .into();

        let segments = linestring.line_segmentize(5).unwrap();
        assert_eq!(segments.0.len(), 5);

        assert_relative_eq!(
            linestring.euclidean_length(),
            segments.euclidean_length(),
            epsilon = f64::EPSILON
        );
    }

    #[test]
    // that 0 returns None and that usize::MAX returns None
    fn n_is_zero() {
        let linestring: LineString = vec![[-1.0, 0.0], [0.5, 1.0], [1.0, 2.0]].into();
        let segments = linestring.line_segmentize(0);
        assert!(segments.is_none())
    }

    #[test]
    fn n_is_max() {
        let linestring: LineString = vec![[-1.0, 0.0], [0.5, 1.0], [1.0, 2.0]].into();
        let segments = linestring.line_segmentize(usize::MAX);
        assert!(segments.is_none())
    }

    #[test]
    fn n_greater_than_lines() {
        let linestring: LineString = vec![[-1.0, 0.0], [0.5, 1.0], [1.0, 2.0]].into();
        let segments = linestring.line_segmentize(5).unwrap();

        // assert that there are n linestring segments
        assert_eq!(segments.0.len(), 5);

        // assert that the lines are equal length
        let lens = segments
            .into_iter()
            .map(|x| x.euclidean_length())
            .collect::<Vec<f64>>();

        let first = lens[0];

        assert!(lens
            .iter()
            .all(|x| first.relative_eq(x, f64::EPSILON, 1e-10)))
    }

    #[test]
    // test the cumulative length is the same
    fn cumul_length() {
        let linestring: LineString = vec![[0.0, 0.0], [1.0, 1.0], [1.0, 2.0], [3.0, 3.0]].into();
        let segments = linestring.line_segmentize(2).unwrap();

        assert_relative_eq!(
            linestring.euclidean_length(),
            segments.euclidean_length(),
            epsilon = f64::EPSILON
        )
    }

    #[test]
    fn n_elems() {
        let linestring: LineString = vec![[0.0, 0.0], [1.0, 1.0], [1.0, 2.0], [3.0, 3.0]].into();
        let segments = linestring.line_segmentize(2).unwrap();
        assert_eq!(segments.0.len(), 2)
    }

    #[test]
    fn tiny_distances() {
        // this test is to ensure that at super small distances
        // the number of units is still the specified one.
        let linestring: LineString = vec![
            [-3.19416, 55.95524],
            [-3.19352, 55.95535],
            [-3.19288, 55.95546],
        ]
        .into();

        let n = 8;
        let segments = linestring.line_segmentize(n).unwrap();
        assert_eq!(segments.0.len(), n)
    }
}
