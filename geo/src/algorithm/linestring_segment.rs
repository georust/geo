use crate::algorithm::{Densify, Length, LinesIter};
use crate::geometry::{Coord, LineString, MultiLineString};
use crate::line_measures::{Euclidean, Haversine, InterpolateLine};

/// Segments a LineString into `segment_count` equal length LineStrings as a MultiLineString
/// using Euclidean distance calculations.  See `LineStringSegmentizeHaversine`
/// if you are dealing with geographic coordinates (lat/lon).
///
/// `None` will be returned when `segment_count` is equal to 0 or when a point
/// cannot be interpolated on a `Line` segment.
///
/// # Examples
/// ```
/// use geo::{LineString, MultiLineString, LineStringSegmentize};
/// // Create a simple line string
/// let lns: LineString<f64> = vec![[0.0, 0.0], [1.0, 2.0], [3.0, 6.0]].into();
/// // Segment it into 6 LineStrings inside of a MultiLineString
/// let segmentized = lns.line_segmentize(6).unwrap();
/// // Compare the number of elements
/// assert_eq!(6, segmentized.0.len());
///```
pub trait LineStringSegmentize {
    fn line_segmentize(&self, segment_count: usize) -> Option<MultiLineString>;
}

/// Segments a LineString into `segment_count` equal length LineStrings as a MultiLineString
/// using Haversine distance calculations. Use this over `LineStringSegmentize`
/// when using data from a geographic coordinate system.
///
/// `None` will be returned when `segment_count` is equal to 0 or when a point
/// cannot be interpolated on a `Line` segment.
///
/// # Examples
/// ```
/// use geo::{LineString, MultiLineString, LineStringSegmentizeHaversine};
/// // Create a simple line string
/// let lns: LineString<f64> = vec![[0.0, 0.0], [1.0, 2.0], [3.0, 6.0]].into();
/// // Segment it into 6 LineStrings inside of a MultiLineString
/// let segmentized = lns.line_segmentize_haversine(6).unwrap();
/// // Compare the number of elements
/// assert_eq!(6, segmentized.0.len());
///```
pub trait LineStringSegmentizeHaversine {
    fn line_segmentize_haversine(&self, segment_count: usize) -> Option<MultiLineString>;
}

macro_rules! implement_segmentize {
    ($trait_name:ident, $method_name:ident, $metric_space:expr) => {
        impl $trait_name for LineString {
            fn $method_name(&self, n: usize) -> Option<MultiLineString> {
                if (n == usize::MIN) || (n == usize::MAX) {
                    return None;
                } else if n == 1 {
                    let mlns = MultiLineString::from(self.clone());
                    return Some(mlns);
                }

                let mut res_coords: Vec<Vec<Coord>> = Vec::with_capacity(n);
                let total_length = $metric_space.length(self);
                let mut cum_length = 0_f64;
                let segment_prop = (1_f64) / (n as f64);
                let segment_length = total_length * segment_prop;
                let densified = $metric_space.densify(self, segment_length - f64::EPSILON);

                if densified.lines().count() == n {
                    let linestrings = densified
                        .lines()
                        .map(LineString::from)
                        .collect::<Vec<LineString>>();
                    return Some(MultiLineString::new(linestrings));
                }

                let n_lines = densified.lines().count();
                let lns = densified.lines_iter();
                let mut ln_vec: Vec<Coord> = Vec::new();

                for (i, segment) in lns.enumerate() {
                    if i == 0 {
                        ln_vec.push(segment.start)
                    }

                    let length = $metric_space.length(&segment);
                    cum_length += length;

                    if (cum_length >= segment_length) && (i != (n_lines - 1)) {
                        let remainder = cum_length - segment_length;
                        let endpoint = $metric_space
                            .point_at_ratio_from_start(&segment, (length - remainder) / length);
                        ln_vec.push(endpoint.into());
                        let to_push = ln_vec.drain(..);
                        res_coords.push(to_push.collect::<Vec<Coord>>());

                        if i != n_lines {
                            ln_vec.push(endpoint.into());
                        }
                        cum_length = remainder;
                    }
                    ln_vec.push(segment.end);
                }

                res_coords.push(ln_vec);
                let res_lines = res_coords
                    .into_iter()
                    .map(LineString::new)
                    .collect::<Vec<LineString>>();
                Some(MultiLineString::new(res_lines))
            }
        }
    };
}

implement_segmentize!(LineStringSegmentize, line_segmentize, Euclidean);
implement_segmentize!(
    LineStringSegmentizeHaversine,
    line_segmentize_haversine,
    Haversine
);

#[cfg(test)]
mod test {
    use approx::RelativeEq;

    use super::*;
    use crate::LineString;

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

        assert_eq!(Euclidean.length(&segments), Euclidean.length(&linestring));
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
            Euclidean.length(&linestring),
            Euclidean.length(&segments),
            epsilon = f64::EPSILON
        );
    }

    #[test]
    fn two_coords() {
        let linestring: LineString = vec![[0.0, 0.0], [0.0, 1.0]].into();

        let segments = linestring.line_segmentize(5).unwrap();
        assert_eq!(segments.0.len(), 5);
        assert_relative_eq!(
            Euclidean.length(&linestring),
            Euclidean.length(&segments),
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
            Euclidean.length(&linestring),
            Euclidean.length(&segments),
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
            .map(|x| Euclidean.length(&x))
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
            Euclidean.length(&linestring),
            Euclidean.length(&segments),
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

    #[test]
    fn haversine_n_elems() {
        let linestring: LineString = vec![
            [-3.19416, 55.95524],
            [-3.19352, 55.95535],
            [-3.19288, 55.95546],
        ]
        .into();

        let n = 8;

        let segments = linestring.line_segmentize_haversine(n).unwrap();
        assert_eq!(n, segments.0.len());
    }

    #[test]
    fn haversine_segment_length() {
        let linestring: LineString = vec![
            [-3.19416, 55.95524],
            [-13.19352, 65.95535],
            [-23.19288, 75.95546],
        ]
        .into();

        let n = 8;

        let total_length = Haversine.length(&linestring);
        let expected_segment_length = total_length / (n as f64);

        let segments = linestring.line_segmentize_haversine(n).unwrap();
        for segment in segments {
            assert_relative_eq!(
                Haversine.length(&segment),
                expected_segment_length,
                epsilon = 1e-9
            );
        }
    }

    #[test]
    fn haversine_total_length() {
        let linestring: LineString = vec![
            [-3.19416, 55.95524],
            [-3.19352, 55.95535],
            [-3.19288, 55.95546],
        ]
        .into();

        assert_relative_eq!(Haversine.length(&linestring), 83.3523000093029);

        let n = 8;

        let segments = linestring.line_segmentize_haversine(n).unwrap();

        // different at 12th decimal which is a picometer
        assert_relative_eq!(
            Haversine.length(&linestring),
            Haversine.length(&segments),
            epsilon = 1e-11
        );
    }
}
