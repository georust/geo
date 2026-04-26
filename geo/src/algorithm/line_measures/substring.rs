//! Extract a sub-linestring of a [`Line`] or [`LineString`] between two
//! distance-along-line (or ratio-along-line) values.
//!
//! See [`Substring`] for the metric-generic extension trait, and
//! [`SubstringableLine`] for the line-side trait. Both are wired up so
//! you can call from either receiver:
//!
//! ```ignore
//! line_string.substring_by_distance(&Euclidean, 5.0, 10.0)
//! Euclidean.substring_by_distance(&line_string, 5.0, 10.0)
//! ```
//!
//! This is the analogue of PostGIS `ST_Line_Substring` and GEOS
//! `LengthIndexedLine::extractLine`. Unlike composing `line_locate_point`
//! and `line_interpolate_point`, this preserves the original interior
//! vertices of the parent linestring between the two endpoints.

use super::{InterpolatePoint, Length};
use crate::algorithm::vector_ops::Vector2DOps;
use geo_types::{Coord, CoordFloat, Line, LineString};

/// Errors returned by substring extraction. See [`SubstringableLine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubstringError {
    /// Input line string contains no coordinates.
    EmptyLineString,
    /// Input line string contains a single vertex (no segments).
    SingleVertex,
    /// Input contains a non-finite coordinate (NaN or infinity).
    NonFiniteCoordinate,
    /// Range invalid: `start > end`, or `start`/`end` is not finite.
    InvalidRange,
}

impl std::fmt::Display for SubstringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubstringError::EmptyLineString => {
                write!(f, "LineString contains no coordinates")
            }
            SubstringError::SingleVertex => {
                write!(f, "LineString contains only a single vertex")
            }
            SubstringError::NonFiniteCoordinate => {
                write!(f, "input contains a non-finite coordinate")
            }
            SubstringError::InvalidRange => {
                write!(
                    f,
                    "invalid range: start must be <= end and both must be finite"
                )
            }
        }
    }
}

impl std::error::Error for SubstringError {}

/// A linear geometry from which a sub-linestring can be extracted between
/// two distance-along-line (or ratio-along-line) bounds.
///
/// Interior vertices of the parent geometry that fall strictly between
/// the two bounds are preserved in the output.
pub trait SubstringableLine<F: CoordFloat> {
    /// The output type of substring extraction. Both `Line` and
    /// `LineString` impls return `Result<LineString<F>, SubstringError>`
    /// for a uniform call site.
    type Output;

    /// Extract a sub-linestring between `start` and `end`, both expressed
    /// as a distance from the start of the line in the units of
    /// `metric_space`.
    ///
    /// Out-of-range values are clamped to `[0, total_length]`. NaN, or
    /// `start > end`, returns `Err(SubstringError::InvalidRange)`.
    ///
    /// # Example
    ///
    /// ```
    /// use geo::{Euclidean, SubstringableLine, wkt};
    ///
    /// let line_string = wkt!(LINESTRING(0.0 0.0, 10.0 0.0, 20.0 0.0, 30.0 0.0));
    /// let result = line_string
    ///     .substring_by_distance(&Euclidean, 5.0, 25.0)
    ///     .unwrap();
    /// // Interior parent vertices at x=10 and x=20 are preserved.
    /// assert_eq!(result, wkt!(LINESTRING(5.0 0.0, 10.0 0.0, 20.0 0.0, 25.0 0.0)));
    /// ```
    fn substring_by_distance<M: InterpolatePoint<F> + Length<F>>(
        &self,
        metric_space: &M,
        start: F,
        end: F,
    ) -> Self::Output;

    /// Extract a sub-linestring between `start` and `end`, both expressed
    /// as a fraction of the line's total length in `[0, 1]`.
    ///
    /// Out-of-range values are clamped to `[0, 1]`. NaN, or
    /// `start > end`, returns `Err(SubstringError::InvalidRange)`.
    ///
    /// # Example
    ///
    /// ```
    /// use geo::{Euclidean, SubstringableLine, wkt};
    ///
    /// let line_string = wkt!(LINESTRING(0.0 0.0, 10.0 0.0, 20.0 0.0, 30.0 0.0));
    /// let first_half = line_string
    ///     .substring_by_ratio(&Euclidean, 0.0, 0.5)
    ///     .unwrap();
    /// assert_eq!(first_half, wkt!(LINESTRING(0.0 0.0, 10.0 0.0, 15.0 0.0)));
    /// ```
    fn substring_by_ratio<M: InterpolatePoint<F> + Length<F>>(
        &self,
        metric_space: &M,
        start: F,
        end: F,
    ) -> Self::Output;
}

/// Extension trait providing substring extraction on any metric space
/// implementing [`InterpolatePoint`] and [`Length`].
///
/// This is the metric-side counterpart to [`SubstringableLine`]; both
/// expose the same operations and you can use whichever is more
/// convenient at the call site.
pub trait Substring<F: CoordFloat>: InterpolatePoint<F> + Length<F> + Sized {
    /// Distance-bounded substring; see
    /// [`SubstringableLine::substring_by_distance`].
    ///
    /// # NOTES
    /// On [LineString], when start == end we deliberately produce a two-coord
    /// [LineString] with both coords equal, rather than an `Err`. The result
    /// is a degenerate linestring at a user-specified
    /// location
    fn substring_by_distance<L: SubstringableLine<F>>(
        &self,
        line: &L,
        start: F,
        end: F,
    ) -> L::Output {
        line.substring_by_distance(self, start, end)
    }

    /// Ratio-bounded substring; see
    /// [`SubstringableLine::substring_by_ratio`].
    fn substring_by_ratio<L: SubstringableLine<F>>(&self, line: &L, start: F, end: F) -> L::Output {
        line.substring_by_ratio(self, start, end)
    }
}

impl<F: CoordFloat, M: InterpolatePoint<F> + Length<F>> Substring<F> for M {}

impl<F: CoordFloat> SubstringableLine<F> for Line<F> {
    type Output = Result<LineString<F>, SubstringError>;

    fn substring_by_distance<M: InterpolatePoint<F> + Length<F>>(
        &self,
        metric_space: &M,
        start: F,
        end: F,
    ) -> Self::Output {
        if !self.start.is_finite() || !self.end.is_finite() {
            return Err(SubstringError::NonFiniteCoordinate);
        }
        if start.is_nan() || end.is_nan() || start > end {
            return Err(SubstringError::InvalidRange);
        }

        let total = metric_space.length(self);
        let start = start.max(F::zero()).min(total);
        let end = end.max(F::zero()).min(total);

        let start_pt =
            metric_space.point_at_distance_between(self.start_point(), self.end_point(), start);
        let end_pt =
            metric_space.point_at_distance_between(self.start_point(), self.end_point(), end);

        Ok(LineString::from(vec![start_pt.0, end_pt.0]))
    }

    fn substring_by_ratio<M: InterpolatePoint<F> + Length<F>>(
        &self,
        metric_space: &M,
        start: F,
        end: F,
    ) -> Self::Output {
        let total = metric_space.length(self);
        self.substring_by_distance(metric_space, start * total, end * total)
    }
}

impl<F: CoordFloat> SubstringableLine<F> for LineString<F> {
    type Output = Result<LineString<F>, SubstringError>;

    fn substring_by_distance<M: InterpolatePoint<F> + Length<F>>(
        &self,
        metric_space: &M,
        start: F,
        end: F,
    ) -> Self::Output {
        if self.0.is_empty() {
            return Err(SubstringError::EmptyLineString);
        }
        if self.0.len() == 1 {
            return Err(SubstringError::SingleVertex);
        }
        for c in &self.0 {
            if !c.x.is_finite() || !c.y.is_finite() {
                return Err(SubstringError::NonFiniteCoordinate);
            }
        }
        if start.is_nan() || end.is_nan() || start > end {
            return Err(SubstringError::InvalidRange);
        }

        let total = metric_space.length(self);
        let start = start.max(F::zero()).min(total);
        let end = end.max(F::zero()).min(total);

        // NOTE: when start == end we deliberately produce a two-coord
        // LineString with both coords equal, rather than an Err. The result
        // is a degenerate linestring at a user-specified
        // location, not a default zero. The rationale is that e.g. callers iterating duplicate
        // mileposts shouldn't have to special-case this. If something like
        // Err(SubstringError::InvalidRange) for start == end is preferred, the
        // change is localised to this function plus the
        // `substring_linestring_zero_length_range` test.
        let mut coords: Vec<Coord<F>> = Vec::with_capacity(self.0.len() + 1);
        let mut cum = F::zero();
        let mut emitted_start = false;
        // When the bound lands exactly on a parent vertex, prefer
        // the original coord over re-interpolating: it's exact, and
        // it avoids floating-point drift in non-Euclidean metrics
        // where `point_at_distance_between(a, b, len(a, b)) != b`.
        let clamp = |segment: &Line<F>, seg_len: F, distance_along: F| -> Coord<F> {
            if distance_along <= F::zero() {
                segment.start
            } else if distance_along >= seg_len {
                segment.end
            } else {
                metric_space
                    .point_at_distance_between(
                        segment.start_point(),
                        segment.end_point(),
                        distance_along,
                    )
                    .0
            }
        };

        for segment in self.lines() {
            let seg_len = metric_space.length(&segment);
            let seg_end = cum + seg_len;

            if !emitted_start && seg_end >= start {
                let local = start - cum;
                coords.push(clamp(&segment, seg_len, local));
                emitted_start = true;
                // If start landed exactly on segment.end, the next interior-vertex
                // emission below would duplicate it. Skip the rest of this iteration;
                // the next iteration handles end-emission and beyond.
                if local >= seg_len {
                    cum = seg_end;
                    continue;
                }
            }

            // Emit the vertex at seg.end if it falls strictly between start and end.
            // (i.e. the parent's interior vertex is preserved.)
            if emitted_start && seg_end < end {
                coords.push(segment.end);
            }

            if seg_end >= end {
                let local = end - cum;
                coords.push(clamp(&segment, seg_len, local));
                break;
            }

            cum = seg_end;
        }

        Ok(LineString::new(coords))
    }

    fn substring_by_ratio<M: InterpolatePoint<F> + Length<F>>(
        &self,
        metric_space: &M,
        start: F,
        end: F,
    ) -> Self::Output {
        let total = metric_space.length(self);
        self.substring_by_distance(metric_space, start * total, end * total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Euclidean;
    use crate::wkt;
    use geo_types::{Line, Point};

    #[test]
    fn substring_error_is_debug_clone_eq() {
        let e = SubstringError::EmptyLineString;
        let cloned = e.clone();
        assert_eq!(e, cloned);
        let _: &dyn std::error::Error = &e;
        assert_eq!(format!("{e}"), "LineString contains no coordinates");
    }

    #[test]
    fn substring_traits_compile() {
        // Generic-bound smoke test: ensures the trait shapes are usable
        // from generic code. Behaviour is exercised by the impl-specific
        // tests below.
        fn _accepts_substringable<L: SubstringableLine<f64>>(_: &L) {}
        fn _accepts_substring<S: Substring<f64>>(_: &S) {}
    }

    #[test]
    fn substring_reexported_from_crate_root() {
        // This test fails to compile if the re-exports in
        // algorithm/mod.rs aren't wired correctly.
        use crate::{Substring, SubstringError, SubstringableLine};
        fn _t<S: Substring<f64>, L: SubstringableLine<f64>>(_s: &S, _l: &L) {}
        let _e: SubstringError = SubstringError::EmptyLineString;
    }

    #[test]
    fn substring_line_full_range() {
        let line = wkt!(LINE(0.0 0.0,10.0 0.0));
        let result = line.substring_by_distance(&Euclidean, 0.0, 10.0).unwrap();
        let expected = wkt!(LINESTRING(0.0 0.0, 10.0 0.0));
        assert_eq!(result, expected);
    }

    #[test]
    fn substring_line_inner_range() {
        let line = wkt!(LINE(0.0 0.0,10.0 0.0));
        let result = line.substring_by_distance(&Euclidean, 2.0, 7.0).unwrap();
        let expected = wkt!(LINESTRING(2.0 0.0, 7.0 0.0));
        assert_eq!(result, expected);
    }

    #[test]
    fn substring_line_clamps_out_of_range() {
        let line = wkt!(LINE(0.0 0.0,10.0 0.0));
        let result = line.substring_by_distance(&Euclidean, -5.0, 100.0).unwrap();
        let expected = wkt!(LINESTRING(0.0 0.0, 10.0 0.0));
        assert_eq!(result, expected);
    }

    #[test]
    fn substring_line_zero_length_range() {
        let line = wkt!(LINE(0.0 0.0,10.0 0.0));
        let result = line.substring_by_distance(&Euclidean, 4.0, 4.0).unwrap();
        let expected = wkt!(LINESTRING(4.0 0.0, 4.0 0.0));
        assert_eq!(result, expected);
    }

    #[test]
    fn substring_line_invalid_range_start_gt_end() {
        let line = wkt!(LINE(0.0 0.0,10.0 0.0));
        assert_eq!(
            line.substring_by_distance(&Euclidean, 7.0, 2.0),
            Err(SubstringError::InvalidRange)
        );
    }

    #[test]
    fn substring_line_invalid_range_nan_bound() {
        let line = wkt!(LINE(0.0 0.0,10.0 0.0));
        assert_eq!(
            line.substring_by_distance(&Euclidean, f64::NAN, 5.0),
            Err(SubstringError::InvalidRange)
        );
    }

    #[test]
    fn substring_line_non_finite_coord() {
        let line = Line::new(Point::new(0.0, 0.0), Point::new(f64::NAN, 0.0));
        assert_eq!(
            line.substring_by_distance(&Euclidean, 0.0, 5.0),
            Err(SubstringError::NonFiniteCoordinate)
        );
    }

    #[test]
    fn substring_line_by_ratio_matches_distance() {
        let line = wkt!(LINE(0.0 0.0,10.0 0.0));
        let by_ratio = line.substring_by_ratio(&Euclidean, 0.2, 0.7).unwrap();
        let by_distance = line.substring_by_distance(&Euclidean, 2.0, 7.0).unwrap();
        assert_eq!(by_ratio, by_distance);
    }

    #[test]
    fn substring_linestring_preserves_interior_vertices() {
        // Total length 30; bounds 5..25 should keep both interior vertices (10, 0) and (20, 0).
        let line_string = wkt!(LINESTRING(0.0 0.0, 10.0 0.0, 20.0 0.0, 30.0 0.0));
        let result = line_string
            .substring_by_distance(&Euclidean, 5.0, 25.0)
            .unwrap();
        let expected = wkt!(LINESTRING(5.0 0.0, 10.0 0.0, 20.0 0.0, 25.0 0.0));
        assert_eq!(result, expected);
    }

    #[test]
    fn substring_linestring_within_single_segment() {
        let line_string = wkt!(LINESTRING(0.0 0.0, 10.0 0.0, 20.0 0.0));
        let result = line_string
            .substring_by_distance(&Euclidean, 2.0, 7.0)
            .unwrap();
        let expected = wkt!(LINESTRING(2.0 0.0, 7.0 0.0));
        assert_eq!(result, expected);
    }

    #[test]
    fn substring_linestring_full_range_round_trip() {
        let line_string = wkt!(LINESTRING(0.0 0.0, 10.0 0.0, 20.0 0.0, 30.0 0.0));
        let result = line_string
            .substring_by_distance(&Euclidean, 0.0, 30.0)
            .unwrap();
        assert_eq!(result, line_string);
    }

    #[test]
    fn substring_linestring_clamps_out_of_range() {
        let line_string = wkt!(LINESTRING(0.0 0.0, 10.0 0.0, 20.0 0.0));
        let result = line_string
            .substring_by_distance(&Euclidean, -5.0, 100.0)
            .unwrap();
        assert_eq!(result, line_string);
    }

    #[test]
    fn substring_linestring_zero_length_range() {
        let line_string = wkt!(LINESTRING(0.0 0.0, 10.0 0.0, 20.0 0.0));
        let result = line_string
            .substring_by_distance(&Euclidean, 7.0, 7.0)
            .unwrap();
        let expected = wkt!(LINESTRING(7.0 0.0, 7.0 0.0));
        assert_eq!(result, expected);
    }

    #[test]
    fn substring_linestring_empty_input() {
        let line_string: LineString<f64> = LineString::new(vec![]);
        assert_eq!(
            line_string.substring_by_distance(&Euclidean, 0.0, 1.0),
            Err(SubstringError::EmptyLineString)
        );
    }

    #[test]
    fn substring_linestring_single_vertex() {
        let line_string = LineString::from(vec![(1.0, 2.0)]);
        assert_eq!(
            line_string.substring_by_distance(&Euclidean, 0.0, 1.0),
            Err(SubstringError::SingleVertex)
        );
    }

    #[test]
    fn substring_linestring_invalid_range() {
        let line_string = wkt!(LINESTRING(0.0 0.0, 10.0 0.0));
        assert_eq!(
            line_string.substring_by_distance(&Euclidean, 7.0, 2.0),
            Err(SubstringError::InvalidRange)
        );
    }

    #[test]
    fn substring_linestring_nan_coord() {
        let line_string = LineString::from(vec![(0.0, 0.0), (f64::NAN, 0.0), (10.0, 0.0)]);
        assert_eq!(
            line_string.substring_by_distance(&Euclidean, 0.0, 5.0),
            Err(SubstringError::NonFiniteCoordinate)
        );
    }

    #[test]
    fn substring_linestring_by_ratio() {
        let line_string = wkt!(LINESTRING(0.0 0.0, 10.0 0.0, 20.0 0.0, 30.0 0.0));
        let by_ratio = line_string
            .substring_by_ratio(&Euclidean, 0.0, 0.5)
            .unwrap();
        let expected = wkt!(LINESTRING(0.0 0.0, 10.0 0.0, 15.0 0.0));
        assert_eq!(by_ratio, expected);
    }

    #[test]
    fn substring_linestring_start_on_interior_vertex() {
        let ls = wkt!(LINESTRING(0.0 0.0, 10.0 0.0, 20.0 0.0, 30.0 0.0));
        // start lands exactly on interior vertex (10, 0).
        assert_eq!(
            ls.substring_by_distance(&Euclidean, 10.0, 15.0).unwrap(),
            wkt!(LINESTRING(10.0 0.0, 15.0 0.0))
        );
        // start on interior vertex; end on the very next interior vertex.
        assert_eq!(
            ls.substring_by_distance(&Euclidean, 10.0, 20.0).unwrap(),
            wkt!(LINESTRING(10.0 0.0, 20.0 0.0))
        );
        // start on the first interior vertex; end further along.
        assert_eq!(
            ls.substring_by_distance(&Euclidean, 10.0, 25.0).unwrap(),
            wkt!(LINESTRING(10.0 0.0, 20.0 0.0, 25.0 0.0))
        );
        // start on the second interior vertex.
        assert_eq!(
            ls.substring_by_distance(&Euclidean, 20.0, 25.0).unwrap(),
            wkt!(LINESTRING(20.0 0.0, 25.0 0.0))
        );
    }

    #[test]
    fn substring_linestring_end_on_interior_vertex() {
        // Pin the symmetric case (which currently passes -- confirm).
        let ls = wkt!(LINESTRING(0.0 0.0, 10.0 0.0, 20.0 0.0, 30.0 0.0));
        assert_eq!(
            ls.substring_by_distance(&Euclidean, 5.0, 10.0).unwrap(),
            wkt!(LINESTRING(5.0 0.0, 10.0 0.0))
        );
        assert_eq!(
            ls.substring_by_distance(&Euclidean, 5.0, 20.0).unwrap(),
            wkt!(LINESTRING(5.0 0.0, 10.0 0.0, 20.0 0.0))
        );
    }

    mod haversine {
        use super::*;
        use crate::Haversine;

        #[test]
        fn substring_linestring_haversine_round_trip_full_length() {
            // Full-range substring should be topologically equal to the input under Haversine.
            let line_string = wkt!(LINESTRING(0.0 0.0, 0.0 10.0, 10.0 10.0));
            let total = crate::Length::length(&Haversine, &line_string);
            let result = line_string
                .substring_by_distance(&Haversine, 0.0, total)
                .unwrap();
            // Endpoints should match exactly; interior vertex preserved.
            assert_eq!(result.0.first(), line_string.0.first());
            assert_eq!(result.0.last(), line_string.0.last());
            assert_eq!(result.0[1], line_string.0[1]);
            assert_eq!(result.0.len(), line_string.0.len());
        }
    }
}
