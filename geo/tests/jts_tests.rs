use jts_test_runner::TestRunner;

#[test]
fn test_centroid() {
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
