#[macro_use]
extern crate log;

mod input;
use input::Operation;

mod runner;
pub use runner::TestRunner;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    fn init_logging() {
        use std::sync::Once;
        static LOG_SETUP: Once = Once::new();
        LOG_SETUP.call_once(|| {
            pretty_env_logger::init();
        });
    }

    #[test]
    fn test_centroid() {
        init_logging();
        let mut runner = TestRunner::new().matching_filename_glob("*Centroid.xml");
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
    #[ignore]
    fn test_all_general() {
        init_logging();
        let mut runner = TestRunner::new();
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
