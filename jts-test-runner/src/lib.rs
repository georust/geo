mod input;

use geo::{Distance, Euclidean, Geometry, Line, LinesIter, MultiPolygon, Point, Relate};
use input::Operation;

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

/// Checks a buffer result against JTS's, with a tolerance scaled by the buffer distance.
pub fn check_buffer_test_case(
    actual: &Geometry,
    expected: &Geometry,
    distance: f64,
) -> std::result::Result<(), String> {
    if actual.relate(expected).is_equal_topo() {
        return Ok(());
    }
    let (actual, expected) = (to_multi_polygon(actual), to_multi_polygon(expected));

    // JTS's own BufferResultMatcher allows the boundaries to be up to `|distance| / 100` apart.
    let tolerance = distance.abs() / 100.0;
    let boundary_deviation = boundary_hausdorff_distance(&actual, &expected);

    if boundary_deviation > tolerance {
        Err(format!("boundary deviation: {boundary_deviation}, tolerance: {tolerance}\nactual:\n  {actual:?}\nexpected:\n  {expected:?}"))
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
fn boundary_hausdorff_distance(a: &MultiPolygon, b: &MultiPolygon) -> f64 {
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
        let expected_test_count: usize = 11723;
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
