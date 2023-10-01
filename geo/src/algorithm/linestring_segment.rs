use crate::line_interpolate_point::LineInterpolatePoint;
use crate::{Coord, Densify, EuclideanLength, LineString, LinesIter, MultiLineString};

/// Segments a LineString into `n` equal length LineStrings as a MultiLineString.
/// `None` will be returned when `n` is equal to 0 or when a point
/// cannot be interpolated on a `Line` segment.
///
///
/// # Examples
/// ```
/// use geo::{LineString, MultiLineString, LineStringSegmentize, Coord};
/// // Create a simple line string
/// let lns: LineString<f64> = vec![[0.0, 0.0], [1.0, 2.0], [3.0, 6.0]].into();
/// // Segment it into 6 LineStrings inside of a MultiLineString
/// let segmentized = lns.line_segmentize(6).unwrap();
///
/// // Recreate the MultiLineString from scratch
/// // this is the inner vector used to create the MultiLineString
/// let all_lines = vec![
///     LineString::new(vec![Coord { x: 0.0, y: 0.0 }, Coord { x: 0.5, y: 1.0 }]),
///     LineString::new(vec![Coord { x: 0.5, y: 1.0 }, Coord { x: 1.0, y: 2.0 }]),
///     LineString::new(vec![Coord { x: 1.0, y: 2.0 }, Coord { x: 1.5, y: 3.0 }]),
///     LineString::new(vec![Coord { x: 1.5, y: 3.0 }, Coord { x: 2.0, y: 4.0 }]),
///     LineString::new(vec![Coord { x: 2.0, y: 4.0 }, Coord { x: 2.5, y: 5.0 }]),
///     LineString::new(vec![Coord { x: 2.5, y: 5.0 }, Coord { x: 3.0, y: 6.0 }])
///     ];
///
/// // Create the MultiLineString
/// let mlns = MultiLineString::new(all_lines);
///
/// // Compare the two
/// assert_eq!(mlns, segmentized);
///```
pub trait LineStringSegmentize {
    fn line_segmentize(&self, n: usize) -> Option<MultiLineString>;
}

impl LineStringSegmentize for LineString {
    fn line_segmentize(&self, n: usize) -> Option<MultiLineString> {
        let n_lines = self.lines().count();

        // Return None if n is 0 or the maximum usize
        if (n == usize::MIN) || (n == usize::MAX) {
            return None;
        } else if n > n_lines {
            let total_len = self.euclidean_length();
            let densified = self.densify(total_len / (n as f64));
            return densified.line_segmentize(n);
        } else if n_lines == n {
            // if the number of line segments equals n then return the
            // lines as LineStrings
            let lns = self
                .lines_iter()
                .map(LineString::from)
                .collect::<Vec<LineString>>();

            return Some(MultiLineString::new(lns));
        } else if n == 1 {
            let mlns = MultiLineString::from(self.clone());
            return Some(mlns);
        }

        // Convert X into an iterator of `Lines`
        let lns = self.lines_iter().peekable();

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

        // instantiate the first Vec<Coord>
        let mut ln_vec: Vec<Coord> = Vec::new();

        // iterate through each line segment in the LineString
        for (i, segment) in lns.enumerate() {
            // All iterations only keep track of the second coordinate
            // in the Line. We need to push the first coordinate in the
            // first line string to ensure the linestring starts at the
            // correct place`
            if i == 0 {
                ln_vec.push(segment.start)
            }

            let length = segment.euclidean_length().abs();

            // update cumulative length
            cum_length += length;

            // if the cumulative lenght is greater than or equal to the segment length we
            // must cut the line in the middle and push the coords
            if (cum_length >= segment_length) || (cum_length == segment_length) {
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

        // if the for loop & if statement pushed the last coord & vec we do not
        // need to push again outside of the loop. This handles the (majority)
        // case where the last iteration does not push into the linestring vec
        if ln_vec.len() < n {
            // push the last linestring vector which isn't done by the for loop
            res_coords.push(ln_vec);
        }

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
    // identical line_iter to original
    fn line_iter() {
        let linestring: LineString = vec![[-1.0, 0.0], [0.5, 1.0], [1.0, 2.0], [1.0, 3.0]].into();

        let segments = linestring.line_segmentize(3).unwrap();

        assert!(linestring.lines_iter().eq(segments.lines_iter()))
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
}
