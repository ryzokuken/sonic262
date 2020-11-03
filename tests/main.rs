use sonic262;
use std::path::PathBuf;

#[test]
fn test262_harness_tests() {
    assert_eq!(
        0,
        sonic262::run_test(
            PathBuf::from("./tests/fixtures/test"),
            PathBuf::from("./tests/fixtures/harness"),
        )
        .unwrap()
        .fail
    )
}
