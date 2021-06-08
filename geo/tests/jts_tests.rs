use jts_test_runner::TestRunner;

fn init_logging() {
    use std::sync::Once;
    static LOG_SETUP: Once = Once::new();
    LOG_SETUP.call_once(|| {
        pretty_env_logger::init();
    });
}

#[test]
fn test_all_general() {
    init_logging();

    let mut runner = TestRunner::new();
    runner.run().expect("test cases failed");

    // sanity check that *something* was run
    assert!(
        runner.failures().len() + runner.successes().len() > 0,
        "No tests were run."
    );

    if runner.failures().is_empty() {
        log::info!(
            "All {} cases succeeded in JTS test suite.",
            runner.successes().len()
        );
    } else {
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
