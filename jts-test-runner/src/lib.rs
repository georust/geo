mod input;

use approx::relative_eq;
use geo::{
    Area, BooleanOps, Distance, Euclidean, Geometry, HasDimensions, Length, Line, LinesIter,
    MultiPolygon, Point, Relate,
};
use input::Operation;
use log::debug;

mod runner;
pub use runner::TestRunner;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

/// ```
/// use jts_test_runner::assert_jts_tests_succeed;
/// assert_jts_tests_succeed("*Relate*.xml");
/// ```
pub fn assert_jts_tests_succeed(pattern: &str) {
    let mut runner = TestRunner::new().matching_filename_glob(pattern);
    runner.run().expect("test cases failed");

    // sanity check that *something* was run
    assert!(
        runner.unexpected_failures().len() + runner.successes().len() > 0,
        "No tests were run."
    );

    if !runner.unexpected_failures().is_empty() {
        let failure_text = runner
            .unexpected_failures()
            .iter()
            .map(|failure| format!("{failure}"))
            .collect::<Vec<String>>()
            .join("\n");

        panic!(
            "{} unexpected failures / {} successes in JTS test suite:\n{}",
            runner.unexpected_failures().len(),
            runner.successes().len(),
            failure_text
        );
    }

    for success in runner.successes() {
        if runner.expected_failures().contains(&success.test_id) {
            panic!("Test {:?} was expected to fail, but it passed. Did you fix something? Good job! Update `expected_failures`.", success.test_id);
        }
    }
}

const BUFFER_ERROR_RATIO: f64 = 0.0014;

pub fn check_buffer_test_case(
    actual: &Geometry,
    expected: &Geometry,
) -> std::result::Result<(), String> {
    // This error threshold is arbitrary. I ratcheted it down until tests started failing.
    // Manually inspecting the output of the borderline cases appeared subjectively "reasonable".
    //
    // In particular, we seem to diverge the most when doing large subtractive (negative buffers)
    // from complex geometries (e.g., polygons with narrow arms that get wholly erased)
    //
    // To inspect which ones fail, you can lower this tolerance and see which test cases fail.
    check_buffer_test_case_with_error_ratio(actual, expected, BUFFER_ERROR_RATIO)
}

pub fn check_buffer_test_case_with_error_ratio(
    actual: &Geometry,
    expected: &Geometry,
    max_error_ratio: f64,
) -> std::result::Result<(), String> {
    let im = actual.relate(expected);
    if im.is_equal_topo() {
        debug!("Buffer success (equal_topo)");
        Ok(())
    } else if relative_eq!(actual, expected) {
        debug!("Buffer success (relative eq)");
        Ok(())
    } else {
        let diff = match (expected, &actual) {
            (Geometry::MultiPolygon(expected), Geometry::MultiPolygon(actual)) => {
                expected.xor(actual)
            }
            (Geometry::Polygon(expected), Geometry::MultiPolygon(actual)) => expected.xor(actual),
            (Geometry::MultiPolygon(expected), Geometry::Polygon(actual)) => expected.xor(actual),
            (Geometry::Polygon(expected), Geometry::Polygon(actual)) => expected.xor(actual),
            _ => unreachable!("unexpected comparison"),
        };

        let diff_area = diff.unsigned_area();
        let area_error_ratio = diff_area / expected.unsigned_area();

        if area_error_ratio > max_error_ratio {
            let error_description = format!("diff_area: {diff_area}, area_error_ratio: {area_error_ratio}\nactual:\n  {actual:?}\nexpected:\n  {expected:?}");
            Err(error_description)
        } else {
            Ok(())
        }
    }
}

/// Checks a buffer result against JTS's, with tolerances scaled by the buffer distance.
///
/// Buffers approximate round joins and caps with straight segments, and libraries place those
/// vertices differently, so correct results differ by a fraction of the distance. JTS's
/// default of 8 segments per quarter circle puts a segment up to `|distance| * (1 - cos(π/16))`
/// (about 2% of the distance) inside the true arc, so this is the tolerance: the boundaries must
/// be within it of each other.
///
/// JTS's own `BufferResultMatcher` uses `|distance| / 100` and requires the symmetric difference
/// to be under 0.1% of the expected area. JTS 1.20 fails that against 15 of the cases in
/// `TestBufferExternal2.xml`, whose results come from an older JTS. The area ratio also fails
/// for results that are small compared to the distance, e.g. the slivers left by a negative
/// buffer. So the area check here also passes if the symmetric difference is, on average, no
/// wider than the tolerance along the expected boundary.
pub fn check_buffer_test_case_with_distance(
    actual: &Geometry,
    expected: &Geometry,
    distance: f64,
) -> std::result::Result<(), String> {
    if actual.relate(expected).is_equal_topo() || relative_eq!(actual, expected) {
        return Ok(());
    }
    let (actual, expected) = (to_multi_polygon(actual), to_multi_polygon(expected));
    if actual.is_empty() && expected.is_empty() {
        return Ok(());
    }

    let tolerance = distance.abs() * (1.0 - (std::f64::consts::PI / 16.0).cos());
    let deviation = boundary_deviation(&actual, &expected);
    let diff_area = expected.xor(&actual).unsigned_area();
    let area = expected.unsigned_area();
    let perimeter: f64 = expected
        .lines_iter()
        .map(|line| Euclidean.length(&line))
        .sum();

    if deviation > tolerance {
        Err(format!("boundary deviation: {deviation}, tolerance: {tolerance}\nactual:\n  {actual:?}\nexpected:\n  {expected:?}"))
    } else if diff_area > BUFFER_ERROR_RATIO * area && diff_area > tolerance * perimeter {
        Err(format!("diff_area: {diff_area}, area_error_ratio: {}, mean diff width: {}, tolerance: {tolerance}\nactual:\n  {actual:?}\nexpected:\n  {expected:?}", diff_area / area, diff_area / perimeter))
    } else {
        Ok(())
    }
}

fn to_multi_polygon(geometry: &Geometry) -> MultiPolygon {
    match geometry {
        Geometry::Polygon(polygon) => MultiPolygon::new(vec![polygon.clone()]),
        Geometry::MultiPolygon(multi_polygon) => multi_polygon.clone(),
        _ => unreachable!("unexpected buffer result: {geometry:?}"),
    }
}

/// The largest distance from a point on either boundary to the other boundary, sampling each
/// segment at quarters like JTS's `DiscreteHausdorffDistance` with a densify fraction of 0.25.
fn boundary_deviation(a: &MultiPolygon, b: &MultiPolygon) -> f64 {
    let directed = |from: &MultiPolygon, to: &MultiPolygon| {
        let to: Vec<Line> = to.lines_iter().collect();
        from.lines_iter()
            .flat_map(|line| (0..4).map(move |i| line.start + line.delta() * (i as f64 / 4.0)))
            .map(|coord| {
                to.iter()
                    .map(|line| Euclidean.distance(&Point::from(coord), line))
                    .fold(f64::INFINITY, f64::min)
            })
            .fold(0.0, f64::max)
    };
    directed(a, b).max(directed(b, a))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    fn init_logging() {
        use std::sync::Once;
        static LOG_SETUP: Once = Once::new();
        LOG_SETUP.call_once(|| {
            pretty_env_logger::init();
        });
    }

    #[test]
    // several of the ConvexHull tests are currently failing
    fn test_all_general() {
        init_logging();
        let mut runner = TestRunner::new();
        runner.run().expect("test cases failed");

        if !runner.unexpected_failures().is_empty() {
            let failure_text = runner
                .unexpected_failures()
                .iter()
                .map(|failure| format!("{failure}"))
                .collect::<Vec<String>>()
                .join("\n");
            panic!(
                "{} unexpected_failures / {} successes in JTS test suite:\n{}",
                runner.unexpected_failures().len(),
                runner.successes().len(),
                failure_text
            );
        }

        // sanity check that the expected number of tests were run.
        //
        // We'll need to increase this number as more tests are added, but it should never be
        // decreased.
        let expected_test_count: usize = 11725;
        let actual_test_count = runner.unexpected_failures().len() + runner.successes().len();
        match actual_test_count.cmp(&expected_test_count) {
            Ordering::Less => {
                panic!(
                    "We're running {} less test cases than before. What happened to them?",
                    expected_test_count - actual_test_count
                );
            }
            Ordering::Equal => {}
            Ordering::Greater => {
                panic!(
                    "Great, looks like we're running new tests. Just increase `expected_test_count` to {actual_test_count}"
                );
            }
        }
    }
}
