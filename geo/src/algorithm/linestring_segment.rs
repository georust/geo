use crate::line_interpolate_point::LineInterpolatePoint;
use crate::{Coord, Densify, EuclideanLength, LineString, LinesIter, MultiLineString};

/// Segments a LineString into `n` equal length LineStrings as a MultiLineString.
/// `None` will be returned when `n` is equal to 0 or when a point
/// cannot be interpolated on a `Line` segment.
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
            // return Some(MultiLineString::new(vec![densified]))
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
            // first line string push the first coord immediately
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
        assert_eq!(segments.clone().0.len(), 5);

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
