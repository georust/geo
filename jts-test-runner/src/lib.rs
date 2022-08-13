mod input;
use input::Operation;

mod runner;
pub use runner::TestRunner;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

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
    fn test_relate() {
        init_logging();
        let mut runner = TestRunner::new().matching_filename_glob("*Relate*.xml");
        runner.run().expect("test cases failed");

        // sanity check that *something* was run
        assert!(
            runner.failures().len() + runner.successes().len() > 0,
            "No tests were run."
        );

        if !runner.failures().is_empty() {
            let failure_text = runner
                .failures()
                .iter()
                .map(|failure| format!("{}", failure))
                .collect::<Vec<String>>()
                .join("\n");
            panic!(
                "{} failures / {} successes in JTS test suite:\n{}",
                runner.failures().len(),
                runner.successes().len(),
                failure_text
            );
        }
    }

    #[test]
    // several of the ConvexHull tests are currently failing
    fn test_all_general() {
        init_logging();
        let mut runner = TestRunner::new().with_overlay_precision_floating();
        runner.run().expect("test cases failed");

        if !runner.failures().is_empty() {
            let failure_text = runner
                .failures()
                .iter()
                .map(|failure| format!("{}", failure))
                .collect::<Vec<String>>()
                .join("\n");
            panic!(
                "{} failures / {} successes in JTS test suite:\n{}",
                runner.failures().len(),
                runner.successes().len(),
                failure_text
            );
        }

        // sanity check that the expected number of tests were run.
        //
        // We'll need to increase this number as more tests are added, but it should never be
        // decreased.
        let expected_test_count: usize = 2213;
        let actual_test_count = runner.failures().len() + runner.successes().len();
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
                    "Great, looks like we're running new tests. Just increase `expected_test_count` to {}",
                    actual_test_count
                );
            }
        }
    }

    #[test]
    fn test_boolean_ops() {
        init_logging();
        let mut runner = TestRunner::new()
            .matching_filename_glob("*Overlay*.xml")
            .with_overlay_precision_floating();
        runner.run().expect("test cases failed");

        // sanity check that *something* was run
        assert!(
            runner.failures().len() + runner.successes().len() > 0,
            "No tests were run."
        );

        if !runner.failures().is_empty() {
            let failure_text = runner
                .failures()
                .iter()
                .map(|failure| format!("{}", failure))
                .collect::<Vec<String>>()
                .join("\n");
            panic!(
                "{} failures / {} successes in JTS test suite:\n{}",
                runner.failures().len(),
                runner.successes().len(),
                failure_text
            );
        }
    }
}
